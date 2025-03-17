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
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let update = context.needs_update(&mut self.last_update);

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
