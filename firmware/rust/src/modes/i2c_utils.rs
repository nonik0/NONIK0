use super::ModeHandler;
use crate::{Context, Event, Peripherals, SavedSettings};

pub struct I2CUtils {
}

impl I2CUtils {
    pub fn new_with_settings(_settings: &SavedSettings) -> Self {
        Self {
        }
    }
}

impl ModeHandler for I2CUtils {
    #[inline(never)]
    fn update(
        &mut self,
        event: &Option<Event>,
        context: &mut Context,
        peripherals: &mut Peripherals,
    ) {
        let update = context.need_update();

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                _ => {}
            }
        }

        let _ = &mut peripherals.i2c;

        if update {
            let buf = b"  TODO  ";
            peripherals.display.print_ascii_bytes(buf).unwrap();
        }
    }
}
