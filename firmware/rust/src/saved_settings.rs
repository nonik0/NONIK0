#[derive(Clone, Copy)]
#[repr(u16)]
enum SavedSettingOffset {
    Version = 0x00,
    Brightness = 0x01,
    Current = 0x02,
    Name = 0x03, // 8 bytes wide
                 //NextSetting = 0x0B,
}

pub struct SavedSettings(crate::Eeprom);

impl SavedSettings {
    pub fn new(eeprom: crate::Eeprom) -> Self {
        SavedSettings { 0: eeprom }
    }

    #[inline(always)]
    pub fn version(&self) -> u8 {
        self.0.read_byte(SavedSettingOffset::Version as u16)
    }

    #[inline(always)]
    pub fn brightness(&self) -> u8 {
        match self.0.read_byte(SavedSettingOffset::Brightness as u16) {
            value @ 0..=15 => value,
            _ => crate::DEFAULT_BRIGHTNESS,
        }
    }

    #[inline(always)]
    pub fn current(&self) -> crate::DisplayPeakCurrent {
        match self.0.read_byte(SavedSettingOffset::Current as u16) {
            0b0010_0000 => crate::DisplayPeakCurrent::Max4_0Ma,
            0b0001_0000 => crate::DisplayPeakCurrent::Max6_4Ma,
            0b0000_0000 => crate::DisplayPeakCurrent::Max9_3Ma,
            0b0011_0000 => crate::DisplayPeakCurrent::Max12_8Ma,
            _ => crate::DEFAULT_CURRENT,
        }
    }

    #[inline(always)]
    pub fn name(&self) -> [u8; 8] {
        let mut name = [0; 8];
        self.0.read(SavedSettingOffset::Name as u16, &mut name).unwrap();
        if name.iter().any(|&byte| byte == 0xFF) {
            *b" NONIK0 "
        } else {
            name
        }
    }

    pub fn save_brightness(&mut self, brightness: u8) {
        if brightness <= 15 {
            self.0
                .write_byte(SavedSettingOffset::Brightness as u16, brightness);
        }
    }

    pub fn save_current(&mut self, current: crate::DisplayPeakCurrent) {
        let value = match current {
            crate::DisplayPeakCurrent::Max4_0Ma => 0b0010_0000,
            crate::DisplayPeakCurrent::Max6_4Ma => 0b0001_0000,
            crate::DisplayPeakCurrent::Max9_3Ma => 0b0000_0000,
            crate::DisplayPeakCurrent::Max12_8Ma => 0b0011_0000,
        };
        self.0.write_byte(SavedSettingOffset::Current as u16, value);
    }

    pub fn save_name(&mut self, name: &[u8; 8]) {
        self.0.write(SavedSettingOffset::Name as u16, name).unwrap();
    }
}
