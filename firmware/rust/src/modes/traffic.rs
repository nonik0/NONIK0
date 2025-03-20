use super::Mode;
use crate::{Context, Display, Event, Rand, COLUMN_GAP, NUM_ROWS, NUM_VIRT_COLS};
use heapless::Vec;
use random_trait::Random;

#[derive(Clone, Copy, Debug, Default)]
struct Truck {
    lane: u8,
    length: u8,
    width: u8,
}

impl Truck {
    fn rand(&mut self, lane_min: u8, lane_max: u8) {
        let mut rand = Rand::default();

        let max_width = lane_max - lane_min + 1;

        self.length = 2;
        if max_width > 1 {
            self.length += rand.get_u8() % 7;
        };

        self.width = 1;
        if max_width > 1 && self.length > 2 {
            self.width = 2;
        }

        self.lane = lane_min;
        let range = max_width - self.width + 1;
        if range > 0 {
            self.lane += rand.get_u8() % range;
        }
    }
}

pub struct Traffic {
    last_update: u16,

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
    truck_count: usize,
    trucks: [Truck; Self::MAX_TRUCKS],
}

impl Traffic {
    const DRIVER_PERIOD_START: u8 = 20;
    const TRAFFIC_PERIOD_START: u8 = 3;
    const GOAL_COL: u8 = 0b0101_0101;
    const GOAL_POS_START: u8 = ((NUM_VIRT_COLS >> 1) + hcms_29xx::CHAR_WIDTH) as u8;
    const TRUCK_MAX_COUNT_START: usize = 1;
    const MAX_TRUCKS: usize = 3;

    pub fn new() -> Self {
        let mut buf: Vec<u8, NUM_VIRT_COLS> = Vec::new();
        for _ in 0..NUM_VIRT_COLS {
            buf.push(0).unwrap();
        }

        Traffic {
            last_update: 0,

            // driver is a 2x1 rectangle
            is_driving: true,
            driver_counter: 0,
            driver_period: Self::DRIVER_PERIOD_START,
            driver_pos: 1,
            driver_lane: NUM_ROWS as u8 >> 1,

            goal_pos: Self::GOAL_POS_START,
            goal_col: Self::GOAL_COL,
            crashed: false,

            // traffic will have random "blocks" (i.e. trucks) to driver around
            traffic_cols: buf,
            traffic_counter: 0,
            traffic_period: Self::TRAFFIC_PERIOD_START,

            truck_max_count: Self::TRUCK_MAX_COUNT_START,
            truck_count: 0,
            trucks: [Truck::default(); Self::MAX_TRUCKS],
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
        self.truck_count = 0;
        for i in 0..Self::MAX_TRUCKS {
            self.trucks[i].length = 0;
        }

        self.traffic_cols.clear();
        for _ in 0..NUM_VIRT_COLS {
            self.traffic_cols.push(0).unwrap();
        }

        // in future clear specific traffic only
        // // clear all traffic behind goal
        // for i in 0..self.goal_pos + 1 {
        //     self.traffic_cols[i as usize] = 0;
        // }
        // // trim width 1 trucks that are behind goal
        // if self.goal_pos < NUM_VIRT_COLS as u8 - 1 {
        //     let col_after_goal = self
        //         .traffic_cols
        //         .get(self.goal_pos as usize + 1)
        //         .copied()
        //         .unwrap_or(0);
        //     let col_after_goal2 = self
        //         .traffic_cols
        //         .get(self.goal_pos as usize + 2)
        //         .copied()
        //         .unwrap_or(0);
        //     self.traffic_cols[self.goal_pos as usize + 1] &=
        //         !(col_after_goal | (col_after_goal ^ col_after_goal2));
        // }
        // TODO: more cleanup
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
        for i in 0..self.truck_count {
            let truck = &mut self.trucks[i];

            if truck.length > 0 {
                next_col |= Self::truck_col(truck);
                truck.length -= 1;
            } else if truck.length == 0 {
                // delay in decreasing count allows for random gap behind
                let mut rand = Rand::default();
                if rand.get_u8() % 2 == 0 {
                    self.truck_count -= 1;
                }
            }
        }

        let mut rand = Rand::default();

        // Find the available gaps in the traffic lanes
        const MIN_GAP_SIZE: usize = 3;
        let mut gap_lanes = [0u8; Self::MAX_TRUCKS + 1];
        let mut gap_sizes = [0u8; Self::MAX_TRUCKS + 1];
        let mut gap_count = 0;

        let mut gap_start_lane = 0u8;
        while gap_start_lane < NUM_ROWS as u8 {
            let mut gap_stop_lane = gap_start_lane;

            while (gap_stop_lane < NUM_ROWS as u8)
                && (next_col & (0b00000001 << (NUM_ROWS as u8 - gap_stop_lane as u8 - 1)) == 0)
            {
                gap_stop_lane += 1;
            }

            let gap_size = gap_stop_lane - gap_start_lane;
            if gap_size >= MIN_GAP_SIZE as u8 {
                gap_lanes[gap_count] = gap_start_lane;
                gap_sizes[gap_count] = gap_size;
                gap_count += 1;
            }

            gap_start_lane = gap_stop_lane + 1;
        }

        // if there's room for another truck
        let truck_chance = 7 + self.truck_count as u8;
        if gap_count > 0
            && self.truck_count < self.truck_max_count
            && (rand.get_u8() % truck_chance) == 0
        {
            for i in 0..Self::MAX_TRUCKS {
                if self.trucks[i].length == 0 {
                    // pick random gap to place truck within, size 3 gaps can only have 1 width trucks
                    let gap_index = rand.get_u8() as usize % gap_count;
                    let gap_lane_start = gap_lanes[gap_index] as u8;
                    let gap_lane_end = gap_lane_start + gap_sizes[gap_index] as u8 - 1;

                    // min lane can be adjacent to edge but not truck
                    let min_truck_lane = if gap_lane_start == 0 {
                        0
                    } else {
                        gap_lane_start + 1
                    };

                    // max lane can be adjacent to edge but not truck
                    let max_truck_lane = if gap_lane_end == NUM_ROWS as u8 - 1 {
                        NUM_ROWS as u8 - 1
                    } else {
                        gap_lane_end - 1
                    };

                    self.trucks[i].rand(min_truck_lane, max_truck_lane);
                    self.truck_count += 1;
                    break;
                }
            }
        }

        Some(next_col)
    }
}

impl Mode for Traffic {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

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
                        self.goal_pos = Self::GOAL_POS_START;

                        if self.truck_max_count < Self::MAX_TRUCKS {
                            self.truck_max_count += 1;
                        } else if self.traffic_period > 2 {
                            self.truck_max_count = 1;
                            self.traffic_period -= 1;
                        } else if self.driver_period > 2 {
                            self.driver_period -= 1;
                        }
                    }
                } else {
                    self.goal_pos = Self::GOAL_POS_START;
                    self.driver_period = Self::DRIVER_PERIOD_START;
                    self.traffic_period = Self::TRAFFIC_PERIOD_START;
                    self.truck_max_count = Self::TRUCK_MAX_COUNT_START;
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

            display.print_cols(cols.as_slice()).unwrap();
        }
    }
}
