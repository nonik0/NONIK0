use avrxmega_hal::pac::NVMCTRL;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Setting {
    Version = 0x00,
    Brightness = 0x01,
    Current = 0x02,
    Name = 0x03, // 8 bytes wide
    LastMode = 0x0B,
    RandomPage = 0x0C,
    SensorPage = 0x0D, // 0x1400 + 0x0D = 0x140D, in decimal 5133
}

//pub struct SavedSettings(crate::Eeprom);
pub struct SavedSettings {
    nvmctrl: NVMCTRL,
    //cpu: CPU,
}

impl SavedSettings {
    // pub fn new(eeprom: crate::Eeprom) -> Self {
    //     SavedSettings { 0: eeprom }
    // }

    //pub fn new(nvmctrl: NVMCTRL, cpu: CPU) -> Self {
    pub fn new(nvmctrl: NVMCTRL) -> Self {
        SavedSettings {
            nvmctrl: nvmctrl,
            //cpu: cpu,
        }
    }

    #[inline(always)]
    pub fn read_setting(&self, setting: Setting, buf: &mut [u8]) {
        //self.0.read(setting as u16, buf).unwrap();
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.raw_read_byte(setting as u16 + i as u16)
        }
    }

    #[inline(always)]
    pub fn read_setting_byte(&self, setting: Setting) -> u8 {
        //self.0.read_byte(setting as u16)
        self.raw_read_byte(setting as u16)
    }

    #[inline(always)]
    pub fn save_setting(&mut self, setting: Setting, buf: &[u8]) {
        //self.0.write(setting as u16, buf).unwrap();
        for (i, byte) in buf.iter().enumerate() {
            self.raw_write_byte(setting as u16 + i as u16, *byte)
        }
    }

    #[inline(always)]
    pub fn save_setting_byte(&mut self, setting: Setting, value: u8) {
        //self.0.write_byte(setting as u16, value);
        self.raw_write_byte(setting as u16, value);
    }

    #[inline(always)]
    pub fn raw_read_byte(&self, offset: u16) -> u8 {
        //self.0.read_byte(setting as u16)
        unsafe { *(0x1400 as *const u8).add(offset as usize) as u8 }
    }

    #[inline(always)]
    fn raw_write_byte(&mut self, offset: u16, value: u8) {        
        //self.0.write_byte(setting as u16, value);

        while self.nvmctrl.status().read().eebusy().bit_is_set()
            || self.nvmctrl.status().read().fbusy().bit_is_set()
        {}

        avr_device::interrupt::disable();

        unsafe {
            let eeprom_ptr = (0x1400 as *mut u8).add(offset as usize);
            core::ptr::write_volatile(eeprom_ptr, value);
            core::ptr::write_volatile(0x34 as *mut u8, 0x9D); // unlock nvmctrl ctrla for cmd
        }

        //self.cpu.ccp().write(|w| w.ccp().spm()); // unlock nvmctrl ctrla for cmd
        self.nvmctrl.ctrla().write(|w| w.cmd().pageerasewrite());

        unsafe {
            avr_device::interrupt::enable();
        }
    }
}
