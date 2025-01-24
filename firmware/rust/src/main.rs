#![no_std]
#![no_main]

use panic_halt as _;
mod hcms29xx;

const NUM_CHARS: usize = 8;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led_pin = pins.d13.into_output();

    // high impedance pins
    pins.sck.into_floating_input();
    pins.mosi.into_floating_input();
    pins.d9.into_floating_input();
    pins.d5.into_floating_input();

    let mut display = hcms29xx::Hcms29xx::new(
        NUM_CHARS,                                // Number of characters in the display
        pins.d0.into_output().downgrade(),        // Data pin
        pins.d1.into_output().downgrade(),        // Clock pin
        pins.d11.into_output().downgrade(),       // Chip select pin
        pins.d2.into_output().downgrade(),        // Reset pin
        Some(pins.d3.into_output().downgrade()),  // Optional: Enable pin
        Some(pins.d6.into_output().downgrade()),  // Optional: Write pin
        Some(pins.d10.into_output().downgrade()), // Optional: Read pin
    )
    .unwrap();
    display.begin().unwrap();
    display.display_unblank().unwrap();
    display.set_int_osc().unwrap();

    loop {
        led_pin.toggle();
        display.print_c_string(b"TEST1234").unwrap();
        arduino_hal::delay_ms(1000);
    }
}
