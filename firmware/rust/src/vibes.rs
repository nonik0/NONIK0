// parallax animation of "driving" during through mountains and clouds

use crate::{Display, random::Rand, COLUMN_GAP, NUM_COLS, NUM_ROWS, NUM_VIRT_COLS};
use random_trait::Random;

const DEFAULT_SKY_PERIOD: u8 = 10;
const DEFAULT_EARTH_PERIOD: u8 = 5;
const NUM_VIRT_COLS2: usize = NUM_VIRT_COLS / 2;

// col bits: msb+1 is bottom row, lsb is top row, i.e. 0b0111_1111 is all on
const SKY_COL: u8 = 0b0111_1111; // silhouetted mountain and clouds so sky pixels are all on

pub struct Vibes {
    cloud_cols: [u8; NUM_VIRT_COLS2],
    cloud_counter: u8,
    cloud_period: u8,
    cloud_state: CloudState,

    earth_cols: [u8; NUM_VIRT_COLS2],
    earth_counter: u8,
    earth_period: u8,
    earth_state: EarthState,

    cloud_index: usize,
    earth_index: usize,
}

impl Vibes {
    pub fn new() -> Self {
        Vibes {
            cloud_cols: [SKY_COL; NUM_VIRT_COLS2],
            cloud_counter: 0,
            cloud_period: DEFAULT_SKY_PERIOD,
            cloud_state: CloudState::new(),

            earth_cols: [SKY_COL; NUM_VIRT_COLS2],
            earth_counter: 0,
            earth_period: DEFAULT_EARTH_PERIOD,
            earth_state: EarthState::new(),

            cloud_index: 0,
            earth_index: 0,
        }
    }

    fn next_index(&self, index: usize) -> usize {
        (index + 1) % NUM_VIRT_COLS2
    }

    fn render(&mut self, display: &mut Display) {
        let mut cols = [0u8; NUM_COLS];
        let mut col_num = 0;

        for virt_col_num in 0..NUM_VIRT_COLS2 {
            if virt_col_num % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                cols[col_num] = self.cloud_cols[(self.cloud_index + virt_col_num) % NUM_VIRT_COLS2];
                col_num += 1;
            }
        }

        for virt_col_num in 0..NUM_VIRT_COLS2 {
            if virt_col_num % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                cols[col_num] = self.earth_cols[(self.earth_index + virt_col_num) % NUM_VIRT_COLS2];
                col_num += 1;
            }
        }
        display.print_cols(&cols).unwrap();
    }

    pub fn update(&mut self, display: &mut Display) {
        let mut update = false;

        self.cloud_counter = (self.cloud_counter + 1) % self.cloud_period;
        if self.cloud_counter == 0 {
            update = true;
            self.cloud_cols[self.cloud_index] = self.cloud_state.next_col();
            self.cloud_index = self.next_index(self.cloud_index);
        }

        self.earth_counter = (self.earth_counter + 1) % self.earth_period;
        if self.earth_counter == 0 {
            update = true;
            self.earth_cols[self.earth_index] = self.earth_state.next_col();
            self.earth_index = self.next_index(self.earth_index);
        }

        if update {
            self.render(display);
        }
    }
}

#[derive(Default)]
struct CloudState {
    loc: u8,
    gap: u8,
    cur_length: u8,
    height: u8,
    length: u8,
}

impl CloudState {
    fn new() -> Self {
        let mut state = CloudState::default();
        state.next_cloud();
        state
    }

    fn next_cloud(&mut self) {
        let mut rng = Rand::default();
        self.gap = rng.get_u8() % 10 + 1;
        self.loc = 1 + rng.get_u8() % (NUM_ROWS as u8 - 3);
        self.height = 2 + rng.get_u8() % 2;
        self.length = 6 + rng.get_u8() % 10;
        if self.height == 3 && self.loc > 4 {
            self.loc -= 4;
        }
    }

    fn next_col(&mut self) -> u8 {
        if self.gap > 0 {
            self.gap -= 1;
            SKY_COL
        } else if self.cur_length < self.length {
            self.cur_length += 1;
            SKY_COL ^ (((1 << self.height) - 1) << self.loc) // check inversion
        } else {
            self.cur_length = 0;
            self.next_cloud();
            SKY_COL
        }
    }
}

#[derive(Default)]
struct EarthState {
    cur_height: u8,
    cur_length: u8,
    height: u8,
    length: u8,
    increment: i8,
}

impl EarthState {
    fn new() -> Self {
        let mut state = EarthState::default();
        state.next_mountain();
        state
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

        (1 << (NUM_ROWS as u8 - self.cur_height)) - 1
    }
}