use super::Mode;
use crate::{Context, Display, Event, NUM_CHARS};

pub struct Nametag {
    name: [u8; NUM_CHARS],
    last_update: u16,
    editing: bool,
    index: usize,
}

impl Nametag {
    pub fn new() -> Self {
        Nametag {
            name: *b"  Nick  ",
            last_update: 0,
            editing: false,
            index: 0, 
        }
    }
}

impl Mode for Nametag {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = false;

        if self.last_update < context.menu_counter {
            self.last_update = context.menu_counter;
            update = true;
        }

        if self.editing {
            if let Some(event) = event {
                match event {
                    Event::LeftHeld => {
                        self.editing = false;
                        update = true;
                    }
                    _ => {}
                }
            }
        } else {
            if let Some(event) = event {
                match event {
                    Event::LeftHeld => {
                        context.menu_counter += 1;
                        context.mode_index = 0;
                        return;
                    }
                    Event::RightHeld => {
                        self.editing = true;
                        update = true;
                    }
                    _ => {}
                }
            }
        }

        if update {
            display.print_ascii_bytes(&self.name).unwrap();
        }
    }
}
