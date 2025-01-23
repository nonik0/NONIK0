#![no_std]
#![no_main]

mod hcms29xx;

use arduino_hal::prelude::*;
//use arduino_hal::usart::{Baudrate, Usart};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output(); // TODO: pin for feather
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // let mut serial = Usart::new(
    //     dp.USART1,
    //     pins.d0,
    //     pins.d1.into_output(),
    //     Baudrate::<arduino_hal::DefaultClock>::new(57600),
    // );
    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap();

    let mut display = hcms29xx::Hcms29xx::new(
        8,
        //pins.d0.into_output().downgrade(),
        //pins.d1.into_output().downgrade(),
        pins.d9.into_output().downgrade(),
        pins.d12.into_output().downgrade(),
        pins.d11.into_output().downgrade(),
        pins.d2.into_output().downgrade(),
        Some(pins.d3.into_output().downgrade()),
        Some(pins.d6.into_output().downgrade()),
    )
    .unwrap();
    display.begin().unwrap();
    display.display_unblank().unwrap();
    display.set_int_osc().unwrap();

    loop {
        led.toggle();
        //display.print_c_string(b"TEST1234").unwrap();
        ufmt::uwriteln!(&mut serial, "toggle!\r").unwrap();
        arduino_hal::delay_ms(1000);
    }
}

#[cfg(not(doc))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // Print out panic location
    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap_infallible();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .unwrap_infallible();
    }

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}
