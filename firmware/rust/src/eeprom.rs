use avr_device::interrupt::Mutex;
use avrxmega_hal::pac::{CPU, NVMCTRL};
use core::cell::RefCell;

// 804 -> 128B EEPROM
// 1604 -> 256B EEPROM

static EEPROM_STATE: Mutex<RefCell<Option<EepromState>>> = Mutex::new(RefCell::new(None));

struct EepromState {
    cpu: CPU,
    nvmctrl: NVMCTRL,
}

#[repr(u16)]
pub enum Setting {
    Version = 0x00,
    Name = 0x01, // 8 bytes wide
    Brightness = 0x09,
    Current = 0x0A,
    Mode = 0x0B,
}

pub struct Eeprom {}

impl Eeprom {
    pub fn init(cpu: CPU, nvmctrl: NVMCTRL) {
        avr_device::interrupt::free(|cs| {
            EEPROM_STATE
                .borrow(cs)
                .replace(Some(EepromState { cpu, nvmctrl }));
        });
    }

    pub fn instance() -> Self {
        Eeprom {}
    }

    pub fn save_setting(&mut self, offset: Setting, value: u8) {
        self.raw_write_byte(offset as u16, value);
    }

    pub fn load_setting(&self, offset: Setting) -> u8 {
        self.raw_read_byte(offset as u16)
    }

    #[inline(always)]
    fn wait_until_ready(&self, eeprom_state: &mut EepromState) {
        while eeprom_state.nvmctrl.status.read().eebusy().bit()
            | eeprom_state.nvmctrl.status.read().fbusy().bit()
        {}
        // TODO: check
    }

    fn raw_read_byte(&self, address: u16) -> u8 {
        avr_device::interrupt::free(|cs| {
            if let Some(ref mut eeprom_state) = *EEPROM_STATE.borrow(cs).borrow_mut() {
                self.wait_until_ready(eeprom_state);
                // TODO: EEPROM addr offset not in avr-device
                unsafe { *((0x1400 as *const u8).add(address as usize)) }
            } else {
                0xAA
            }
        })
    }

    fn raw_write_byte(&mut self, address: u16, data: u8) {
        avr_device::interrupt::free(|cs| {
            if let Some(ref mut eeprom_state) = *EEPROM_STATE.borrow(cs).borrow_mut() {
                eeprom_state.nvmctrl.addr.write(|w| w.bits(address));
                eeprom_state.nvmctrl.data.write(|w| w.bits(data as u16));
                eeprom_state.cpu.ccp.write(|w| w.ccp().spm());
                eeprom_state
                    .nvmctrl
                    .ctrla
                    .write(|w| w.cmd().pageerasewrite());
            }
        });
    }

    fn raw_erase_byte(&mut self, address: u16) {
        avr_device::interrupt::free(|cs| {
            if let Some(ref mut eeprom_state) = *EEPROM_STATE.borrow(cs).borrow_mut() {
                eeprom_state.nvmctrl.addr.write(|w| w.bits(address));
                eeprom_state.cpu.ccp.write(|w| w.ccp().spm());
                eeprom_state.nvmctrl.ctrla.write(|w| w.cmd().pageerase());
            }
        });
    }
}
