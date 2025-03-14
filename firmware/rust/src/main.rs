#![no_std]
#![no_main]

//mod animation;
mod buttons;
mod game;
mod menu;
mod nametag;
mod panic;
mod random;

use avrxmega_hal::port::{mode::Output, *};
use embedded_hal::delay::DelayNs;
use random::Rand;

pub type CoreClock = avrxmega_hal::clock::MHz10;
pub type Delay = avrxmega_hal::delay::Delay<CoreClock>;
pub type Event = buttons::ButtonEvent;
type Display = hcms_29xx::Hcms29xx<
    NUM_CHARS,
    Pin<Output, PA6>,
    Pin<Output, PA4>,
    Pin<Output, PA3>,
    Pin<Output, PA2>,
    Pin<Output, PA1>,
    hcms_29xx::UnconfiguredPin,
    Pin<Output, PB0>,
>;

// The virtual display size is larger to accomodate the physical gaps between characters.
// The const COLUMN_GAP is the number of "empty" columns between characters and will set
// the NUM_VIRT_COLS value, the virtual "width" of the display. During display
// updates, specific columns are dropped/skipped to create final NUM_COLS-wide display buffer.
pub const NUM_MODES: usize = 3;
pub const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const NUM_VIRT_COLS: usize = NUM_COLS + (NUM_CHARS - 1) * COLUMN_GAP;
const COLUMN_GAP: usize = 2;

const BASE_DELAY_MS: u32 = 10;

pub struct Context {
    mode_counter: u16,
    mode_index: usize,
    rand: Rand,
}

pub trait Mode {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context);
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

    //let mut animation = animation::Animation::new();
    let mut nametag = nametag::Nametag::new();
    let mut game = game::Game::new();
    let mut menu = menu::Menu::new();


    //let modes: [&mut dyn Mode; NUM_MODES] = [&mut menu, &mut nametag, &mut animation, &mut game];
    let modes: [&mut dyn Mode; NUM_MODES] = [&mut menu, &mut nametag, &mut game];

    let mut context = Context {
        mode_counter: 0,
        mode_index: 1,
        rand: Rand,
    };

    loop {
        let event = buttons.update();

        // special case to get to menu
        if let Some(Event::BothHeld) = event {
            context.mode_index = 0;
            context.mode_counter += 1;
        }

        modes[context.mode_index].update(&event, &mut display, &mut context);

        delay.delay_ms(BASE_DELAY_MS);
    }
}
