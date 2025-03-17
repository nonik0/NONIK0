use crate::{Context, Display, Event};
use super::{names, Mode, NUM_MODES};

pub struct Menu {
    index: u8,
    last_update: u16,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            index: 1,
            last_update: 0,
        }
    }
}

impl Mode for Menu {
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
            let menu_name = names(self.index);
            display.print_ascii_bytes(menu_name).unwrap();
            self.last_update = context.menu_counter;
        }
    }
}
