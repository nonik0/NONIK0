use super::Mode;
use crate::{
    eeprom::{Eeprom, EepromOffset as EepromSetting},
    Context, Display, DisplayPeakCurrent, Event,
};

enum Setting {
    Brightness,
    Current,
}

pub struct Settings {
    cur_setting: Setting,
    last_update: u16,
    brightness: u8,
    current: DisplayPeakCurrent,
    saved_brightness: u8,
    saved_current: DisplayPeakCurrent,
}

impl Settings {
    pub fn new_with_settings(brightness: u8, current: DisplayPeakCurrent) -> Self {
        Settings {
            cur_setting: Setting::Brightness,
            last_update: 0,
            brightness,
            current,
            saved_brightness: brightness,
            saved_current: current,
        }
    }
}

impl Mode for Settings {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            update = true;
            match event {
                Event::LeftHeld => {
                    // Save settings when exiting if they have changed
                    if self.brightness != self.saved_brightness {
                        Eeprom::instance().save_setting(EepromSetting::Brightness, self.brightness);
                        self.saved_brightness = self.brightness;
                    }
                    if self.current != self.saved_current {
                        Eeprom::instance().save_setting(EepromSetting::Current, self.current as u8);
                        self.saved_current = self.current;
                    }
                    context.to_menu();
                    return;
                }
                Event::RightHeld => match self.cur_setting {
                    Setting::Brightness => self.cur_setting = Setting::Current,
                    Setting::Current => self.cur_setting = Setting::Brightness,
                },
                Event::LeftReleased => match self.cur_setting {
                    Setting::Brightness => {
                        self.brightness = (self.brightness + 15) % 16;
                        display.set_brightness(self.brightness).unwrap();
                    }
                    Setting::Current => {
                        self.current = match self.current {
                            DisplayPeakCurrent::Max4_0Ma => DisplayPeakCurrent::Max12_8Ma,
                            DisplayPeakCurrent::Max6_4Ma => DisplayPeakCurrent::Max4_0Ma,
                            DisplayPeakCurrent::Max9_3Ma => DisplayPeakCurrent::Max6_4Ma,
                            DisplayPeakCurrent::Max12_8Ma => DisplayPeakCurrent::Max9_3Ma,
                        };
                        display.set_peak_current(self.current).unwrap();
                    }
                },
                Event::RightReleased => match self.cur_setting {
                    Setting::Brightness => {
                        self.brightness = (self.brightness + 1) % 16;
                        display.set_brightness(self.brightness).unwrap();
                    }
                    Setting::Current => {
                        self.current = match self.current {
                            DisplayPeakCurrent::Max4_0Ma => DisplayPeakCurrent::Max6_4Ma,
                            DisplayPeakCurrent::Max6_4Ma => DisplayPeakCurrent::Max9_3Ma,
                            DisplayPeakCurrent::Max9_3Ma => DisplayPeakCurrent::Max12_8Ma,
                            DisplayPeakCurrent::Max12_8Ma => DisplayPeakCurrent::Max4_0Ma,
                        };
                        display.set_peak_current(self.current).unwrap();
                    }
                },
                _ => {}
            }
        }

        if update {
            match self.cur_setting {
                Setting::Brightness => {
                    let buffer = match self.brightness {
                        00 => b"Brite: 0",
                        01 => b"Brite: 1",
                        02 => b"Brite: 2",
                        03 => b"Brite: 3",
                        04 => b"Brite: 4",
                        05 => b"Brite: 5",
                        06 => b"Brite: 6",
                        07 => b"Brite: 7",
                        08 => b"Brite: 8",
                        09 => b"Brite: 9",
                        10 => b"Brite:10",
                        11 => b"Brite:11",
                        12 => b"Brite:12",
                        13 => b"Brite:13",
                        14 => b"Brite:14",
                        15 => b"Brite:15",
                        _ => b"Brite:12",
                    };
                    display.print_ascii_bytes(buffer).unwrap();
                }
                Setting::Current => match self.current {
                    DisplayPeakCurrent::Max4_0Ma => display.print_ascii_bytes(b"Cur: 4mA").unwrap(),
                    DisplayPeakCurrent::Max6_4Ma => display.print_ascii_bytes(b"Cur: 6mA").unwrap(),
                    DisplayPeakCurrent::Max9_3Ma => display.print_ascii_bytes(b"Cur: 9mA").unwrap(),
                    DisplayPeakCurrent::Max12_8Ma => {
                        display.print_ascii_bytes(b"Cur: 13mA").unwrap()
                    }
                },
            }

            // Save settings to EEPROM if they have changed
            if self.brightness != self.saved_brightness {
                Eeprom::instance().save_setting(EepromSetting::Brightness, self.brightness);
                self.saved_brightness = self.brightness;
            }
            if self.current != self.saved_current {
                Eeprom::instance().save_setting(EepromSetting::Current, self.current as u8);
                self.saved_current = self.current;
            }
        }
    }
}
