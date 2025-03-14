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
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftPressed | Event::RightPressed => {
                    update = true;
                }
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                _ => {}
            }
        }

        if update {
            display.print_u32(crate::Rand::default().get_u32()).unwrap();
        }
    }
}
