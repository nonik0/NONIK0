use crate::{Delay, NUM_CHARS};

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use heapless::String;

// TODO: I think core::fmt is huge, try ufmt instead

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();
    let mut delay = Delay::new();
    let dp = unsafe { avrxmega_hal::Peripherals::steal() };
    let pins = avrxmega_hal::pins!(dp);

    let mut display = hcms_29xx::Hcms29xx::<{ NUM_CHARS }, _, _, _, _, _, _, _>::new(
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

    let mut panic_msg: String<64> = String::new();
    for _ in 0..NUM_CHARS {
        panic_msg.push(' ').ok();
    }
    panic_msg.push_str("PANIC! ").ok();

    if let Some(loc) = info.location() {
        write!(
            &mut panic_msg,
            "{}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .ok();
    }
    for _ in 0..NUM_CHARS {
        panic_msg.push(' ').ok();
    }

    let panic_msg = panic_msg.as_bytes();
    let mut cursor = 0;
    loop {
        display
            .print_ascii_bytes(&panic_msg[cursor..cursor + NUM_CHARS])
            .ok();
        cursor = (cursor + 1) % (panic_msg.len() - NUM_CHARS);

        delay.delay_ms(50);
    }
}
