#![allow(dead_code)]

use super::ModeHandler;
use crate::{
    i2c::{Direction, Error, I2c},
    utils::*,
    Context, Event, Peripherals, SavedSettings, NUM_CHARS,
};

const I2C_MIN_ADDRESS: u8 = 0x02;
const I2C_MAX_ADDRESS: u8 = 0x77;

pub struct I2CUtils {
    scan_address: u8,
    scan_direction: Direction,
    scan_error: Option<Error>,
    found_address: u8,
}

impl I2CUtils {
    pub fn new_with_settings(_settings: &SavedSettings) -> Self {
        Self {
            // hacky way to progress to scan first address as logic increments target first
            scan_address: I2C_MIN_ADDRESS - 1,
            scan_direction: Direction::Write,
            scan_error: None,
            found_address: 0,
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
        let mut update = context.need_update();

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                Event::RightPressed => {
                    // clear paused state
                    self.found_address = 0;
                    self.scan_error = None;
                }
                _ => {}
            }
        }

        // if update not pending and not paused
        if !update && self.found_address == 0 && self.scan_error.is_none() {
            // proceed to next scan target
            if self.scan_direction == Direction::Read {
                self.scan_direction = Direction::Write;
            } else {
                self.scan_direction = Direction::Read;
                self.scan_address += 1;
                if self.scan_address > I2C_MAX_ADDRESS {
                    self.scan_address = I2C_MIN_ADDRESS;
                }
            }

            // ping device at address
            match peripherals
                .i2c
                .ping_device(self.scan_address, self.scan_direction)
            {
                // no client ACK, continue scanning
                Ok(false) => {}
                // client ACK, stop scanning
                Ok(true) => {
                    self.found_address = self.scan_address;
                }
                // error, stop scanning
                Err(e) => {
                    self.scan_error = Some(e);
                }
            }

            update = true;
        }

        if update {
            fn u4_to_hex(b: u8) -> u8 {
                match b {
                    x if x < 0xa => 0x30 + x,
                    x if x < 0x10 => 0x57 + x,
                    _ => b'?',
                }
            }
            fn addr_to_ascii(addr: u8) -> [u8; 2] {
                [
                    u4_to_hex(addr >> 4),
                    u4_to_hex(addr & 0x0F),
                ]
            }

            let mut buf = [0u8; NUM_CHARS];
            if let Some(error) = self.scan_error {
                format_uint(&mut buf, b" ERR:", error as u16, 0, None);
            } else if self.found_address > 0 {
                format_buf(&mut buf, b" ACK:", &addr_to_ascii(self.found_address));
            } else {
                format_buf(&mut buf, b"SCAN:", &addr_to_ascii(self.scan_address));
            }
            peripherals.display.print_ascii_bytes(&buf).unwrap();
        }
    }
}
