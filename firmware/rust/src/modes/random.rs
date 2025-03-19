use super::Mode;
use crate::{Context, Display, Event, Rand, NUM_CHARS};
use random_trait::Random as _;

pub struct Random {
    last_update: u16,
}

impl Random {
    pub fn new() -> Self {
        Random { last_update: 0 }
    }
}

impl Mode for Random {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
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
            //display.print_u32(crate::Rand::default().get_u32()).unwrap();
            //const MAX_VALUE: u32 = (10u32).pow(NUM_CHARS as u32) - 1;
            //let rand_value = Rand::default().get_u32() % MAX_VALUE;
            let mut rand_value = Rand::default().get_u32();
            let mut buf = [b' '; NUM_CHARS];
            for index in (0..NUM_CHARS).rev() {
                buf[index] = b'0' + (rand_value % 10) as u8;
                rand_value /= 10;
            }

            display.print_ascii_bytes(&buf).unwrap()
        }
    }
}
