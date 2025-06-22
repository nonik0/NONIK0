use super::ModeHandler;
use crate::{
    utils::{format_buf, format_uint},
    Context, Display, DisplayPeakCurrent, Event, Peripherals, SavedSettings,
    Setting as EepromSetting, NUM_CHARS,
};

const BRIGHTNESS_DEFAULT: u8 = 12;
const BRIGHTNESS_MAX: u8 = 16;
const CURRENT_MAX: u8 = 4;
const CURRENT_DEFAULT: u8 = 1;
const CURRENT_LEVELS: [u8; CURRENT_MAX as usize] = [4, 6, 9, 13];

enum Setting {
    Brightness,
    Current,
    ToneToggle, // tone state is held in context
}

pub struct Settings {
    cur_setting: Setting,
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
        let update = event.is_some() || context.need_update();

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    // Save settings if changed
                    if self.brightness
                        != context
                            .settings
                            .read_setting_byte(EepromSetting::Brightness)
                    {
                        context
                            .settings
                            .save_setting_byte(EepromSetting::Brightness, self.brightness);
                    }
                    if self.current != context.settings.read_setting_byte(EepromSetting::Current) {
                        context
                            .settings
                            .save_setting_byte(EepromSetting::Current, self.current);
                    }
                    if context.tone_enabled
                        != context.settings.read_setting_bool(EepromSetting::Tone)
                    {
                        context
                            .settings
                            .save_setting_bool(EepromSetting::Tone, context.tone_enabled);
                    }
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    self.cur_setting = match self.cur_setting {
                        Setting::Brightness => Setting::Current,
                        Setting::Current => Setting::ToneToggle,
                        Setting::ToneToggle => Setting::Brightness,
                    };
                }
                Event::LeftReleased | Event::RightReleased => {
                    let inc = matches!(event, Event::RightReleased);
                    match self.cur_setting {
                        Setting::Brightness => {
                            self.brightness = if inc {
                                (self.brightness + 1) % BRIGHTNESS_MAX
                            } else {
                                (self.brightness + BRIGHTNESS_MAX - 1) % BRIGHTNESS_MAX
                            };
                            peripherals.display.set_brightness(self.brightness).unwrap();
                        }
                        Setting::Current => {
                            self.current = if inc {
                                (self.current + 1) % CURRENT_MAX
                            } else {
                                (self.current + CURRENT_MAX - 1) % CURRENT_MAX
                            };
                            peripherals
                                .display
                                .set_peak_current(Self::current_into(self.current))
                                .unwrap();
                        }
                        Setting::ToneToggle => {
                            context.tone_enabled = !context.tone_enabled;
                        }
                    }
                }
                _ => {}
            }
        }

        if update {
            let mut buffer = [0u8; NUM_CHARS];
            match self.cur_setting {
                Setting::Brightness => {
                    format_uint(&mut buffer, b"Brite:", self.brightness as u16, 0, None);
                }
                Setting::Current => {
                    format_uint(
                        &mut buffer,
                        b"Cur:",
                        CURRENT_LEVELS[self.current as usize] as u16,
                        0,
                        Some(b"mA"),
                    );
                }
                Setting::ToneToggle => {
                    format_buf(
                        &mut buffer,
                        b"Tone:",
                        if context.tone_enabled { b"On" } else { b"Off" },
                    );
                }
            }
            peripherals.display.print_ascii_bytes(&buffer).unwrap();
        }
    }
}
