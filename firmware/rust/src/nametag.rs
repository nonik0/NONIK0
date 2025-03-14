use crate::{Context, Display, Event, Mode, NUM_CHARS};

pub struct Nametag {
    name: [u8; NUM_CHARS],
    last_update: u16,
}

impl Nametag {
    pub fn new() -> Self {
        Nametag {
            name: *b"  Nick  ",
            last_update: 0,
        }
    }
}

impl Mode for Nametag {
    fn update(&mut self, _: &Option<Event>, display: &mut Display, context: &mut Context) {
        if self.last_update < context.mode_counter {
            self.last_update = context.mode_counter;

            display.print_ascii_bytes(&self.name).unwrap();
        }
    }
}
