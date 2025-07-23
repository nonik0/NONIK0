#![allow(dead_code)]

use super::ModeHandler;
use crate::{
    i2c::{Direction, Error, I2c},
    utils::*,
    Context, Event, Peripherals, SavedSettings, Setting, I2C_BUS_SPEED, NUM_CHARS,
};

const I2C_MIN_ADDRESS: u8 = 0x02;
const I2C_MAX_ADDRESS: u8 = 0x7F;
const CLIENT_ADDRESS: u8 = 0x13;

#[derive(Clone, Copy)]
pub enum I2CUtil {
    Scan,
    Receive,
    //ScrollMsg,
}

pub struct I2CUtils {
    cur_util: I2CUtil,
    util_init: bool,
    // scan data
    scan_address: u8,
    scan_direction: Direction,
    scan_error: Option<Error>,
    found_address: u8,
    // receive data
    recv_data: [u8; NUM_CHARS],
    recv_len: u8,
    display_counter: u8,
}

impl I2CUtils {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let saved_util = match settings.read_setting_byte(Setting::I2CPage) {
            0 => I2CUtil::Receive,
            _ => I2CUtil::Receive,
        };

        Self {
            cur_util: saved_util,
            util_init: false,
            // hacky way to progress to scan first address as logic increments target first
            scan_address: I2C_MIN_ADDRESS - 1,
            scan_direction: Direction::Write,
            scan_error: None,
            recv_data: [0; NUM_CHARS],
            recv_len: 0,
            found_address: 0,
            display_counter: 0,
        }
    }

    fn scan_init(&mut self, i2c: &mut I2c) {
        self.scan_address = I2C_MIN_ADDRESS - 1; // reset to first address
        self.scan_direction = Direction::Write;
        self.scan_error = None;
        self.found_address = 0;

        i2c.host_setup(I2C_BUS_SPEED);
    }

    fn scan_update(&mut self, i2c: &mut I2c) -> bool {
        // detect pause state
        if self.found_address != 0 || self.scan_error.is_some() {
            return false;
        }

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
        match i2c.host_ping_device(self.scan_address, self.scan_direction) {
            // no client ACK, continue scanning
            Ok(false) => {}
            // client ACK, stop scanning
            Ok(true) => {
                self.found_address = self.scan_address;
                self.scan_direction = Direction::Write; // don't scan addr twice if ACK
            }
            // error, stop scanning
            Err(e) => {
                self.scan_error = Some(e);
            }
        }
        true
    }

    fn format_scan_result(&self, buf: &mut [u8; NUM_CHARS]) {
        fn u4_to_hex(b: u8) -> u8 {
            match b {
                x if x < 0x0A => b'0' + x,
                x if x < 0x10 => b'A' + x,
                _ => b'?',
            }
        }
        fn addr_to_ascii(addr: u8) -> [u8; 2] {
            [u4_to_hex(addr >> 4), u4_to_hex(addr & 0x0F)]
        }
        if let Some(error) = self.scan_error {
            if self.display_counter < 0x7F {
                format_buf(buf, b"ERR:0x", &addr_to_ascii(self.scan_address));
            } else {
                format_uint(buf, b"ERR:", error as u16, 0, None);
            }
        } else if self.found_address > 0 {
            format_buf(buf, b"ACK:0x", &addr_to_ascii(self.found_address));
        } else {
            format_buf(buf, b"NAK:0x", &addr_to_ascii(self.scan_address));
        }
    }

    fn receive_init(&mut self, i2c: &mut I2c) {
        self.recv_len = 0;
        self.found_address = 0;

        i2c.client_setup(CLIENT_ADDRESS);
    }

    fn receive_update(&mut self, i2c: &mut I2c) -> bool {
        // detect pause state
        if self.recv_len > 0 {
            return false;
        }

        // read data from I2C client
        while let Some(data) = i2c.client_read() {
            if self.recv_len < NUM_CHARS as u8 {
                self.recv_data[self.recv_len as usize] = data;
                self.recv_len += 1;
            }
        }

        self.recv_len > 0
    }

    fn format_receive_result(&self, buf: &mut [u8; NUM_CHARS]) {
        if self.recv_len == 0 {
            let msg = b"RCV:0x13";
            let len = buf.len().min(msg.len());
            buf[..len].copy_from_slice(&msg[..len]);
        } else {
            for index in 0..NUM_CHARS {
                buf[index] = if index < self.recv_len as usize {
                    self.recv_data[index]
                } else {
                    b' '
                };
            }
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
        self.display_counter = self.display_counter.wrapping_add(1);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    peripherals.i2c.end();
                    self.cur_util = match self.cur_util {
                        I2CUtil::Scan => I2CUtil::Receive,
                        I2CUtil::Receive => I2CUtil::Scan,
                    };
                    context
                        .settings
                        .save_setting_byte(Setting::I2CPage, self.cur_util as u8);
                    update = true;
                    self.util_init = false;
                }
                Event::RightPressed => {
                    // clear any paused state
                    self.found_address = 0;
                    self.scan_error = None;
                    self.recv_len = 0;
                }
                _ => {}
            }
        }

        // initialize I2C utility if not already initialized
        if !self.util_init {
            match self.cur_util {
                I2CUtil::Scan => self.scan_init(&mut peripherals.i2c),
                I2CUtil::Receive => self.receive_init(&mut peripherals.i2c),
            };
            self.util_init = true;
        }

        // update utility if no other pending update
        if !update {
            update = match self.cur_util {
                I2CUtil::Scan => self.scan_update(&mut peripherals.i2c),
                I2CUtil::Receive => self.receive_update(&mut peripherals.i2c),
            };
        }

        if update {
            let mut buf = [0u8; NUM_CHARS];
            match self.cur_util {
                I2CUtil::Scan => self.format_scan_result(&mut buf),
                I2CUtil::Receive => self.format_receive_result(&mut buf),
            };

            peripherals.display.print_ascii_bytes(&buf).unwrap();
        }
    }
}
