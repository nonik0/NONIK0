use crate::{Context, Display, Event, Mode, NUM_MODES};

pub struct Menu {
    index: usize,
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
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = self.last_update < context.mode_counter;

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
                Event::LeftHeld | Event::RightHeld => {
                    context.mode_index = self.index;
                }
                _ => {}
            }
        }

        if update {
            let menu_name = match self.index {
                1 => *b"Nametag ",
                2 => *b"  Game  ",
                3 => *b"  Anim  ",
                _ => *b"  ????  ",
            };
            display.print_ascii_bytes(&menu_name).unwrap();
            self.last_update = context.mode_counter;
        }
    }
}
