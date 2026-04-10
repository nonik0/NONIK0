use super::ModeHandler;
use crate::{Context, Event, Peripherals, SavedSettings, Setting, NUM_CHARS};

const BLINK_PERIOD_ON: u8 = 1;
const BLINK_PERIOD: u8 = 20;
const BLINK_CHAR: u8 = b'_';
const MAX_IDLE_CYCLES: u8 = 200;

pub struct Nametag {
    name: [u8; NUM_CHARS],
    edit_index: Option<u8>, // None = not editing, Some(edit_index) = editing
    blink_counter: u8,
    idle_counter: u8,
    settings_dirty: bool,
}

impl Nametag {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let mut name_buf = [0; NUM_CHARS];
        settings.read_setting(Setting::Name, &mut name_buf);

        if name_buf
            .iter()
            .any(|&byte| !byte.is_ascii_alphanumeric() && byte != b' ')
        {
            name_buf.copy_from_slice(b" NONIK0 ")
        }

        Nametag {
            name: name_buf,
            edit_index: None,
            blink_counter: 0,
            idle_counter: 0,
            settings_dirty: false,
        }
    }

    fn adjust_char(&self, c: u8, forward: bool) -> u8 {
        match (c, forward) {
            (b' ', true)  => b'A',  (b' ', false) => b'9',
            (b'Z', true)  => b'a',  (b'a', false) => b'Z',
            (b'z', true)  => b'0',  (b'A', false) => b' ',
            (b'9', true)  => b' ',  (b'0', false) => b'z',
            (c, true)  if c.is_ascii_alphanumeric() => c + 1,
            (c, false) if c.is_ascii_alphanumeric() => c - 1,
            _ => b' ',
        }
    }

    fn next_char(&self, c: u8) -> u8 {
        self.adjust_char(c, true)
    }

    fn prev_char(&self, c: u8) -> u8 {
        self.adjust_char(c, false)
    }

    fn start_editing(&mut self) {
        self.edit_index = Some(0);
        self.blink_counter = 0;
        self.idle_counter = 0;
    }

    fn stop_editing(&mut self) {
        self.edit_index = None;
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
        let mut update = context.need_update();

        if let Some(edit_index) = self.edit_index {
            let edit_index = edit_index as usize;
            self.blink_counter = (self.blink_counter + 1) % BLINK_PERIOD;
            self.idle_counter += 1;
            if self.idle_counter >= MAX_IDLE_CYCLES {
                self.stop_editing();
            }

            if let Some(event) = event {
                self.idle_counter = 0;
                match event {
                    Event::LeftReleased => {
                        self.name[edit_index] = self.prev_char(self.name[edit_index]);
                        self.settings_dirty = true;
                        update = true;
                    }
                    Event::RightReleased => {
                        self.name[edit_index] = self.next_char(self.name[edit_index]);
                        self.settings_dirty = true;
                        update = true;
                    }
                    Event::LeftHeld => {
                        if edit_index == 0 {
                            self.stop_editing();
                        } else {
                            self.edit_index = Some((edit_index - 1) as u8);
                        }
                    }
                    Event::RightHeld => {
                        if edit_index + 1 >= NUM_CHARS {
                            self.stop_editing();
                        } else {
                            self.edit_index = Some((edit_index + 1) as u8);
                        }
                    }
                    _ => {}
                }
            }

            // Only update display on change or blink
            if update || self.blink_counter == 0 || self.blink_counter == BLINK_PERIOD_ON {
                let mut buf = self.name;
                if self.blink_counter < BLINK_PERIOD_ON {
                    buf[edit_index] = BLINK_CHAR;
                }
                peripherals.display.print_ascii_bytes(&buf).unwrap();
                return;
            }
        } else if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    if self.settings_dirty {
                        context.settings.save_setting(Setting::Name, &self.name);
                    }
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    self.start_editing();
                }
                _ => {}
            }
        }

        if update {
            peripherals.display.print_ascii_bytes(&self.name).unwrap();
        }
    }
}