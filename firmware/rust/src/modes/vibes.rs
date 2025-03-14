// parallax animation of "driving" during through mountains and clouds

use crate::{Context, Display, Event, Rand, COLUMN_GAP, NUM_ROWS, NUM_VIRT_COLS};
use super::Mode;
use heapless::Vec;
use random_trait::Random; // Import the correct module based on feature flag

const SKY_PERIOD: u8 = 7;
const EARTH_PERIOD: u8 = 3;

// col bits: msb+1 is bottom row, lsb is top row, i.e. 0b0111_1111 is all on
const SKY_COL: u8 = 0b0111_1111; // silhouetted mountain and clouds so sky pixels are all on

struct CloudState {
    loc: u8,
    gap: u8,
    cur_length: u8,
    height: u8,
    length: u8,
}

impl CloudState {
    fn new() -> Self {
        CloudState {
            loc: 0,
            gap: 1,
            cur_length: 0,
            length: 0,
            height: 0,
        }
    }

    fn next_cloud(&mut self) {
        let mut rng = Rand::default();
        self.gap = rng.get_u8() % 10 + 1;
        self.loc = 1 + rng.get_u8() % (NUM_ROWS as u8 - 2);
        self.height = 2 + rng.get_u8() % 2;
        self.length = 6 + rng.get_u8() % 10;
        if self.height == 3 && self.loc > 4 {
            self.loc -= 4;
        }
    }

    fn next_col(&mut self) -> u8 {
        let mut col = SKY_COL;

        if self.gap > 0 {
            self.gap -= 1;
        } else if self.cur_length < self.length {
            for i in 0..NUM_ROWS {
                let bit = if (i as u8) >= self.loc && (i as u8) < self.loc + self.height {
                    0
                } else {
                    1
                };
                col = col << 1 | bit;
            }
            self.cur_length += 1;
        } else {
            self.cur_length = 0;
            self.next_cloud();
        }

        col
    }
}

struct MountainState {
    cur_height: u8,
    cur_length: u8,
    height: u8,
    length: u8,
    increment: i8,
}

impl MountainState {
    fn new() -> Self {
        MountainState {
            cur_height: 0,
            cur_length: 0,
            height: 7,
            length: 15,
            increment: 1,
        }
    }

    fn next_mountain(&mut self) {
        let mut rng = Rand::default();
        //self.cur_height remains the same
        self.cur_length = 0;
        self.height = 4 + rng.get_u8() % 3; // height range: [4, 6]
        self.length = self.height
            + 1
            + (rng.get_u8() % (self.height + 1) + rng.get_u8() % (self.height + 1)) / 2; // length range: [height + 1, 2*height+2], 2 samples to give normal-er distribution
        self.increment = 1;
    }

    fn next_col(&mut self) -> u8 {
        self.cur_height = (self.cur_height as i8 + self.increment) as u8;
        self.cur_length += 1;

        // Shift in 0s from bottom/left to build to current height
        let mut col = SKY_COL;
        for _ in 0..self.cur_height {
            col = col >> 1;
        }

        // Start going down
        if self.increment > 0 && self.cur_height >= self.height {
            self.increment = -1;
        }

        // Stop going down
        if self.increment < 0 && self.cur_height == 0 {
            self.increment = 0;
        }

        // Start new mountain
        if self.increment <= 0 && self.cur_length == self.length {
            self.next_mountain();
        }

        col
    }
}

pub struct Vibes {
    last_update: u16,
    cloud_cols: Vec<u8, NUM_VIRT_COLS>,
    earth_cols: Vec<u8, NUM_VIRT_COLS>,
    cloud_counter: u8,
    earth_counter: u8,
    cloud_state: CloudState,
    earth_state: MountainState,
}

impl Vibes {
    pub fn new() -> Self {
        let mut cloud_cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
        let mut earth_cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
        let cloud_counter: u8 = 0;
        let earth_counter: u8 = 0;

        let cloud_state = CloudState::new();
        let earth_state = MountainState::new();

        // fill display buffer with initial columns
        for _ in 0..NUM_VIRT_COLS {
            cloud_cols.push(SKY_COL).unwrap();
        }
        for _ in 0..NUM_VIRT_COLS {
            earth_cols.push(SKY_COL).unwrap();
        }

        Vibes {
            last_update: 0,
            cloud_cols,
            earth_cols,
            cloud_counter,
            earth_counter,
            cloud_state,
            earth_state,
        }
    }
}

impl Mode for Vibes {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                _ => {}
            }
        }

        self.cloud_counter = (self.cloud_counter + 1) % SKY_PERIOD;
        if self.cloud_counter == 0 {
            let new_cloud_col = self.cloud_state.next_col();
            self.cloud_cols.remove(0);
            self.cloud_cols.push(new_cloud_col).unwrap();

            update = true;
        }

        self.earth_counter = (self.earth_counter + 1) % EARTH_PERIOD;
        if self.earth_counter == 0 {
            let new_earth_col = self.earth_state.next_col();
            self.earth_cols.remove(0);
            self.earth_cols.push(new_earth_col).unwrap();

            update = true;
        }

        if update {
            let mut cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
            for i in 0..NUM_VIRT_COLS {
                if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                    let cloud_col = self.cloud_cols.get(i).copied().unwrap_or(0);
                    let earth_col = self.earth_cols.get(i).copied().unwrap_or(0);
                    cols.push(cloud_col & earth_col).unwrap();
                }
            }

            display.print_cols(cols.as_slice()).unwrap();
        }
    }
}