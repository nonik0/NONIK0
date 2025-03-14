use random_trait::Random as _;

use super::Mode;
use crate::{Context, Display, Event};

pub struct Settings {
    last_update: u16,
}

impl Settings {
    pub fn new() -> Self {
        Settings { last_update: 0 }
    }
}

impl Mode for Settings {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                _ => {}
            }
        }

        // if self.last_update < context.menu_counter {
        //     self.last_update = context.menu_counter;
        //     update = true;
        // }

        if update {
            display.print_ascii_bytes(b" TO IMPL").unwrap();
        }
    }
}
