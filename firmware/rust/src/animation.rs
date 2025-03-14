use crate::{
    Context, Event, Mode, Rand, BASE_DELAY_MS, COLUMN_GAP, NUM_CHARS, NUM_COLS, NUM_ROWS,
    NUM_VIRT_COLS,
};
use heapless::Vec;
use random_trait::Random; // Import the correct module based on feature flag

// virtual display size
pub const NUM_CLOUD_CHARS: usize = 8;
const NUM_CLOUD_COLS: usize =
    NUM_CLOUD_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_CLOUD_CHARS - 1) * COLUMN_GAP;
const SKY_PERIOD: u8 = 7;

pub const NUM_EARTH_CHARS: usize = 8;
const NUM_EARTH_COLS: usize =
    NUM_EARTH_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_EARTH_CHARS - 1) * COLUMN_GAP;
const EARTH_PERIOD: u8 = 3;

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
}

fn generate_cloud_column(state: &mut CloudState) -> u8 {
    let mut col = SKY_COL;

    if state.gap > 0 {
        state.gap -= 1;
    } else if state.cur_length < state.length {
        for i in 0..NUM_ROWS {
            let bit = if (i as u8) >= state.loc && (i as u8) < state.loc + state.height {
                0
            } else {
                1
            };
            col = col << 1 | bit;
        }
        state.cur_length += 1;
    } else {
        state.cur_length = 0;
        state.next_cloud();
    }

    col
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
}

fn generate_mountain_column(state: &mut MountainState) -> u8 {
    state.cur_height = (state.cur_height as i8 + state.increment) as u8;
    state.cur_length += 1;

    // shift in 0s from bottom/left to build to current height
    let mut col = SKY_COL;
    for _ in 0..state.cur_height {
        col = col >> 1;
    }

    // start going down
    if state.increment > 0 && state.cur_height >= state.height {
        state.increment = -1;
    }

    // stop going down
    if state.increment < 0 && state.cur_height == 0 {
        state.increment = 0;
    }

    // start new mountain
    if state.increment <= 0 && state.cur_length == state.length {
        state.next_mountain();
    }

    col
}

fn update() {
    // col bits: msb+1 is bottom row, lsb is top row, i.e. 0b0111_1111 is all on
    let mut cloud_cols: Vec<u8, NUM_CLOUD_COLS> = Vec::new();
    let mut earth_cols: Vec<u8, NUM_EARTH_COLS> = Vec::new();
    let mut cloud_counter: u8 = 0;
    let mut earth_counter: u8 = 0;

    let mut sky_state = CloudState::new();
    let mut earth_state = MountainState::new();

    // fill display buffer with initial columns
    for _ in 0..NUM_CLOUD_COLS {
        cloud_cols.push(SKY_COL).unwrap();
    }
    for _ in 0..NUM_EARTH_COLS {
        earth_cols.push(SKY_COL).unwrap();
    }

    // sky and mountains will update at different rates for parallax effect (embassy would be nice)
    loop {
        cloud_counter = (cloud_counter + 1) % SKY_PERIOD;
        if cloud_counter == 0 {
            let new_cloud_col = generate_cloud_column(&mut sky_state);
            cloud_cols.remove(0);
            cloud_cols.push(new_cloud_col).unwrap();
        }

        earth_counter = (earth_counter + 1) % EARTH_PERIOD;
        if earth_counter == 0 {
            let new_earth_col = generate_mountain_column(&mut earth_state);
            earth_cols.remove(0);
            earth_cols.push(new_earth_col).unwrap();
        }

        // let mut cols: Vec<u8, NUM_COLS> = Vec::new();
        // for i in 0..NUM_VIRT_COLS {
        //     if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
        //         let cloud_col = cloud_cols.get(i).copied().unwrap_or(0);
        //         let earth_col = earth_cols.get(i).copied().unwrap_or(0);
        //         cols.push(cloud_col & earth_col).unwrap();
        //     }
        // }

        // display.print_cols(&cols).unwrap();
        // delay.delay_ms(BASE_DELAY_MS);
    }
}
