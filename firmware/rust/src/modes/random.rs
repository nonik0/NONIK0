use random_trait::Random as _;

use super::Mode;
use crate::{Context, Display, Event};

pub struct Random {
    last_update: u16,
}

impl Random {
    pub fn new() -> Self {
        Random { last_update: 0 }
    }
}

impl Mode for Random {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = false;

        if self.last_update < context.menu_counter {
            self.last_update = context.menu_counter;
            update = true;
        }

        if let Some(event) = event {
            match event {
                Event::LeftPressed | Event::RightPressed => {
                    update = true;
                }
                Event::LeftHeld => {
                    context.menu_counter += 1;
                    context.mode_index = 0;
                    return;
                }
                _ => {}
            }
        }

        if update {
            display.print_u32(context.rand.get_u32()).unwrap();
        }
    }
}
