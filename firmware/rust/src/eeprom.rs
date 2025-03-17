use avr_device::interrupt::Mutex;
use avrxmega_hal::pac::{CPU, NVMCTRL};
use core::arch::asm;
use core::cell::RefCell;

pub struct EepromSettings {
    pub version: u8,
    pub brightness: u8,
    pub current: crate::DisplayPeakCurrent,
    pub name: [u8; 8],
}

impl EepromSettings {
    pub fn read() -> Self {
        let eeprom = crate::eeprom::Eeprom::instance();
        let version = eeprom.load_setting(crate::eeprom::EepromOffset::Version);

        // TODO: version check

        let brightness = match eeprom.load_setting(crate::eeprom::EepromOffset::Brightness) {
            value @ 0..=15 => value,
            _ => crate::DEFAULT_BRIGHTNESS,
        };

        let current = match eeprom.load_setting(crate::eeprom::EepromOffset::Current) {
            0b0010_0000 => crate::DisplayPeakCurrent::Max4_0Ma,
            0b0001_0000 => crate::DisplayPeakCurrent::Max6_4Ma,
            0b0000_0000 => crate::DisplayPeakCurrent::Max9_3Ma,
            0b0011_0000 => crate::DisplayPeakCurrent::Max12_8Ma,
            _ => crate::DEFAULT_CURRENT,
        };

        let mut name = [0; 8];
        eeprom.load_setting_slice(EepromOffset::Name, &mut name);
        let name = if name.iter().any(|&byte| byte == Eeprom::EEPROM_UNINITIALIZED) {
            *b" NONIK0 "
        } else {
            name
        };

        EepromSettings {
            version,
            brightness,
            current,
            name,
        }
    }
}

// 804 -> 128B EEPROM
// 1604 -> 256B EEPROM

static EEPROM_STATE: Mutex<RefCell<Option<EepromState>>> = Mutex::new(RefCell::new(None));

struct EepromState {
    //cpu: CPU,
    nvmctrl: NVMCTRL,
}

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum EepromOffset {
    Version = 0x00,
    Brightness = 0x01,
    Current = 0x02,
    Name = 0x03, // 8 bytes wide
                 //NextSetting = 0x0B,
}

pub struct Eeprom {}

impl Eeprom {
    const EEPROM_UNINITIALIZED: u8 = 0xFF;
    const EEPROM_ADDR_START: u16 = 0x1400;

    pub fn init(nvmctrl: NVMCTRL) {
        avr_device::interrupt::free(|cs| {
            EEPROM_STATE
                .borrow(cs)
                .replace(Some(EepromState { nvmctrl }));
        });
    }

    pub fn instance() -> Self {
        Eeprom {}
    }

    pub fn save_setting(&mut self, offset: EepromOffset, value: u8) {
        self.raw_write_byte(offset as u16, value);
    }

    pub fn save_setting_slice(&mut self, start_offset: EepromOffset, values: &[u8]) {
        for (i, &value) in values.iter().enumerate() {
            self.raw_write_byte(start_offset as u16 + i as u16, value);
        }
    }

    pub fn load_setting(&self, offset: EepromOffset) -> u8 {
        self.raw_read_byte(offset as u16)
    }

    pub fn load_setting_slice(&self, start_offset: EepromOffset, buffer: &mut [u8]) {
        for (i, value) in buffer.iter_mut().enumerate() {
            *value = self.raw_read_byte(start_offset as u16 + i as u16);
        }
    }

    #[inline(always)]
    fn wait_until_ready(&self, eeprom_state: &mut EepromState) {
        while eeprom_state.nvmctrl.status.read().eebusy().bit()
            | eeprom_state.nvmctrl.status.read().fbusy().bit()
        {}
    }

    fn raw_read_byte(&self, offset: u16) -> u8 {
        avr_device::interrupt::free(|cs| {
            if let Some(ref mut eeprom_state) = *EEPROM_STATE.borrow(cs).borrow_mut() {
                self.wait_until_ready(eeprom_state);
                // TODO: EEPROM addr offset not in avr-device crate
                unsafe { *(Self::EEPROM_ADDR_START as *const u8).add(offset as usize) as u8 }
            } else {
                Self::EEPROM_UNINITIALIZED
            }
        })
    }

    // TODO: maybe try to get this to work instead of assembly
    // fn raw_write_byte(&mut self, offset: u16, data: u8) {
    //     avr_device::interrupt::free(|cs| {
    //         if let Some(ref mut eeprom_state) = *EEPROM_STATE.borrow(cs).borrow_mut() {
    //             self.wait_until_ready(eeprom_state);
    //             eeprom_state
    //                 .nvmctrl
    //                 .addr
    //                 .write(|w| w.bits(Self::EEPROM_ADDR_START + offset));
    //             eeprom_state.nvmctrl.data.write(|w| w.bits(data as u16));
    //             eeprom_state.cpu.ccp.write(|w| w.ccp().spm());
    //             eeprom_state
    //                 .nvmctrl
    //                 .ctrla
    //                 .write(|w| w.cmd().pageerasewrite());
    //         }
    //     });
    // }

    #[cfg(target_arch = "avr")]
    fn raw_write_byte(&mut self, offset: u16, data: u8) {
        // adapted from https://github.com/SpenceKonde/megaTinyCore/blob/4d0d75660ccfa72de79c9c4f15a8cd17c9f0ed16/megaavr/libraries/EEPROM/src/EEPROM.h#L78
        // TODO: how to specify upper reg constraint? can avoid specifying r17 and r18
        let address = Self::EEPROM_ADDR_START + offset;
        unsafe {
            asm!(
                "ldi r30, 0x00", // Z <- 0x1000 (NVMCTRL base address)
                "ldi r31, 0x10", //
                "in r0, 0x3f",   // r0 = SREG (save interrupt state)
                "ldd r18, Z+2",  // load NVMCTRL.STATUS (Z+2, 0x1002)
                "andi r18, 3",   // if NVMCTRL.STATUS.EEBUSY | NVMCTRL.STATUS.FBUSY
                "brne .-6",      // then keep checking
                "cli",           // disable interrupts
                "st X, r17",     // *address = data
                "ldi r18, 0x9D", //
                "out 0x34, r18", // CPU.CCP = 0x9D (SPM unlock)
                "ldi r18, 0x03", //
                "st Z, r18",     // NVMCTRL.CTRLA = 0x03 (ERWP erase/write cmd)
                "out 0x3f, r0",  // SREG = r0 (restore interrupt state)
                in("r17") data,  // r17 = data (needs to be upper reg for ldi)
                in("X") address, // X = address
                out("r18") _,    // clobbered (same as data reg, upper reg for ldi)
                out("r30") _,    // clobbered
                out("r31") _,    // clobbered
            );
        }
    }

    // fn raw_erase_byte(&mut self, address: u16) {
    //     self.raw_write_byte(address, Self::EEPROM_UNINITIALIZED);
    // }
}
