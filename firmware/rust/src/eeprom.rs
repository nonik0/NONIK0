use avrxmega_hal::pac::NVMCTRL;

pub struct Eeprom {
    nvmctrl: NVMCTRL,
}

impl Eeprom {
    pub fn new(nvmctrl: NVMCTRL) -> Self {
        Eeprom { nvmctrl }
    }

    fn is_busy(&self) -> bool {
        self.nvmctrl.status.read().eebusy().bit() | self.nvmctrl.status.read().fbusy().bit()
    }

    fn wait_until_ready(&self) {
        while self.is_busy() {}
        // TODO: check 
    }

    // fn raw_read_byte(&self, address: u16) -> u8 {
    //     self.wait_until_ready();
    //     unsafe {
    //         self.nvmctrl.
    //     }
    // }

    fn raw_write_byte(&mut self, address: u16, data: u8) {
        // TODO: whole u16
        self.nvmctrl.addr.write(|w| w.bits(address));
        self.nvmctrl.data.write(|w| w.bits(data as u16));
        self.nvmctrl.ctrla.write(|w| w.cmd().pageerasewrite());
    }

    fn raw_erase_byte(&mut self, address: u16) {
        // TODO: whole u16
        // TODO: prob not right
        self.nvmctrl.addr.write(|w| w.bits(address));
        self.nvmctrl.ctrla.write(|w| w.cmd().pageerase());
    }
}
