#![no_std]
#![no_main]

mod hcms29xx;

use atmega_hal::usart::{Baudrate, Usart};
use embedded_hal::delay::DelayNs;
use panic_halt as _;

type CoreClock = atmega_hal::clock::MHz8;
type Delay = atmega_hal::delay::Delay<crate::CoreClock>;

fn delay_ms(ms: u16) {
    Delay::new().delay_ms(u32::from(ms))
}

#[allow(dead_code)]
fn delay_us(us: u32) {
    Delay::new().delay_us(us)
}

#[avr_device::entry]
fn main() -> ! {
    let dp = atmega_hal::Peripherals::take().unwrap();
    let pins = atmega_hal::pins!(dp);

    let mut led = pins.pb7.into_output();
    let mut _serial = Usart::new(
        dp.USART1,
        pins.pd2,
        pins.pd3.into_output(),
        Baudrate::<crate::CoreClock>::new(57600),
    );

    loop {
        led.toggle();
        delay_ms(1000);
    }
}