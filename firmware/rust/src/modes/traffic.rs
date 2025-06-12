use super::ModeHandler;
use crate::{Context, Event, Peripherals, Rand, COLUMN_GAP, NUM_ROWS, NUM_VIRT_COLS};
use heapless::Vec;
use random_trait::Random;

const DRIVER_PERIOD_START: u8 = 35;
const TRAFFIC_PERIOD_START: u8 = 5;
const GOAL_COL: u8 = 0b0101_0101;
const GOAL_POS_START: u8 = ((NUM_VIRT_COLS >> 1) + hcms_29xx::CHAR_WIDTH) as u8;
const TRUCK_MAX_COUNT_START: usize = 1;
const MAX_TRUCKS: usize = 3;

#[derive(Clone, Copy)]
struct Truck {
    lane: u8,
    length: u8,
    width: u8,
}

impl Truck {
    fn rand(&mut self, lane_min: u8, lane_max: u8) {
        let mut rand = Rand::default();
        let max_width = lane_max - lane_min + 1;

        self.length = 2 + if max_width > 1 { rand.get_u8() % 7 } else { 0 };
        self.width = if max_width > 1 && self.length > 2 {
            2
        } else {
            1
        };

        self.lane = lane_min;
        let range = max_width - self.width + 1;
        if range > 0 {
            self.lane += rand.get_u8() % range;
        }
    }
}

pub struct Traffic {
    is_driving: bool,
    driver_counter: u8,
    driver_period: u8,
    driver_pos: u8,
    driver_lane: u8,

    goal_pos: u8,
    goal_col: u8,
    crashed: bool,

    traffic_cols: Vec<u8, NUM_VIRT_COLS>,
    traffic_counter: u8,
    traffic_period: u8,
    truck_max_count: usize,
    trucks: [Truck; MAX_TRUCKS],
}

impl Traffic {
    pub fn new() -> Self {
        let mut buf: Vec<u8, NUM_VIRT_COLS> = Vec::new();
        buf.resize_default(NUM_VIRT_COLS).ok();
        Traffic {
            // driver is a 2x1 rectangle
            is_driving: true,
            driver_counter: 0,
            driver_period: DRIVER_PERIOD_START,
            driver_pos: 1,
            driver_lane: NUM_ROWS as u8 >> 1,

            goal_pos: GOAL_POS_START,
            goal_col: GOAL_COL,
            crashed: false,

            // traffic will have random "blocks" (i.e. trucks) to driver around
            traffic_cols: buf,
            traffic_counter: 0,
            traffic_period: TRAFFIC_PERIOD_START,
            truck_max_count: TRUCK_MAX_COUNT_START,
            trucks: [Truck {
                lane: 0,
                length: 0,
                width: 0,
            }; MAX_TRUCKS],
        }
    }

    fn driver_col(&self) -> u8 {
        0b00000001 << (NUM_ROWS as u8 - self.driver_lane - 1)
    }

    fn truck_col(truck: &Truck) -> u8 {
        let mut col = 0;
        for i in 0..truck.width {
            if truck.lane + i < NUM_ROWS as u8 {
                col |= 0b00000001 << (NUM_ROWS as u8 - truck.lane - i - 1);
            }
        }
        col
    }

    fn is_driver_pos(&self, pos: usize) -> bool {
        let pos = pos as u8;
        pos == self.driver_pos || pos == self.driver_pos - 1
    }

    fn clear_traffic(&mut self) {
        self.crashed = false;
        for truck in &mut self.trucks {
            truck.length = 0;
        }

        self.traffic_cols.clear();
        self.traffic_cols.resize_default(NUM_VIRT_COLS).ok();
    }

    fn next_driver_pos(&mut self) -> Option<u8> {
        self.driver_counter = (self.driver_counter + 1) % self.driver_period;
        if self.driver_counter != 0 {
            return None;
        }

        self.driver_pos += 1;
        Some(self.driver_pos)
    }

    fn next_traffic_col(&mut self) -> Option<u8> {
        self.traffic_counter = (self.traffic_counter + 1) % self.traffic_period;
        if self.traffic_counter != 0 {
            return None;
        }

        // generate next col
        let mut next_col: u8 = 0;
        let mut truck_count = 0;
        for truck in &mut self.trucks {
            if truck.length > 0 {
                next_col |= Self::truck_col(truck);
                truck.length -= 1;
                truck_count += 1;
            }
        }

        // Find gaps
        const MIN_GAP_SIZE: usize = 3;
        let mut gap_lanes = [0u8; MAX_TRUCKS + 1];
        let mut gap_sizes = [0u8; MAX_TRUCKS + 1];
        let mut gap_count = 0;
        let mut lane = 0u8;
        while lane < NUM_ROWS as u8 {
            let mut gap = 0u8;
            while lane + gap < NUM_ROWS as u8
                && (next_col & (0b00000001 << (NUM_ROWS as u8 - lane - gap - 1))) == 0
            {
                gap += 1;
            }
            if gap >= MIN_GAP_SIZE as u8 {
                gap_lanes[gap_count] = lane;
                gap_sizes[gap_count] = gap;
                gap_count += 1;
            }
            lane += gap + 1;
        }

        let truck_chance = 7 + truck_count as u8;
        if gap_count > 0
            && truck_count < self.truck_max_count
            && (Rand::default().get_u8() % truck_chance) == 0
        {
            for truck in &mut self.trucks {
                if truck.length == 0 {
                    let gap_index = Rand::default().get_u8() as usize % gap_count;
                    let gap_lane_start = gap_lanes[gap_index];
                    let gap_lane_end = gap_lane_start + gap_sizes[gap_index] - 1;
                    let min_truck_lane = if gap_lane_start == 0 {
                        0
                    } else {
                        gap_lane_start + 1
                    };
                    let max_truck_lane = if gap_lane_end == NUM_ROWS as u8 - 1 {
                        NUM_ROWS as u8 - 1
                    } else {
                        gap_lane_end - 1
                    };
                    truck.rand(min_truck_lane, max_truck_lane);
                    break;
                }
            }
        }

        Some(next_col)
    }
}

impl ModeHandler for Traffic {
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
                    if !self.is_driving {
                        update = true;

                        // clear traffic and signal restart by setting truck max count to 0
                        self.clear_traffic();
                        self.truck_max_count = 0;
                    }
                }
                Event::LeftReleased => {
                    update = true;
                    if self.driver_lane < NUM_ROWS as u8 - 1 {
                        self.driver_lane += 1;
                    }
                }
                Event::RightReleased => {
                    update = true;
                    if self.driver_lane > 0 {
                        self.driver_lane -= 1;
                    }
                }
                _ => {}
            }
        }

        if self.is_driving {
            if let Some(new_traffic_col) = self.next_traffic_col() {
                update = true;

                self.goal_col = !self.goal_col;
                self.traffic_cols.remove(0);
                self.traffic_cols.push(new_traffic_col).unwrap();
            }

            if let Some(driver_pos) = self.next_driver_pos() {
                update = true;

                // stage complete
                if driver_pos == self.goal_pos {
                    self.is_driving = false;
                    self.clear_traffic();
                }
            }
        } else if !self.crashed {
            update = true;

            self.driver_pos -= 1;
            if self.driver_pos == 1 {
                // if truck_max_count > 0 is stage success, otherwise stage failure
                if self.truck_max_count > 0 {
                    // drive faster
                    if self.driver_period > 2 {
                        self.driver_period -= 1;
                    }

                    // move goal forward
                    self.goal_pos += (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) as u8;

                    // goal resets increase diff
                    if self.goal_pos >= NUM_VIRT_COLS as u8 - 1 {
                        self.goal_pos = GOAL_POS_START;

                        if self.truck_max_count < MAX_TRUCKS {
                            self.truck_max_count += 1;
                        } else if self.traffic_period > 2 {
                            self.truck_max_count = 1;
                            self.traffic_period -= 1;
                        } else if self.driver_period > 2 {
                            self.driver_period -= 1;
                        }
                    }
                } else {
                    self.goal_pos = GOAL_POS_START;
                    self.driver_period = DRIVER_PERIOD_START;
                    self.traffic_period = TRAFFIC_PERIOD_START;
                    self.truck_max_count = TRUCK_MAX_COUNT_START;
                }

                self.is_driving = true;
            }
        }

        if update {
            let mut cols: Vec<u8, NUM_VIRT_COLS> = Vec::new();
            for col_pos in 0..NUM_VIRT_COLS {
                if col_pos % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                    let mut col = self.traffic_cols.get(col_pos).copied().unwrap_or(0);

                    if self.is_driver_pos(col_pos) {
                        // col_pos can wrap around at start but should be OK
                        let collision = col & self.driver_col() != 0;
                        if collision {
                            self.is_driving = false;
                            self.crashed = true;
                        }

                        col |= self.driver_col();
                    } else if self.is_driving && col_pos == self.goal_pos as usize {
                        col |= self.goal_col;
                    }

                    cols.push(col).unwrap();
                }
            }

            peripherals.display.print_cols(cols.as_slice()).unwrap();
        }
    }
}
