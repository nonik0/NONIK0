
use core::cell::RefCell;
use embedded_hal::digital::OutputPin;

// pub struct Hcms <DataPin: OutputPin, RegisterSelectPin: OutputPin, ClockPin: OutputPin, EnablePin: OutputPin, ResetPin: OutputPin>
// {
//     data_pin: DataPin,
//     register_select: RegisterSelectPin,
//     clock_pin: ClockPin,
//     enable_pin: EnablePin,
//     reset_pin: ResetPin,
// }

// pub struct Hcms29xx<Pin1, Pin2, Pin3, Pin4, const N: usize>
// where
//     Pin1: OutputPin,
//     Pin2: OutputPin,
//     Pin3: OutputPin,
//     Pin4: OutputPin,
// {
//     data: RefCell<Pin1>,
//     rs: RefCell<Pin2>,
//     clk: RefCell<Pin3>,
//     en: RefCell<Pin4>,
//     display_size: usize,
// }

// impl<Pin1, Pin2, Pin3, Pin4, const N: usize> Hcms29xx<Pin1, Pin2, Pin3, Pin4, const N: usize>
// where
//     Pin1: OutputPin,
//     Pin2: OutputPin,
//     Pin3: OutputPin,
//     Pin4: OutputPin,
// {

// }

pub struct Hcms29xx
{
    data: OutputPin,
    rs: OutputPin,
    clk: OutputPin,
    en: OutputPin,
    display_size: u8,
}