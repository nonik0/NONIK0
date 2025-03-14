use crate::{Delay, NUM_CHARS};

// using ufmt::uWrite over core::fmt::Write saves like ~1kB
use core::convert::Infallible;
use embedded_hal::delay::DelayNs;
use ufmt::{uwrite, uWrite};

struct ByteArrayWriter<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> ByteArrayWriter<'a> {
    fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.pos]
    }
}

impl<'a> ByteArrayWriter<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
}

impl<'a> uWrite for ByteArrayWriter<'a> {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let bytes = s.as_bytes();
        let remaining = self.buffer.len().saturating_sub(self.pos);
        let to_copy = bytes.len().min(remaining);

        self.buffer[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;

        Ok(())
    }
}

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

    let mut panic_msg: [u8; 64] = [0; 64];
    let mut panic_msg = ByteArrayWriter::new(&mut panic_msg);
    for _ in 0..NUM_CHARS {
        panic_msg.write_char(' ').ok();
    }
    panic_msg.write_str("PANIC! ").ok();

    if let Some(loc) = info.location() {
        uwrite!(
            &mut panic_msg,
            "{}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .ok();
    }
    for _ in 0..NUM_CHARS {
        panic_msg.write_char(' ').ok();
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
