#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(type_alias_impl_trait)]

mod adc;
mod i2c;
mod input;
mod modes;
#[cfg(feature = "debug_panic")]
mod panic;
#[cfg(not(feature = "debug_panic"))]
use panic_halt as _;
mod random;
mod saved_settings;
mod tone;
mod utils;

use avrxmega_hal::eeprom::Eeprom;
use avrxmega_hal::port::{mode::Output, *};
use embedded_hal::delay::DelayNs;
use modes::*;
use random::Rand;

// using until proper ADC HAL implementation
type Adc0 = avrxmega_hal::pac::ADC0;
//type I2C = avrxmega_hal::pac::TWI0;
type Sigrow = avrxmega_hal::pac::SIGROW;
type Vref = avrxmega_hal::pac::VREF;
//type Adc = avrxmega_hal::adc::Adc<CoreClock>;
type CoreClock = avrxmega_hal::clock::MHz10;
type Delay = avrxmega_hal::delay::Delay<CoreClock>;
//type I2c = avrxmega_hal::i2c::I2c<CoreClock>;
#[cfg(feature = "board_v0")]
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
#[cfg(not(feature = "board_v0"))]
type Display = hcms_29xx::Hcms29xx<
    NUM_CHARS,
    Pin<Output, PA6>,
    Pin<Output, PA4>,
    Pin<Output, PA3>,
    Pin<Output, PA2>,
    Pin<Output, PA1>,
    hcms_29xx::UnconfiguredPin,
    Pin<Output, PB2>,
>;
type DisplayPeakCurrent = hcms_29xx::PeakCurrent;
type Event = input::InputEvent;
type Setting = saved_settings::Setting;
type SavedSettings = saved_settings::SavedSettings;

// The virtual display size is larger to accomodate the physical gaps between characters.
// The const COLUMN_GAP is the number of "empty" columns between characters and will set
// the NUM_VIRT_COLS value, the virtual "width" of the display. During display
// updates, specific columns are dropped/skipped to create final NUM_COLS-wide display buffer.
const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const NUM_VIRT_COLS: usize = NUM_COLS + (NUM_CHARS - 1) * COLUMN_GAP;
const COLUMN_GAP: usize = 2;

const BASE_DELAY_MS: u32 = 5;
const I2C_BUS_SPEED: u32 = 100_000; // 100kHz

#[avr_device::entry]
fn main() -> ! {
    let dp = avrxmega_hal::Peripherals::take().unwrap();
    let pins = avrxmega_hal::pins!(dp);

    let mut buttons =
        input::Buttons::new(pins.pa7.into_pull_up_input(), pins.pb3.into_pull_up_input());
    let mut delay = Delay::new();

    let mut adc = adc::Adc::new(dp.ADC0, dp.SIGROW, dp.VREF);
    adc.seed_rand();

    #[cfg(not(feature = "no_i2cutils"))]
    let i2c = i2c::I2c::new(
        dp.TWI0,
        pins.pb1.into_pull_up_input(),
        pins.pb0.into_pull_up_input(),
        I2C_BUS_SPEED,
    );

    let eeprom = Eeprom::new(dp.NVMCTRL);
    let settings = saved_settings::SavedSettings::new(eeprom);
    let buzzer = tone::Tone::new(dp.TCB0, pins.pa5.into_output());
   
    let mut display = Display::new(
        pins.pa6.into_output(),
        pins.pa4.into_output(),
        pins.pa3.into_output(),
        pins.pa2.into_output(),
        pins.pa1.into_output(),
        hcms_29xx::UnconfiguredPin,
        #[cfg(feature = "board_v0")]
        pins.pb0.into_output(),
        #[cfg(not(feature = "board_v0"))]
        pins.pb2.into_output(),
    )
    .unwrap();
    display.begin().unwrap();
    display.display_unblank().unwrap();

    let mut context = Context::new(settings);
    let mut peripherals = Peripherals::new(
        adc,
        buzzer,
        display,
        #[cfg(not(feature = "no_i2cutils"))]
        i2c,
    );

    // TODO: improve, apply saved display settings
    let settings = Settings::new_with_settings(&context.settings);
    settings.apply(&mut peripherals.display);

    // initialize default/saved mode
    let mut mode = Mode::from_context(&context);
    let mut mode_index = context.mode_index();
    loop {
        let event = buttons.update();

        match event {
            // special case to get always get back to menu
            Some(Event::BothHeld) => {
                if !context.is_menu() {
                    context.to_menu();
                }
            }
            // higher/shorter tone on button press
            Some(Event::LeftPressed) | Some(Event::RightPressed) => {
                if context.tone_enabled {
                    peripherals.buzzer.tone(5000, 5);
                }
            }
            // lower/longer tone on button held press
            Some(Event::LeftHeld) | Some(Event::RightHeld) => {
                if context.tone_enabled {
                    peripherals.buzzer.tone(4000, 10);
                }
            }
            _ => {}
        }

        // change mode when requested
        if mode_index != context.mode_index() {
            mode_index = context.mode_index();
            mode = Mode::from_context(&context);
        }

        mode.update(&event, &mut context, &mut peripherals);

        delay.delay_ms(BASE_DELAY_MS);
    }
}
