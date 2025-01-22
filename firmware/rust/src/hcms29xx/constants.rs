use bitflags::bitflags;

pub const CHAR_WIDTH: u8 = 5;
pub const CHAR_HEIGHT: u8 = 7;
pub const DEVICE_CHARS: u8 = 4;

//
// define the bitflags fto select control word 0 or 1
//

//pub const CONTROL_WORD_SELECT_MASK: u8 = 0b1000_0000;
// pub const CONTROL_WORD_1: u8 = 0b0000_0000;
// pub const CONTROL_WORD_2: u8 = 0b1000_0000;

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Select {
//     ControlWord0 = 0b0,
//     ControlWord1 = 0b1,
// }

//
// define the bitflags for control word 0
//

bitflags! {
    pub struct ControlWord0: u8 {
        const SELECT = 0b0000_0000;
        const BRIGHTNESS_MASK = 0b0000_1111;
        //const CURRENT_MASK = 0b0011_0000;
        const CURRENT_4_0MA = 0b0010_0000;
        const CURRENT_6_4MA = 0b0001_0000;
        const CURRENT_9_3MA = 0b0000_0000;
        const CURRENT_12_8MA = 0b0011_0000;
        //const SLEEP_MASK = 0b0100_0000;
        const NORMAL_OPERATION = 0b0100_0000;
        const SLEEP = 0b0000_0000;
    }
}

// // values in bitfield are linearly mapped, no enum needed
pub const MAX_BRIGHTNESS: u8 = 15;
pub const DEFAULT_BRIGHTNESS: u8 = 12;

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum PeakCurrent {
//     Current4_0mA = 0b10,
//     Current6_4mA = 0b01,
//     Current9_3mA = 0b00,
//     Current12_8mA = 0b11,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum SleepMode {
//     Sleep = 0b0, // int oscillator off, display blanked
//     NormalOperation = 0b1,
// }

//
// define the bitflags for control word 1
//

bitflags! {
    pub struct ControlWord1: u8 {
        const SELECT = 0b0000_0000;
        //const DATA_OUT = 0b0000_0001; // low: serial, high: simultaneous
        const DATA_OUT_SERIAL = 0b0000_0000;
        const DATA_OUT_SIMULTANEOUS = 0b0000_0001;
        //const EXT_OSC_PRESCALER = 0b0000_0010; // low: clock/1, clock/8
        const EXT_OSC_PRESCALER_DIRECT = 0b0000_0000;
        const EXT_OSC_PRESCALER_PRESCALE8 = 0b0000_0010;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataOutMode {
    Serial = 0b0,
    Simultaneous = 0b1,
}

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ExtOscPrescaler {
//     Direct = 0b0,
//     Prescale8 = 0b1,
// }