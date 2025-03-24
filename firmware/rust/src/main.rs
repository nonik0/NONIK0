#![no_std]
#![no_main]

mod panic;
mod random;
mod vibes;

use arduino_hal::port::{mode::Output, *};
use random::Rand;

type Display = hcms_29xx::Hcms29xx<
    NUM_CHARS,
    Pin<Output, D0>,
    Pin<Output, D1>,
    Pin<Output, D11>,
    Pin<Output, D2>,
    Pin<Output, D3>,
    Pin<Output, D6>,
    Pin<Output, D10>,
>;

const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const NUM_VIRT_COLS: usize = NUM_COLS + (NUM_CHARS - 1) * COLUMN_GAP;
const COLUMN_GAP: usize = 2;

const BASE_DELAY_MS: u16 = 10;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let entropy_pin = pins.a4.into_analog_input(&mut adc);
    Rand::seed(adc.read_blocking(&entropy_pin) as u32);

    //let mut led_pin = pins.d13.into_output();

    // high impedance pins
    pins.sck.into_floating_input();
    pins.mosi.into_floating_input();
    pins.d9.into_floating_input();
    pins.d5.into_floating_input();

    let mut display = hcms_29xx::Hcms29xx::<NUM_CHARS, _, _, _, _, _, _, _>::new(
        pins.d0.into_output(),  // Data pin
        pins.d1.into_output(),  // RS pin
        pins.d11.into_output(), // Clock pin
        pins.d2.into_output(),  // CE pin
        pins.d3.into_output(),  // Optional: Blank pin
        pins.d6.into_output(),  // Optional: OscSel pin
        pins.d10.into_output(), // Optional: Reset pin
    )
    .unwrap();

    display.begin().unwrap();
    display.display_unblank().unwrap();
    display
        .set_peak_current(hcms_29xx::PeakCurrent::Max6_4Ma)
        .unwrap();

    let mut vibes = vibes::Vibes::new();
    loop {
        vibes.update(&mut display);
        arduino_hal::delay_ms(BASE_DELAY_MS);
    }
}
