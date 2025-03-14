use super::Mode;
use crate::{Context, Display, DisplayPeakCurrent, Event, DEFAULT_BRIGHTNESS, DEFAULT_CURRENT};

enum Setting {
    Brightness,
    Current,
}

pub struct Settings {
    cur_setting: Setting,
    last_update: u16,
    brightness: u8,
    current: DisplayPeakCurrent,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            cur_setting: Setting::Brightness,
            last_update: 0,
            brightness: DEFAULT_BRIGHTNESS,
            current: DEFAULT_CURRENT,
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
                    context.to_menu();
                    return;
                }
                Event::LeftReleased => match self.cur_setting {
                    Setting::Brightness => self.cur_setting = Setting::Current,
                    Setting::Current => self.cur_setting = Setting::Brightness,
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
                        00 => b"BRITE: 0",
                        01 => b"BRITE: 1",
                        02 => b"BRITE: 2",
                        03 => b"BRITE: 3",
                        04 => b"BRITE: 4",
                        05 => b"BRITE: 5",
                        06 => b"BRITE: 6",
                        07 => b"BRITE: 7",
                        08 => b"BRITE: 8",
                        09 => b"BRITE: 9",
                        10 => b"BRITE:10",
                        11 => b"BRITE:11",
                        12 => b"BRITE:12",
                        13 => b"BRITE:13",
                        14 => b"BRITE:14",
                        15 => b"BRITE:15",
                        _ => b"BRITE:12",
                    };
                    display.print_ascii_bytes(buffer).unwrap();
                }
                Setting::Current => {
                    match self.current {
                        DisplayPeakCurrent::Max4_0Ma => {
                            display.print_ascii_bytes(b"Imax:4mA").unwrap()
                        }
                        DisplayPeakCurrent::Max6_4Ma => {
                            display.print_ascii_bytes(b"Imax:6mA").unwrap()
                        }
                        DisplayPeakCurrent::Max9_3Ma => {
                            display.print_ascii_bytes(b"Imax:9mA").unwrap()
                        }
                        DisplayPeakCurrent::Max12_8Ma => {
                            display.print_ascii_bytes(b"Imx:13mA").unwrap()
                        }
                    }
                }
            }
        }
    }
}
