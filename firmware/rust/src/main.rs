#![no_std]
#![no_main]

mod animation;
mod buttons;
mod panic;
mod random;

use embedded_hal::delay::DelayNs;
use random::Rand;

pub type CoreClock = avrxmega_hal::clock::MHz10;
pub type Delay = avrxmega_hal::delay::Delay<CoreClock>;

// The virtual display size is larger to accomodate the physical gaps between characters.
// The const COLUMN_GAP is the number of "empty" columns between characters and will set
// the NUM_VIRT_COLS value, the virtual "width" of the display. During display
// updates, specific columns are dropped/skipped to create final NUM_COLS-wide display buffer.
pub const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const NUM_VIRT_COLS: usize = NUM_COLS + (NUM_CHARS - 1) * COLUMN_GAP;
const COLUMN_GAP: usize = 2;

const BASE_DELAY_MS: u32 = 10;

struct Context {
    rng: Rand,
}

enum Event {
    Button(crate::buttons::ButtonEvent),
    // future events?
}

trait Mode {
    fn update(&mut self, event: &Event, ctx: &mut Context) -> [u8; NUM_VIRT_COLS];
}

#[avr_device::entry]
fn main() -> ! {
    let dp = avrxmega_hal::Peripherals::take().unwrap();
    let pins = avrxmega_hal::pins!(dp);

    // let mut adc = avrxmega_hal::Adc::new(dp.ADC0, Default::default());
    let mut buttons =
        buttons::Buttons::new(pins.pa7.into_pull_up_input(), pins.pb3.into_pull_up_input());
    let mut delay = Delay::new();

    // // read voltage from floating pin for reasonable entropy
    // let entropy_pin = pins.a0.into_analog_input(&mut adc);
    // let seed_value_1 = entropy_pin.analog_read(&mut adc);
    // let seed_value_2 = entropy_pin.analog_read(&mut adc);
    // let seed_value = (seed_value_1 as u32) << 16 | seed_value_2 as u32;
    // Rand::seed(seed_value);

    let mut display = hcms_29xx::Hcms29xx::<{ crate::NUM_CHARS }, _, _, _, _, _, _, _>::new(
        pins.pa6.into_output(),
        pins.pa4.into_output(),
        pins.pa3.into_output(),
        pins.pa2.into_output(),
        pins.pa1.into_output(),
        hcms_29xx::UnconfiguredPin,
        pins.pb0.into_output(),
    )
    .unwrap();

    display.begin().unwrap();
    display.display_unblank().unwrap();
    display
        .set_peak_current(hcms_29xx::PeakCurrent::Max6_4Ma)
        .unwrap();

    loop {
        let button_event = buttons.update();

        if let Some(event) = button_event {
            match event {
                crate::buttons::ButtonEvent::BothPressed => {
                    display.print_ascii_bytes(b"BothPres").unwrap();
                }
                crate::buttons::ButtonEvent::BothHeld => {
                    display.print_ascii_bytes(b"BothHeld").unwrap();
                }
                crate::buttons::ButtonEvent::LeftPressed => {
                    display.print_ascii_bytes(b"LeftPres").unwrap();
                }
                crate::buttons::ButtonEvent::LeftHeld => {
                    display.print_ascii_bytes(b"LeftHeld").unwrap();
                }
                crate::buttons::ButtonEvent::LeftReleased => {
                    display.print_ascii_bytes(b"LeftRele").unwrap();
                }
                crate::buttons::ButtonEvent::RightPressed => {
                    display.print_ascii_bytes(b"RightPre").unwrap();
                }
                crate::buttons::ButtonEvent::RightHeld => {
                    display.print_ascii_bytes(b"RightHel").unwrap();
                }
                crate::buttons::ButtonEvent::RightReleased => {
                    display.print_ascii_bytes(b"RightRel").unwrap();
                }
            }
        }

        delay.delay_ms(BASE_DELAY_MS);
    }
}
