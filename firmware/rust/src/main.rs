#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(type_alias_impl_trait)]

mod input;
mod modes;
//mod panic;
use panic_halt as _;
mod random;
mod saved_settings;

use avrxmega_hal::eeprom::Eeprom;
use avrxmega_hal::port::{mode::Output, *};
use embedded_hal::delay::DelayNs;
use modes::*;
use random::Rand;

// using until proper ADC HAL implementation
type Adc0 = avrxmega_hal::pac::ADC0;
type Sigrow = avrxmega_hal::pac::SIGROW;
type Vref = avrxmega_hal::pac::VREF;
//type Adc = avrxmega_hal::adc::Adc<CoreClock>;
type CoreClock = avrxmega_hal::clock::MHz10;
type Delay = avrxmega_hal::delay::Delay<CoreClock>;
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
type Event = input::InputEvent;
type SavedSettings = saved_settings::SavedSettings;

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

    let mut buttons =
        input::Buttons::new(pins.pa7.into_pull_up_input(), pins.pb3.into_pull_up_input());
    let mut delay = Delay::new();

    let eeprom = Eeprom::new(dp.NVMCTRL);
    let settings = saved_settings::SavedSettings::new(eeprom);

    let mut context = Context::new(settings);
    let modes = modes::take(dp.ADC0, dp.SIGROW, dp.VREF, &context);

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
    display.set_brightness(context.settings.brightness()).unwrap();
    display.set_peak_current(context.settings.current()).unwrap();

    loop {
        let event = buttons.update();

        // special case to get always get back to menu
        if let Some(Event::BothHeld) = event {
            if !context.is_menu() {
                context.to_menu();
            }
        }

        modes[context.mode()].update(&event, &mut context, &mut display);

        delay.delay_ms(BASE_DELAY_MS);
    }
}
