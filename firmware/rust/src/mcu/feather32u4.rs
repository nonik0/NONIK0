use arduino_hal::port::{mode::Output, *};

pub const NUM_SKY_CHARS: usize = 4;
pub const NUM_EARTH_CHARS: usize = 4;
pub const OVERLAY: bool = false;

pub fn init() -> hcms_29xx::Hcms29xx<
    super::NUM_CHARS,
    arduino_hal::port::Pin<Output, D0>,
    arduino_hal::port::Pin<Output, D1>,
    arduino_hal::port::Pin<Output, D11>,
    arduino_hal::port::Pin<Output, D2>,
    arduino_hal::port::Pin<Output, D3>,
    arduino_hal::port::Pin<Output, D6>,
    arduino_hal::port::Pin<Output, D10>,
> {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    hcms_29xx::Hcms29xx::<super::NUM_CHARS, _, _, _, _, _, _, _>::new(
        pins.d0.into_output(),
        pins.d1.into_output(),
        pins.d11.into_output(),
        pins.d2.into_output(),
        pins.d3.into_output(),
        pins.d6.into_output(),
        pins.d10.into_output(),
    )
    .unwrap()
}
