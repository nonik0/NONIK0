use super::ModeHandler;
use crate::{Context, Event, Peripherals, SavedSettings, Setting, NUM_CHARS};

const BLINK_PERIOD_ON: u8 = 1;
const BLINK_PERIOD: u8 = 20;
const BLINK_CHAR: u8 = b'_';
const MAX_IDLE_CYCLES: u8 = 200;

pub struct Nametag {
    name: [u8; NUM_CHARS],
    last_update: u16,
    // edit tracking
    editing: bool,
    edit_index: usize,
    blink_counter: u8,
    blink_char: u8,
    idle_counter: u8,
}

impl Nametag {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let mut name_buf = [0; NUM_CHARS];
        settings.read_setting(Setting::Name, &mut name_buf);

        if name_buf.iter().any(|&byte| !byte.is_ascii_alphanumeric() && byte != b' ') {
            name_buf.copy_from_slice(b" NONIK0 ")
        }

        Nametag {
            name: name_buf,
            last_update: 0,

            editing: false,
            edit_index: 0,
            blink_counter: 0,
            blink_char: BLINK_CHAR,
            idle_counter: 0,
        }
    }

    fn next_char(&self, c: u8) -> u8 {
        match c {
            b' ' => b'A',
            b'Z' => b'a',
            b'z' => b'0',
            b'9' => b' ',
            c if c.is_ascii_alphanumeric() => c + 1,
            _ => b' ',
        }
    }

    fn prev_char(&self, c: u8) -> u8 {
        match c {
            b' ' => b'9',
            b'0' => b'z',
            b'a' => b'Z',
            b'A' => b' ',
            c if c.is_ascii_alphanumeric() => c - 1,
            _ => b' ',
        }
    }

    fn start_editing(&mut self) {
        self.editing = true;
        self.edit_index = 0;
        self.blink_counter = 0;
        self.blink_char = BLINK_CHAR;
        self.idle_counter = 0;
    }

    fn stop_editing(&mut self) {
        self.editing = false;
        self.edit_index = 0;
        self.idle_counter = 0;
    }
}

impl ModeHandler for Nametag {
    #[inline(never)]
    fn update(
        &mut self,
        event: &Option<Event>,
        context: &mut Context,
        peripherals: &mut Peripherals,
    ) {
        let mut update = context.needs_update(&mut self.last_update);

        // different behavior when editing
        if self.editing {
            self.blink_counter = (self.blink_counter + 1) % BLINK_PERIOD;
            self.idle_counter += 1;

            if self.idle_counter >= MAX_IDLE_CYCLES {
                self.stop_editing();
            }

            if self.blink_counter == 0 {
                update = true;

                if self.blink_char == BLINK_CHAR {
                    self.blink_char = self.name[self.edit_index];
                    self.name[self.edit_index] = BLINK_CHAR;
                    self.blink_counter = BLINK_PERIOD - BLINK_PERIOD_ON;
                } else {
                    self.name[self.edit_index] = self.blink_char;
                    self.blink_char = BLINK_CHAR;
                }
            }

            // hold left or right to move cursor, move off side to stop editing
            // tap left or right to change character at cursor (TODO: hold for faster scrolling)
            if let Some(event) = event {
                self.idle_counter = 0;

                match event {
                    Event::LeftHeld => {
                        update = true;
                        if self.edit_index <= 0 {
                            self.stop_editing();
                        } else {
                            self.edit_index = self.edit_index - 1;
                        }
                    }
                    Event::RightHeld => {
                        update = true;
                        self.edit_index = self.edit_index + 1;
                        if self.edit_index >= NUM_CHARS {
                            self.stop_editing();
                        }
                    }
                    Event::LeftReleased => {
                        update = true;
                        self.name[self.edit_index] = self.prev_char(self.name[self.edit_index]);
                    }
                    Event::RightReleased => {
                        update = true;
                        self.name[self.edit_index] = self.next_char(self.name[self.edit_index]);
                    }
                    _ => {}
                }
            }
        } else {
            // not editing
            if let Some(event) = event {
                match event {
                    Event::LeftHeld => {
                        // save name if eeprom if updated
                        let mut saved_named = [0; NUM_CHARS];
                        context.settings.read_setting(Setting::Name, &mut saved_named);
                        if self.name != saved_named {
                            context.settings.save_setting(Setting::Name, &self.name);
                        }
                        context.to_menu();
                        return;
                    }
                    Event::RightHeld => {
                        self.start_editing();
                        update = true;
                    }
                    _ => {}
                }
            }
        }

        if update {
            peripherals.display.print_ascii_bytes(&self.name).unwrap();
        }
    }
}
