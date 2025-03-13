use avrxmega_hal::port::{mode::Output, *};

pub type CoreClock = avrxmega_hal::clock::MHz10;
pub type Delay = avrxmega_hal::delay::Delay<CoreClock>;

pub const NUM_SKY_CHARS: usize = 8;
pub const NUM_EARTH_CHARS: usize = 8;
pub const OVERLAY: bool = true;

pub fn init() -> hcms_29xx::Hcms29xx<
    { crate::NUM_CHARS },
    avrxmega_hal::port::Pin<Output, PA6>,
    avrxmega_hal::port::Pin<Output, PA4>,
    avrxmega_hal::port::Pin<Output, PA3>,
    avrxmega_hal::port::Pin<Output, PA2>,
    avrxmega_hal::port::Pin<Output, PA1>,
    hcms_29xx::UnconfiguredPin,
    avrxmega_hal::port::Pin<Output, PB0>,
> {
    let dp = avrxmega_hal::Peripherals::take().unwrap();
    let pins = avrxmega_hal::pins!(dp);

    hcms_29xx::Hcms29xx::<{ crate::NUM_CHARS }, _, _, _, _, _, _, _>::new(
        pins.pa6.into_output(),
        pins.pa4.into_output(),
        pins.pa3.into_output(),
        pins.pa2.into_output(),
        pins.pa1.into_output(),
        hcms_29xx::UnconfiguredPin,
        pins.pb0.into_output(),
    )
    .unwrap()
}
