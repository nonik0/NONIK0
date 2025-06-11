use super::ModeHandler;
use crate::{
    Context, Display, DisplayPeakCurrent, Event, Peripherals, SavedSettings,
    Setting as EepromSetting,
};

const BRIGHTNESS_DEFAULT: u8 = 12;
const BRIGHTNESS_MAX: u8 = 16;
const CURRENT_MAX: u8 = 4;
const CURRENT_DEFAULT: u8 = 1;
const BRIGHTNESS_LEVELS: [&[u8]; BRIGHTNESS_MAX as usize] = [
    b"Brite: 0",
    b"Brite: 1",
    b"Brite: 2",
    b"Brite: 3",
    b"Brite: 4",
    b"Brite: 5",
    b"Brite: 6",
    b"Brite: 7",
    b"Brite: 8",
    b"Brite: 9",
    b"Brite:10",
    b"Brite:11",
    b"Brite:12",
    b"Brite:13",
    b"Brite:14",
    b"Brite:15",
];
const CURRENT_LEVELS: [&[u8]; CURRENT_MAX as usize] =
    [b"Cur: 4mA", b"Cur: 6mA", b"Cur: 9mA", b"Cur:13mA"];

enum Setting {
    Brightness,
    Current,
}

pub struct Settings {
    cur_setting: Setting,
    last_update: u16,
    brightness: u8,
    current: u8,
}

impl Settings {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let mut saved_brightness = settings.read_setting_byte(EepromSetting::Brightness);
        let mut saved_current = settings.read_setting_byte(EepromSetting::Current);

        if saved_brightness >= BRIGHTNESS_MAX {
            saved_brightness = BRIGHTNESS_DEFAULT;
        }

        if saved_current >= CURRENT_MAX {
            saved_current = CURRENT_DEFAULT;
        }

        Settings {
            cur_setting: Setting::Brightness,
            last_update: 0,
            brightness: saved_brightness,
            current: saved_current,
        }
    }

    fn current_into(value: u8) -> DisplayPeakCurrent {
        match value {
            0 => DisplayPeakCurrent::Max4_0Ma,
            1 => DisplayPeakCurrent::Max6_4Ma,
            2 => DisplayPeakCurrent::Max9_3Ma,
            _ => DisplayPeakCurrent::Max12_8Ma,
        }
    }

    pub fn apply(&self, display: &mut Display) {
        display.set_brightness(self.brightness).unwrap();
        display
            .set_peak_current(Self::current_into(self.current))
            .unwrap();
    }
}

impl ModeHandler for Settings {
    #[inline(never)]
    fn update(
        &mut self,
        event: &Option<Event>,
        context: &mut Context,
        peripherals: &mut Peripherals,
    ) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            update = true;
            match event {
                Event::LeftHeld => {
                    let saved_brightness = context
                        .settings
                        .read_setting_byte(EepromSetting::Brightness);
                    let saved_current = context.settings.read_setting_byte(EepromSetting::Current);

                    // Save settings when exiting if they have changed
                    if self.brightness != saved_brightness {
                        context
                            .settings
                            .save_setting_byte(EepromSetting::Brightness, self.brightness);
                    }
                    if self.current != saved_current {
                        context
                            .settings
                            .save_setting_byte(EepromSetting::Current, self.current);
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
                        self.brightness = (self.brightness + BRIGHTNESS_MAX - 1) % BRIGHTNESS_MAX;
                        peripherals.display.set_brightness(self.brightness).unwrap();
                    }
                    Setting::Current => {
                        self.current = (self.current + CURRENT_MAX - 1) % CURRENT_MAX;
                        peripherals
                            .display
                            .set_peak_current(Self::current_into(self.current))
                            .unwrap();
                    }
                },
                Event::RightReleased => match self.cur_setting {
                    Setting::Brightness => {
                        self.brightness = (self.brightness + 1) % BRIGHTNESS_MAX;
                        peripherals.display.set_brightness(self.brightness).unwrap();
                    }
                    Setting::Current => {
                        self.current = (self.current + 1) % CURRENT_MAX;
                        peripherals
                            .display
                            .set_peak_current(Self::current_into(self.current))
                            .unwrap();
                    }
                },
                _ => {}
            }
        }

        if update {
            match self.cur_setting {
                Setting::Brightness => {
                    let buffer = BRIGHTNESS_LEVELS[self.brightness as usize];
                    peripherals.display.print_ascii_bytes(buffer).unwrap();
                }
                Setting::Current => {
                    let buffer = CURRENT_LEVELS[self.current as usize];
                    peripherals.display.print_ascii_bytes(buffer).unwrap();
                }
            }
        }
    }
}
