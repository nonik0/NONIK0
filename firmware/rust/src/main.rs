#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(type_alias_impl_trait)]

// TODO: add back feather support now code is better organized (featuregating modes, etc.)
// TODO: optimize power usage by going to sleep when not in use (i.e. nametag mode)
// TODO: implement interrupt-based button handling
// TODO: implement tone generation for buzzer
// TODO: implement ADC for rand seeding
// TODO: implement EEPROM for persisting state
// TODO: implement Random with OnceCell for portability, can use avr_hal sync types

mod eeprom;
mod input;
mod modes;
//mod panic;
use panic_halt as _;
mod random;

use avrxmega_hal::port::{mode::Output, *};
use embedded_hal::delay::DelayNs;
use modes::*;
use random::Rand;

type Adc = avrxmega_hal::adc::Adc<CoreClock>;
type CoreClock = avrxmega_hal::clock::MHz10;
type Delay = avrxmega_hal::delay::Delay<CoreClock>;
type Event = input::InputEvent;
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
type DisplayPeakCurrent = hcms_29xx::PeakCurrent;

const DEFAULT_BRIGHTNESS: u8 = 12;
const DEFAULT_CURRENT: DisplayPeakCurrent = DisplayPeakCurrent::Max6_4Ma;

// The virtual display size is larger to accomodate the physical gaps between characters.
// The const COLUMN_GAP is the number of "empty" columns between characters and will set
// the NUM_VIRT_COLS value, the virtual "width" of the display. During display
// updates, specific columns are dropped/skipped to create final NUM_COLS-wide display buffer.
const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const NUM_VIRT_COLS: usize = NUM_COLS + (NUM_CHARS - 1) * COLUMN_GAP;
const COLUMN_GAP: usize = 2;

const BASE_DELAY_MS: u32 = 10;

#[avr_device::entry]
fn main() -> ! {
    let dp = avrxmega_hal::Peripherals::take().unwrap();
    let pins = avrxmega_hal::pins!(dp);

    let mut adc = Adc::new(dp.ADC0, Default::default());
    let mut buttons =
        input::Buttons::new(pins.pa7.into_pull_up_input(), pins.pb3.into_pull_up_input());
    let mut delay = Delay::new();
    
    eeprom::Eeprom::init(dp.NVMCTRL);
    let settings = eeprom::EepromSettings::read();

    // read voltage from floating pin for maybe some entropy
    let entropy_pin = pins.pb1.into_analog_input(&mut adc);
    let seed_value_1 = entropy_pin.analog_read(&mut adc);
    let seed_value_2 = entropy_pin.analog_read(&mut adc);
    let seed_value_3 = entropy_pin.analog_read(&mut adc);
    let seed_value_4 = entropy_pin.analog_read(&mut adc);
    let seed_value = (seed_value_1 as u32) << 24 | (seed_value_2 as u32) << 16 | (seed_value_3 as u32) << 8 | seed_value_4 as u32;
    Rand::seed(seed_value);

    let mut context = Context::default();
    let modes = modes::take(&settings);

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
    display.set_brightness(settings.brightness).unwrap();
    display.set_peak_current(settings.current).unwrap();

    loop {
        let event = buttons.update();

        // special case to get always get back to menu
        if let Some(Event::BothHeld) = event {
            if !context.is_menu() {
                context.to_menu();
            }
        }

        modes[context.mode()].update(&event, &mut display, &mut context);

        delay.delay_ms(BASE_DELAY_MS);
    }
}
