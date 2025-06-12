use crate::CoreClock;
use avrxmega_hal::{
    clock::Clock,
    port::{mode::Output, PA5},
};
use core::cell::RefCell;

type Pin = avrxmega_hal::port::Pin<Output, PA5>;
type Timer = avrxmega_hal::pac::TCB0;

static TONE_STATE: avr_device::interrupt::Mutex<RefCell<Option<ToneState>>> =
    avr_device::interrupt::Mutex::new(RefCell::new(None));

struct ToneState {
    timer: Timer,
    output_pin: Pin,

    cycles_left: u8,
    cycles_per_toggle: u8,
    toggles_left: Option<u64>,
}

impl ToneState {
    pub fn enable(
        &mut self,
        clock_divider: u8,
        division_factor: u8,
        compare_value: u16,
        toggle_count: Option<u64>,
    ) {
        self.output_pin.set_low();
        self.toggles_left = toggle_count;
        self.cycles_per_toggle = 1 << division_factor;
        self.cycles_left = self.cycles_per_toggle;

        self.timer.ctrla().write(|w| match clock_divider {
            1 => w.clksel().clkdiv1(),
            _ => w.clksel().clkdiv2(),
        });
        self.timer.ccmp().write(|w| w.set(compare_value as u16));
        self.timer.ctrlb().write(|w| w.cntmode().int());
        self.timer.cnt().write(|w| w.set(0));
        self.timer.intctrl().write(|w| w.capt().set_bit());
        self.timer.ctrla().modify(|_, w| w.enable().set_bit());
    }

    pub fn disable(&mut self) {
        self.output_pin.set_low();
        self.toggles_left = None;

        self.timer.ctrla().modify(|_, w| w.enable().clear_bit());
        self.timer.intctrl().write(|w| w.capt().clear_bit());
        self.timer.intflags().write(|w| w.capt().set_bit());
    }

    pub fn int_tick(&mut self) {
        self.cycles_left -= 1;
        if self.cycles_left == 0 {
            self.cycles_left = self.cycles_per_toggle;
            self.output_pin.toggle();

            if let Some(mut toggles_left) = self.toggles_left {
                toggles_left -= 1;
                if toggles_left == 0 {
                    self.disable();
                } else {
                    self.toggles_left = Some(toggles_left);
                }
            }
        }

        self.timer.intflags().write(|w| w.capt().set_bit());
    }
}

pub struct Tone {}

impl Tone {
    pub fn new(timer: Timer, output_pin: Pin) -> Self {
        let state = ToneState {
            timer,
            output_pin,

            toggles_left: None,
            cycles_left: 0,
            cycles_per_toggle: 0,
        };

        avr_device::interrupt::free(|cs| {
            let mut state_opt = TONE_STATE.borrow(cs).borrow_mut();
            *state_opt = Some(state);
        });

        // TODO: should caller/owner be responsible for enabling interrupts?
        unsafe {
            avr_device::interrupt::enable();
        }

        Self {}
    }

    pub fn tone(&mut self, frequency: u32, duration: u32) {
        if frequency == 0 {
            self.no_tone();
            return;
        }

        let toggle_count = if duration > 0 {
            Some(frequency as u64 * duration as u64 / 500)
        } else {
            None
        };

        let mut division_factor = 1u8;
        let mut compare_value = (CoreClock::FREQ / frequency) >> 1;

        let clock_divider = if compare_value < 0x0001_0000 {
            1
        } else {
            division_factor -= 1;
            2
        };

        while compare_value > 0x0001_0000 && division_factor < 6 {
            compare_value >>= 1;
            division_factor += 1;
        }

        compare_value -= 1;
        if compare_value > 0x0000_FFFF {
            // over 9000!!!
            compare_value = 0x0000_FFFF;
        }

        // update static state
        avr_device::interrupt::free(|cs| {
            let state_opt_refcell = TONE_STATE.borrow(cs);
            let mut state_opt = state_opt_refcell.borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.enable(
                clock_divider,
                division_factor,
                compare_value as u16,
                toggle_count,
            );
        });
    }

    pub fn no_tone(&mut self) {
        avr_device::interrupt::free(|cs| {
            let state_opt_refcell = TONE_STATE.borrow(cs);
            let mut state_opt = state_opt_refcell.borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.disable();
        });
    }
}

#[avr_device::interrupt(attiny1604)]
fn TCB0_INT() {
    avr_device::interrupt::free(|cs| {
        let mut state_opt = TONE_STATE.borrow(cs).borrow_mut();
        let state = state_opt.as_mut().unwrap(); // unwrap is safe here bc interrupt won't be enabled if state is None

        state.int_tick();
    })
}
