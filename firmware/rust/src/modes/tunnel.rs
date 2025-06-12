use super::ModeHandler;
use crate::{Context, Event, Peripherals, Rand, COLUMN_GAP, NUM_ROWS, NUM_VIRT_COLS};
use heapless::Vec;
use random_trait::Random;

const TUNNEL_PERIOD: u8 = 5;

pub struct Tunnel {
    tunnel_cols: Vec<u8, NUM_VIRT_COLS>,

    is_running: bool,
    runner_pos: u8,
    tunnel_state: TunnelState,
}

impl Tunnel {
    pub fn new() -> Self {
        let mut buf: Vec<u8, NUM_VIRT_COLS> = Vec::new();
        for _ in 0..NUM_VIRT_COLS {
            buf.push(0).unwrap();
        }

        Tunnel {
            tunnel_cols: buf,
            is_running: true,
            runner_pos: NUM_ROWS as u8 / 2,
            tunnel_state: TunnelState::new(),
        }
    }

    fn get_runner_col(&self) -> u8 {
        0b00000001 << (NUM_ROWS as u8 - self.runner_pos - 1)
    }
}

impl ModeHandler for Tunnel {
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
                Event::RightHeld => {
                    // restart if over
                    if !self.is_running {
                        update = true;

                        self.tunnel_cols.clear();
                        for _ in 0..NUM_VIRT_COLS {
                            self.tunnel_cols.push(0).unwrap();
                        }
                        self.tunnel_state = TunnelState::new();

                        self.is_running = true;
                    }
                }
                Event::LeftReleased => {
                    update = true;
                    if self.runner_pos < NUM_ROWS as u8 - 1 {
                        self.runner_pos += 1;
                    }
                }
                Event::RightReleased => {
                    update = true;
                    if self.runner_pos > 0 {
                        self.runner_pos -= 1;
                    }
                }
                _ => {}
            }
        }

        if self.is_running {
            if let Some(new_tunnel_col) = self.tunnel_state.next_tunnel_col() {
                update = true;

                self.tunnel_cols.remove(0);
                self.tunnel_cols.push(new_tunnel_col).unwrap();
            }
        }

        if update {
            let mut cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
            for i in 0..NUM_VIRT_COLS {
                if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                    let mut col = self.tunnel_cols.get(i).copied().unwrap_or(0);

                    if i == 2 as usize || i == 3 as usize {
                        let collision = col & self.get_runner_col() != 0;
                        if collision {
                            self.is_running = false;
                        }

                        col |= self.get_runner_col();
                    }

                    cols.push(col).unwrap();
                }
            }

            peripherals.display.print_cols(cols.as_slice()).unwrap();
        }
    }
}

struct TunnelState {
    counter: u8,
    period: u8,
    pos: u8,
    cur_width: u8,
    min_width: u8,
    max_width: u8,
}

impl TunnelState {
    fn new() -> Self {
        TunnelState {
            counter: 0,
            period: TUNNEL_PERIOD,
            pos: 1,
            cur_width: NUM_ROWS as u8 - 2,
            min_width: NUM_ROWS as u8 - 2,
            max_width: NUM_ROWS as u8 - 1,
        }
    }

    fn next_tunnel_col(&mut self) -> Option<u8> {
        self.counter = (self.counter + 1) % self.period;
        if self.counter != 0 {
            return None;
        }

        // shift/expand tunnel
        let mut rand = Rand::default();
        let will_shift = rand.get_u8() % 2 == 0;
        if will_shift {
            let shift_up = rand.get_u8() % 2 == 0;
            if shift_up && self.pos + self.cur_width < NUM_ROWS as u8 - 1 {
                self.pos += 1;
            } else if !shift_up && self.pos > 0 {
                self.pos -= 1;
            }
        }

        let will_change_size = !will_shift && rand.get_u8() % 2 == 0;
        if will_change_size {
            let shrink = rand.get_u8() % 2 == 0;
            let shift = rand.get_u8() % 2 == 0;
            if shrink && self.cur_width > self.min_width && self.pos + self.cur_width > 2 {
                if shift {
                    self.pos += 1;
                }
                self.cur_width -= 1;
            } else if !shrink
                && self.cur_width < self.max_width
                && self.pos + self.cur_width < NUM_ROWS as u8 - 1
            {
                if shift && self.pos > 0 {
                    self.pos -= 1;
                }
                self.cur_width += 1;
            }
        }

        let difficulty_increase = rand.get_u8() % 100 == 0;
        if difficulty_increase {
            let min_size_decrease = rand.get_u8() % 3 != 0; // 2/3 chance to decrease min size
            if min_size_decrease && self.min_width > 1 {
                self.min_width -= 1;
            } else if self.max_width > self.min_width {
                self.min_width -= 1;
            }
        }

        let period_decrease = !difficulty_increase && rand.get_u8() % 200 == 0;
        if period_decrease {
            if self.period > 1 {
                self.period -= 1;
            }
        }

        // generate next col
        let mut col: u8 = 0;
        for i in 0..NUM_ROWS {
            let bit = if (i as u8) >= self.pos && (i as u8) < self.pos + self.cur_width {
                0
            } else {
                1
            };
            col = col << 1 | bit;
        }
        Some(col)
    }
}
