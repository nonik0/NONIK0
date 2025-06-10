use crate::{Context, Display, Event, Setting, SavedSettings};
use super::{ModeHandler, MODE_NAMES, NUM_MODES};

pub struct Menu {
    index: usize,
    last_update: u16,
}

impl Menu {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let mut saved_index = settings.read_setting_byte(Setting::LastMode) as usize;
        if saved_index >= NUM_MODES {
            saved_index = 1;
        }

        Menu {
            index: saved_index,
            last_update: 0,
        }
    }
}

impl ModeHandler for Menu {
    #[inline(never)]
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftReleased => {
                    if self.index == 1 {
                        self.index = NUM_MODES - 1;
                    } else {
                        self.index -= 1;
                    }

                    update = true;
                }
                Event::RightReleased => {
                    self.index = (self.index + 1) % NUM_MODES;
                    if self.index == 0 {
                        self.index = 1;
                    }

                    update = true;
                }
                Event::RightHeld => {
                    context.to_mode(self.index);
                }
                _ => {}
            }
        }

        if update {
            let menu_name = MODE_NAMES[self.index];
            display.print_ascii_bytes(menu_name).unwrap();
            self.last_update = context.menu_counter;
        }
    }
}
