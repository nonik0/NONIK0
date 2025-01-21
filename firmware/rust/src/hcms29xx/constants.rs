use bitflags::bitflags;

pub const CHAR_WIDTH: u8 = 5;
pub const CHAR_HEIGHT: u8 = 7;

bitflags! {
    pub struct ControlWord0: u8 {
        const SELECT = 0b0000_0000;
        const PWM_MASK = 0b0000_1111;
        const CURRENT_MASK = 0b0011_0000;
        const SLEEP_MASK = 0b0100_0000;

        // const DEFAULT_PWM = 0b1100;
        // const DEFAULT_CURRENT 0b10;
    }    

    pub struct ControlWord1: u8 {
        const SELECT = 0b1000_0000;
        const DOUT_MODE = 0b0000_0001; // low: serial, high: simultaneous
        const EXT_OSC_PRESCALAR = 0b0000_0010; // low: clock/1, clock/8
    }
}