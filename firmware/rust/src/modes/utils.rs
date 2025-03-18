use super::Mode;
use crate::{Adc0, Context, Display, Event, Sigrow, Vref};

/// Configuration for the ADC peripheral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdcSettings {
    pub clock_divider: ClockDivider,
    pub ref_voltage: ReferenceVoltage,
    pub resolution: Resolution,
    pub samp_cap: bool,
    //pub samples: u8,
}

impl Default for AdcSettings {
    fn default() -> Self {
        AdcSettings {
            clock_divider: ClockDivider::default(),
            ref_voltage: ReferenceVoltage::default(),
            resolution: Resolution::default(),
            samp_cap: true, // Vref default 1.1V > 1.0V
                            //samples: 0,
        }
    }
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

    adc_settings: AdcSettings,
    adc0: Adc0,
    sigrow: Sigrow,
    vref: Vref,

    util_init: bool,
    buf: [u8; 8],
}

impl Utils {
    pub fn new_with_adc(adc0: Adc0, sigrow: Sigrow, vref: Vref) -> Self {
        Utils {
            cur_util: Util::Vext,
            last_update: 0,

            adc_settings: AdcSettings::default(),
            adc0,
            sigrow,
            vref,

            util_init: false,
            buf: *b"Vext:...",
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
                self.adc0.command.write(|w| w.stconv().set_bit());
                self.util_init = true;
                None
            }
            // Measurement ongoing
            (true, true) => None,
            // Measurement complete, get result and start again
            (true, false) => {
                self.adc0.command.write(|w| w.stconv().set_bit());
                Some(self.adc0.res.read().bits())
            }
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
        self.vref
            .ctrla
            .modify(|_, w| match self.adc_settings.ref_voltage {
                ReferenceVoltage::IntRef0_55V => w.adc0refsel()._0v55(),
                ReferenceVoltage::IntRef1_1V => w.adc0refsel()._1v1(),
                ReferenceVoltage::IntRef2_5V => w.adc0refsel()._2v5(),
                ReferenceVoltage::IntRef4_34V => w.adc0refsel()._4v34(),
                ReferenceVoltage::IntRef1_5V => w.adc0refsel()._1v5(),
                _ => w,
            });
        //self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());
        
        self.adc0.ctrla.write(|w| w.enable().set_bit());
        self.adc0.ctrlc.write(|w| {
            match self.adc_settings.ref_voltage {
                ReferenceVoltage::Vdd => w.refsel().vddref(),
                _ => w.refsel().intref(),
            };
            match self.adc_settings.clock_divider {
                ClockDivider::Factor2 => w.presc().div2(),
                ClockDivider::Factor4 => w.presc().div4(),
                ClockDivider::Factor8 => w.presc().div8(),
                ClockDivider::Factor16 => w.presc().div16(),
                ClockDivider::Factor32 => w.presc().div32(),
                ClockDivider::Factor64 => w.presc().div64(),
                ClockDivider::Factor128 => w.presc().div128(),
                ClockDivider::Factor256 => w.presc().div256(),
            };
            w.sampcap().bit(self.adc_settings.samp_cap)
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
                    self.adc0.ctrla.write(|w| w.enable().clear_bit());
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
                        }
                        Util::Vext => {
                            self.buf = *b"Temp:?\x98F";
                            Util::Temp
                            //self.buf = *b"Vref:?.?";
                            //Util::Vref
                        } // Util::Vref => {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClockDivider {
    Factor2 = 0,
    Factor4 = 1,
    Factor8 = 2,
    Factor16 = 3,
    Factor32 = 4,
    Factor64 = 5,
    /// (default)
    Factor128 = 6,
    Factor256 = 7,
}

impl Default for ClockDivider {
    fn default() -> Self {
        Self::Factor128
    }
}

/// Select the voltage reference for the ADC peripheral, binary values for ADC0REFSEL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReferenceVoltage {
    IntRef0_55V = 0,
    IntRef1_1V = 1,
    IntRef2_5V = 2,
    IntRef4_34V = 3,
    IntRef1_5V = 4,
    Vdd = 5,
}

impl Default for ReferenceVoltage {
    fn default() -> Self {
        Self::IntRef1_1V
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Resolution {
    _10bit = 0b0,
    _8bit = 0b1,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::_10bit
    }
}
