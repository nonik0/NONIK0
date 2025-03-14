use crate::{Context, Display, Event, Mode};

pub struct Game {
    last_update: u16,
}

impl Game {
    pub fn new() -> Self {
        Game { last_update: 0 }
    }
}

impl Mode for Game {
    fn update(&mut self, _: &Option<Event>, display: &mut Display, context: &mut Context) {
        if self.last_update < context.mode_counter {
            self.last_update = context.mode_counter;

            display.print_ascii_bytes(b"TO IMPL").unwrap();
        }
    }
}
