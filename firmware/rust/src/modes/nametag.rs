use super::Mode;
use crate::{Context, Display, Event, NUM_CHARS};

const BLINK_PERIOD: u8 = 10;

pub struct Nametag {
    name: [u8; NUM_CHARS],
    last_update: u16,
    editing: bool,
    index: usize,
    blink_counter: u8,
    blink_char: u8,
}

impl Nametag {
    pub fn new() -> Self {
        Nametag {
            name: *b"  Nick  ",
            last_update: 0,
            editing: false,
            index: 0, 
            blink_counter: 0,
            blink_char: b'_',
        }
    }
}

impl Mode for Nametag {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = context.needs_update(&mut self.last_update);

        // different behavior when editing
        if self.editing {
            self.blink_counter = (self.blink_counter + 1) % BLINK_PERIOD;
            if self.blink_counter == 0 {
                self.name[self.index] = b' ';
                self.index = (self.index + 1) % NUM_CHARS;
                self.name[self.index] = b'_';
                update = true;
            }

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
