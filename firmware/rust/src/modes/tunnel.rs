use super::Mode;
use crate::{Context, Display, Event, Rand, COLUMN_GAP, NUM_ROWS, NUM_VIRT_COLS};
use heapless::Vec;

pub struct Tunnel {
    last_update: u16,
    display_buf: Vec<u8, NUM_VIRT_COLS>,

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
            last_update: 0,
            display_buf: buf,
            runner_pos: 0,
            tunnel_state: TunnelState::new(),
        }
    }
}

impl Mode for Tunnel {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {}
                Event::LeftReleased => {
                    if self.runner_pos < NUM_ROWS as u8 - 1 {
                        self.runner_pos += 1;
                    }
                }
                Event::RightReleased => {
                    if self.runner_pos > 0 {
                        self.runner_pos -= 1;
                    }
                }
                _ => {}
            }
        }

        if let Some(new_tunnel_col) = self.tunnel_state.next_tunnel_col() {
            update = true;

            self.display_buf.remove(0);
            self.display_buf.push(new_tunnel_col).unwrap();
        }

        if update {
            let mut cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
            for i in 0..NUM_VIRT_COLS {
                // if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                //     let cloud_col = self.cloud_cols.get(i).copied().unwrap_or(0);
                //     let earth_col = self.earth_cols.get(i).copied().unwrap_or(0);
                //     cols.push(cloud_col & earth_col).unwrap();
                // }
                cols.push(self.display_buf.get(i).copied().unwrap_or(0))
                    .unwrap();
            }

            display.print_cols(cols.as_slice()).unwrap();
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
            period: 10,
            pos: 1,
            cur_width: NUM_ROWS as u8 - 2,
            min_width: NUM_ROWS as u8 - 2,
            max_width: NUM_ROWS as u8 - 1,
        }
    }

    fn next_tunnel_col(&mut self) -> Option<u8> {
        self.counter += 1;
        if self.counter >= self.period {
            self.counter = 0;

            None
        } else {
            None
        }
    }
}
