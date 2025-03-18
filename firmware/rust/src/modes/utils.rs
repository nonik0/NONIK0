use super::Mode;
use crate::{Adc0, Context, Display, Event, Sigrow, Vref};

use avr_hal_generic::adc::{AdcChannel, ClockDivider};

/// Select the voltage reference for the ADC peripheral
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReferenceVoltage {
    /// Internal 1.1V? reference.
    Internal = 0b00,
    /// VDD as reference voltage.
    VDD = 0b01,
}

impl Default for ReferenceVoltage {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Resolution {
    _10bit = 0b0,
    _12bit = 0b1,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::_10bit
    }
}

/// Configuration for the ADC peripheral.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdcSettings {
    pub clock_divider: ClockDivider,
    pub ref_voltage: ReferenceVoltage,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Util {
    //I2CDetect,
    Temp,
    Vext,
    //Vref,
    // ADC settings
    //VrefSet,
    //Prescaler,
    //Resolution,
}

pub struct Utils {
    cur_util: Util,
    last_update: u16,

    adc0: Adc0,
    sigrow: Sigrow,
    vref: Vref,
    util_init: bool,

    buf: [u8; 8],
    //vrefsel: Vref::CTRLA::ADC0REFSEL_R,
}

impl Utils {
    pub fn new_with_adc(adc0: Adc0, sigrow: Sigrow, vref: Vref) -> Self {
        Utils {
            cur_util: Util::Vext,
            last_update: 0,
            adc0,
            sigrow,
            vref,
            util_init: false,
            buf: b"Vext:...",
            //vrefsel: Vref::CTRLA::ADC0REFSEL_R::default(),
        }
    }

    fn read_raw(&mut self, util: Util) -> Option<u16> {
        match (
            self.util_init,
            self.adc0.command.read().stconv().bit_is_set(),
        ) {
            // Other measurement ongoing
            (false, true) => None,
            // Set up for measurement and start
            (false, false) => {
                match util {
                    Util::Temp => self.configure_temp(),
                    Util::Vext => self.configure_vext(),
                    _ => {}
                }
                self.command.write(|w| w.stconv().set_bit());
                self.util_init = true;
                None
            }
            // Measurement ongoing
            (true, true) => None,
            // Measurement complete, get result and start again
            (true, false) => {
                self.command.write(|w| w.stconv().set_bit());
                Some(self.adc0.res.read().bits())
            },
        }
    }

    // Fclk_per = F_coreclock / 1 (prescaler/1 default)
    // Fclk_adc = Fclk_per / adc0.ctrlc.PRESC
    fn configure_temp(&mut self) {
        self.vref.ctrla.modify(|_, w| w.adc0refsel()._1v1());
        self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());
        self.adc0.ctrla.modify(|_, w| {
            //w.ressel().clear_bit();
            w.enable().set_bit()
        });
        self.adc0.ctrlc.modify(|_, w| {
            w.presc().div256(); // 32us x Fclk_adc_div0 = 1.25
            w.refsel().intref();
            w.sampcap().set_bit() // Vref >1 SAMPCAMP=1
        });
        self.adc0.ctrld.modify(|_, w| w.initdly().dly16()); // INITDLY>= 32us x Fclk_adc
        self.adc0.sampctrl.modify(|_, w| w.samplen().bits(2)); // SAMPLEN >= 32us x Fclk_adc
        self.adc0.muxpos.modify(|_, w| w.muxpos().tempsense());
    }

    fn configure_vext(&mut self) {
        // self.vref.ctrla.modify(|_, w| w.adc0refsel()._1v1());
        // self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());
        self.adc0.ctrla.write(|w| w.enable().set_bit());
        self.adc0.ctrlc.write(|w| {
            w.presc().div128();
            w.refsel().intref();
            w.sampcap().set_bit() // Vref >1 SAMPCAMP=1
        });
        // self.adc0.ctrld.reset();
        // self.adc0.sampctrl.reset();
        self.adc0.muxpos.modify(|_, w| w.muxpos().ain10()); // PB1/SDA
    }

    //fn format_temp(&mut self, display: &mut Display) {
        // if let Some(temp) = self.read_raw() {
        //     //let temp_f = ((temp as i32 - 273) * 9 / 5 + 32) as u16;
        //     display.print_u32(temp as u32).unwrap();
        // } else {
        //     display.print_ascii_bytes(b"Temp: ...").unwrap();
        // }
        //let sigrow_offset = self.sigrow.tempsense1.read().bits() as i16; // Read signed value from signature row
        //let sigrow_gain = self.sigrow.tempsense0.read().bits() as u16;   // Read unsigned value from signature row
        //let adc_reading = self.adc0.res.read().bits() as u16;           // ADC conversion result with 1.1 V internal reference

        // let mut raw_temp = (adc_reading as i32) - (sigrow_offset as i32); // Perform subtraction with proper casting
        // raw_temp *= sigrow_gain as i32;                                  // Multiply with gain
        // raw_temp += 0x80;                                                // Add 1/2 to get correct rounding
        // raw_temp >>= 8;                                                  // Divide result to get Kelvin
        // let temperature_in_k = raw_temp as u16;                          // Cast back to u16

        // let temp_f = ((raw_temp as i32 - 273) * 9 / 5 + 32) as u16;
        // display.print_ascii_bytes(format!("Temp: {:.1}\x98F", temp_f).as_bytes()).unwrap();
    //}
}

impl Mode for Utils {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    self.adc0.ctrla.write(|w| w.enable().set_bit());
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    update = true;
                    self.util_init = false;
                    let next_util = match self.cur_util {
                        // Util::I2CDetect => {
                        //     self.buf = *b"Temp:?\x98F";
                        //     Util::Temp
                        // },
                        Util::Temp => {
                            self.buf = *b"Vext:...";
                            Util::Vext
                        },
                        Util::Vext => {
                            self.buf = *b"Temp:?\x98F";
                            Util::Temp
                            //self.buf = *b"Vref:?.?";
                            //Util::Vref
                        },
                        // Util::Vref => {
                        //     self.buf = *b"RSet:?.?";
                        //     Util::VrefSet
                        // },
                        // Util::VrefSet => {
                        //     self.buf = *b"Prsc:???";
                        //     Util::Prescaler
                        // },
                        // Util::Prescaler => {
                        //     self.buf = *b"Res: ???";
                        //     Util::Resolution
                        // },
                        // Util::Resolution => {
                        //     self.buf = *b"I2C: ???";
                        //     Util::I2CDetect
                        // },
                    };
                    self.cur_util = next_util;
                }
                // Event::LeftReleased => match self.cur_util {
                //     Util::VrefSet => {}
                //     _ => {}
                // },
                // Event::RightReleased => match self.cur_util {
                //     Util::VrefSet => {}
                //     _ => {}
                // },
                _ => {}
            }
        }

        if let Some(reading) = self.read_raw(self.cur_util) {
            //update = true;
            display.print_u32(reading as u32).unwrap();
        }

        if update {
            display.print_ascii_bytes(&self.buf).unwrap();
        }
    }
}
