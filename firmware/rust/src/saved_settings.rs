#[allow(dead_code)]
pub enum Setting {
    Version = 0x00,
    Brightness = 0x01,
    Current = 0x02,
    Name = 0x03, // 8 bytes wide
    LastMode = 0x0B,
    RandomPage = 0x0C,
    SensorPage = 0x0D,
}

pub struct SavedSettings(crate::Eeprom);

impl SavedSettings {
    pub fn new(eeprom: crate::Eeprom) -> Self {
        SavedSettings { 0: eeprom }
    }

    #[inline(always)]
    pub fn read_setting(&self, setting: Setting, buf: &mut [u8]) {
        self.0.read(setting as u16, buf).unwrap();
    }

    #[inline(always)]
    pub fn read_setting_byte(&self, setting: Setting) -> u8 {
        self.0.read_byte(setting as u16)
    }

    #[inline(always)]
    pub fn save_setting(&mut self, setting: Setting, buf: &[u8]) {
        self.0.write(setting as u16, buf).unwrap();
    }

    #[inline(always)]
    pub fn save_setting_byte(&mut self, setting: Setting, value: u8) {
        self.0.write_byte(setting as u16, value);
    }
}
