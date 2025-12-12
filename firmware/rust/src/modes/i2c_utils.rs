#![allow(dead_code)]

use super::ModeHandler;
use crate::{
    i2c::{Direction, Error, I2c, I2C_BUS_SPEED, I2C_BUFFER_SIZE},
    utils::*,
    Context, Event, Peripherals, SavedSettings, Setting, NUM_CHARS,
};

const I2C_CLIENT_ADDRESS: u8 = 0x13;
const I2C_MIN_ADDRESS: u8 = 0x02;
const I2C_MAX_ADDRESS: u8 = 0x7F;
const MAX_MESSAGE_SIZE: usize = I2C_BUFFER_SIZE;

#[derive(Clone, Copy)]
pub enum I2CUtil {
    ScannerHost,
    MessageClient,
}

pub struct I2CUtils {
    cur_util: I2CUtil,
    util_init: bool,
    // scan data
    scan_found_address: u8,
    scan_address: u8,
    scan_direction: Direction,
    scan_error: Option<Error>,
    // message data
    //msg_buf: [u8; MAX_MESSAGE_SIZE],
    msg_data: [u8; MAX_MESSAGE_SIZE],
    msg_display: bool,
    msg_len: u8,
    //msg_buf_pos: u8,
    msg_scroll_pos: u8,
    msg_speed: u8,
    // for display timing
    counter: u8,
}

impl I2CUtils {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let saved_util = match settings.read_setting_byte(Setting::I2CPage) {
            0 => I2CUtil::ScannerHost,
            _ => I2CUtil::MessageClient,
        };

        let mut msg_data = [0u8; MAX_MESSAGE_SIZE];
        let msg = b"Waiting for message...";
        let len = msg_data.len().min(msg.len());
        msg_data[..len].copy_from_slice(&msg[..len]);

        Self {
            cur_util: saved_util,
            util_init: false,
            // hacky way to progress to scan first address as logic increments target first
            scan_found_address: 0,
            scan_address: I2C_MIN_ADDRESS - 1,
            scan_direction: Direction::Write,
            scan_error: None,
            //msg_buf: [0; MAX_MESSAGE_SIZE],
            msg_data,
            msg_display: true,
            msg_len: len as u8,
            //msg_buf_pos: 0,
            msg_scroll_pos: 0,
            msg_speed: 93,
            counter: 0,
        }
    }

    fn scan_init(&mut self, i2c: &mut I2c) {
        self.scan_address = I2C_MIN_ADDRESS - 1; // reset to first address
        self.scan_direction = Direction::Write;
        self.scan_error = None;
        self.scan_found_address = 0;

        i2c.host_setup(I2C_BUS_SPEED);
    }

    fn scan_update(&mut self, i2c: &mut I2c) -> bool {
        // detect pause state
        if self.scan_found_address != 0 || self.scan_error.is_some() {
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
                self.scan_found_address = self.scan_address;
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
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        fn addr_to_ascii(addr: u8) -> [u8; 2] {
            [HEX[(addr >> 4) as usize], HEX[(addr & 0x0F) as usize]]
        }
        if let Some(error) = self.scan_error {
            if self.counter < 0x7F {
                format_buf(buf, b"ERR:0x", &addr_to_ascii(self.scan_address));
            } else {
                format_uint(buf, b"ERR:", error as u16, 0, None);
            }
        } else if self.scan_found_address > 0 {
            format_buf(buf, b"ACK:0x", &addr_to_ascii(self.scan_found_address));
        } else {
            format_buf(buf, b"NAK:0x", &addr_to_ascii(self.scan_address));
        }
    }

    fn scroll_msg_init(&mut self, i2c: &mut I2c) {
        self.msg_scroll_pos = 0;
        //self.msg_buf_pos = 0;
        i2c.client_setup(I2C_CLIENT_ADDRESS);
    }

    fn scroll_msg_update(&mut self, i2c: &mut I2c) -> bool {
        let mut update = false;

        if i2c.client_available() > 0 {
            let command = i2c.client_read().unwrap();

            // setDisplay
            if command == 0x00 {
                self.msg_display = i2c.client_read().unwrap_or(self.msg_display as u8) != 0;
                update = true;
            }
            // setMessage
            else if command == 0x01 {
                // read all received data into buffer, discard extra bytes if filled
                self.msg_len = 0;
                while let Some(data) = i2c.client_read() {
                    if self.msg_len < MAX_MESSAGE_SIZE as u8 {
                        self.msg_data[self.msg_len as usize] = data;
                        self.msg_len += 1;
                    }
                }

                // remove null terminator if present
                if self.msg_data[self.msg_len as usize - 1] == b'\0' {
                    self.msg_len -= 1;
                }

                self.msg_scroll_pos = 0;
                update = true;

                // TODO: debug dropped chunks with consecutive writes from host
                // while let Some(data) = i2c.client_read() {
                //     if self.msg_buf_pos < MAX_MESSAGE_SIZE as u8 {
                //         self.msg_buf[self.msg_buf_pos as usize] = data;
                //         self.msg_buf_pos += 1;
                //     }
                // }

                // // if buffer is full, add null terminator
                // if self.msg_buf_pos >= MAX_MESSAGE_SIZE as u8 {
                //     self.msg_buf[self.msg_buf_pos as usize - 1] = b'\0';
                // }

                // // last chunk if null terminator seen
                // if self.msg_buf_pos > 0 && self.msg_buf[self.msg_buf_pos as usize - 1] == b'\0' {
                //     // copy data without null terminator
                //     self.msg_len = self.msg_buf_pos - 1; 
                //     self.msg_data[..self.msg_len as usize]
                //         .copy_from_slice(&self.msg_buf[..self.msg_len as usize]);

                //     // reset positions
                //     self.msg_buf_pos = 0;
                //     self.msg_scroll_pos = 0;
                //     update = true;
                // }
            }
            // setScrollSpeed
            else if command == 0x02 {
                self.msg_speed = i2c.client_read().unwrap_or(self.msg_speed);
                if self.msg_speed > 100 {
                    self.msg_speed = 100;
                }
            }

            // flush any extra data
            while i2c.client_read().is_some() {}
        }

        if self.counter > 100 - self.msg_speed {
            self.counter = 0;

            let msg_full_len = (self.msg_len as usize + NUM_CHARS) as u8; // NUM_CHARS blank padding
            self.msg_scroll_pos = (self.msg_scroll_pos + 1) % msg_full_len;
            update = true;
        }

        update
    }

    fn format_scroll_msg(&self, buf: &mut [u8; NUM_CHARS]) {
        // adds padding spaces before and after message for scrolling effect
        for (display_index, buf_char) in buf.iter_mut().enumerate().take(NUM_CHARS) {
            let offset_index = self.msg_scroll_pos as usize + display_index;
            *buf_char = if self.msg_display && offset_index >= NUM_CHARS {
                let actual_index = offset_index - NUM_CHARS;
                if actual_index < self.msg_len as usize {
                    self.msg_data[actual_index]
                } else {
                    b' '
                }
            } else {
                b' '
            };
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
        self.counter = self.counter.wrapping_add(1);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    // disable I2C host or client module before leaving mode
                    match self.cur_util {
                        I2CUtil::ScannerHost => peripherals.i2c.host_end(),
                        I2CUtil::MessageClient => peripherals.i2c.client_end(),
                    };
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    // disable I2C host or client module before switching current util
                    self.cur_util = match self.cur_util {
                        I2CUtil::ScannerHost => {
                            peripherals.i2c.host_end();
                            I2CUtil::MessageClient
                        }
                        I2CUtil::MessageClient => {
                            peripherals.i2c.client_end();
                            I2CUtil::ScannerHost
                        }
                    };
                    context
                        .settings
                        .save_setting_byte(Setting::I2CPage, self.cur_util as u8);
                    update = true;
                    self.util_init = false;
                }
                Event::RightPressed => {
                    // clear any paused state
                    self.scan_found_address = 0;
                    self.scan_error = None;
                }
                _ => {}
            }
        }

        // initialize I2C utility if not already initialized
        if !self.util_init {
            match self.cur_util {
                I2CUtil::ScannerHost => self.scan_init(&mut peripherals.i2c),
                I2CUtil::MessageClient => self.scroll_msg_init(&mut peripherals.i2c),
            };
            self.util_init = true;
        }

        // update utility if no other pending update
        if !update {
            update = match self.cur_util {
                I2CUtil::ScannerHost => self.scan_update(&mut peripherals.i2c),
                I2CUtil::MessageClient => self.scroll_msg_update(&mut peripherals.i2c),
            };
        }

        if update {
            let mut buf = [0u8; NUM_CHARS];
            match self.cur_util {
                I2CUtil::ScannerHost => self.format_scan_result(&mut buf),
                I2CUtil::MessageClient => self.format_scroll_msg(&mut buf),
            };

            peripherals.display.print_ascii_bytes(&buf).unwrap();
        }
    }
}