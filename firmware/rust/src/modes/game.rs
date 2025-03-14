use crate::{Context, Display, Event};
use super::Mode;

pub struct Game {
    last_update: u16,
}

impl Game {
    pub fn new() -> Self {
        Game { last_update: 0 }
    }
}

impl Mode for Game {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = false;

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.menu_counter += 1;
                    context.mode_index = 0;
                    return;
                }
                _ => {}
            }
        }

        if self.last_update < context.menu_counter {
            self.last_update = context.menu_counter;
            update = true;
        }

        if update {
            display.print_ascii_bytes(b"TO IMPL").unwrap();
        }
    }
}
