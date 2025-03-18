use super::Mode;
use crate::{Adc0, Context, Display, Event, Sigrow, Vref, NUM_CHARS};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Util {
    ReadTemp,
    ReadVext,
    SetVref,
    SetPrescaler,
    SetResolution,
}

pub struct Utils {
    cur_util: Util,
    last_update: u16,

    adc_settings: AdcSettings,
    adc0: Adc0,
    sigrow: Sigrow,
    vref: Vref,

    util_init: bool,
    show_raw: bool,
}

impl Utils {
    pub fn new_with_adc(adc0: Adc0, sigrow: Sigrow, vref: Vref) -> Self {
        Utils {
            cur_util: Util::ReadVext,
            last_update: 0,

            adc_settings: AdcSettings::default(),
            adc0,
            sigrow,
            vref,

            util_init: false,
            show_raw: false,
        }
    }

    fn format_util(&self) -> &[u8; NUM_CHARS] {
        match self.cur_util {
            Util::ReadTemp => b"Tf:....\x98",
            Util::ReadVext => b"Ve:....V",
            Util::SetVref => match self.adc_settings.ref_voltage {
                ReferenceVoltage::IntRef0_55V => b"Vr:0.55V",
                ReferenceVoltage::IntRef1_1V => b"Vr: 1.1V",
                ReferenceVoltage::IntRef2_5V => b"Vr: 2.5V",
                ReferenceVoltage::IntRef4_34V => b"Vr:4.34V",
                ReferenceVoltage::IntRef1_5V => b"Vr: 1.5V",
                ReferenceVoltage::Vdd => b"Vr:  Vdd",
            },
            Util::SetPrescaler => match self.adc_settings.clock_divider {
                ClockDivider::Factor2 => b"Div:   2",
                ClockDivider::Factor4 => b"Div:   4",
                ClockDivider::Factor8 => b"Div:   8",
                ClockDivider::Factor16 => b"Div:  16",
                ClockDivider::Factor32 => b"Div:  32",
                ClockDivider::Factor64 => b"Div:  64",
                ClockDivider::Factor128 => b"Div: 128",
                ClockDivider::Factor256 => b"Div: 256",
            },
            Util::SetResolution => match self.adc_settings.resolution {
                Resolution::_10bit => b"Res: 10b",
                Resolution::_8bit => b"Res:  8b",
            },
        }
    }

    fn format_raw(&mut self, raw: u16, buf: &mut [u8; NUM_CHARS]) {
        const PREFIX_LEN: usize = 3;

        let (prefix, mut reading_val, suffix) = match self.cur_util {
            Util::ReadTemp => {
                if self.show_raw {
                    (b"Tf:", raw, None)
                } else {
                    (b"Tf:", self.temp_from_raw(raw), Some(b'\x98'))
                }
            }
            Util::ReadVext => {
                if self.show_raw {
                    (b"Ve:", raw, None)
                } else {
                    (b"Ve:", self.voltage_from_raw(raw), Some(b'V'))
                }
            }
            _ => return,
        };

        // copy prefix and suffix to buf (i.e. "Ve:____V")
        buf[..PREFIX_LEN].copy_from_slice(prefix);
        if let Some(suffix) = suffix {
            buf[NUM_CHARS - 1] = suffix;
        }

        let reading_len = if suffix.is_some() {
            NUM_CHARS - PREFIX_LEN - 1
        } else {
            NUM_CHARS - PREFIX_LEN
        };
        buf[PREFIX_LEN + reading_len - 1] = b'0';
        for index in (PREFIX_LEN..PREFIX_LEN + reading_len).rev() {
            if reading_val > 0 {
                buf[index] = b'0' + (reading_val % 10) as u8;
                reading_val /= 10;
            } else {
                buf[index] = b' ';
            }
        }
    }

    fn read_raw(&mut self) -> Option<u16> {
        match (
            self.util_init,
            self.adc0.command.read().stconv().bit_is_set(),
        ) {
            // Other measurement ongoing
            (false, true) => None,
            // Set up for measurement and start
            (false, false) => {
                match self.cur_util {
                    Util::ReadTemp => self.configure_temp(),
                    Util::ReadVext => self.configure_vext(),
                    _ => return None,
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

    fn voltage_from_raw(&self, raw: u16) -> u16 {
        0
    }

    fn temp_from_raw(&mut self, raw: u16) -> u16 {
        let sigrow_offset = self.sigrow.tempsense1.read().bits() as i16; // Read signed value from signature row
        let sigrow_gain = self.sigrow.tempsense0.read().bits() as u16; // Read unsigned value from signature row
        let adc_reading = self.adc0.res.read().bits() as u16; // ADC conversion result with 1.1 V internal reference

        let mut raw_temp = (adc_reading as i32) - (sigrow_offset as i32); // Perform subtraction with proper casting
        raw_temp *= sigrow_gain as i32; // Multiply with gain
        raw_temp += 0x80; // Add 1/2 to get correct rounding
        raw_temp >>= 8; // Divide result to get Kelvin
                        //let temperature_in_k = raw_temp as u16; // Cast back to u16
        let temp_f = ((raw_temp as i32 - 273) * 9 / 5 + 32) as u16;
        temp_f
    }

    fn configure_temp(&mut self) {
        // Fclk_per = F_coreclock / 1 (prescaler/1 default)
        // Fclk_adc = Fclk_per / adc0.ctrlc.PRESC
        self.vref.ctrla.modify(|_, w| w.adc0refsel()._1v1());
        self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());
        self.adc0.ctrla.write(|w| w.enable().set_bit());
        self.adc0.ctrlc.write(|w| {
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

        self.adc0.ctrla.write(|w| {
            match self.adc_settings.resolution {
                Resolution::_10bit => w.ressel()._10bit(),
                Resolution::_8bit => w.ressel()._8bit(),
            };
            w.enable().set_bit()
        });
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
        self.adc0.ctrld.reset();
        self.adc0.sampctrl.reset();
        self.adc0.muxpos.modify(|_, w| w.muxpos().ain10()); // PB1/SDA
    }

    fn increment_util_setting(&mut self) -> bool {
        match self.cur_util {
            Util::SetPrescaler => {
                self.adc_settings.clock_divider = match self.adc_settings.clock_divider {
                    ClockDivider::Factor2 => ClockDivider::Factor4,
                    ClockDivider::Factor4 => ClockDivider::Factor8,
                    ClockDivider::Factor8 => ClockDivider::Factor16,
                    ClockDivider::Factor16 => ClockDivider::Factor32,
                    ClockDivider::Factor32 => ClockDivider::Factor64,
                    ClockDivider::Factor64 => ClockDivider::Factor128,
                    ClockDivider::Factor128 => ClockDivider::Factor256,
                    ClockDivider::Factor256 => ClockDivider::Factor256, // No wrap around
                };
            }
            Util::SetVref => {
                self.adc_settings.ref_voltage = match self.adc_settings.ref_voltage {
                    ReferenceVoltage::IntRef0_55V => ReferenceVoltage::IntRef1_1V,
                    ReferenceVoltage::IntRef1_1V => ReferenceVoltage::IntRef2_5V,
                    ReferenceVoltage::IntRef2_5V => ReferenceVoltage::IntRef4_34V,
                    ReferenceVoltage::IntRef4_34V => ReferenceVoltage::IntRef1_5V,
                    ReferenceVoltage::IntRef1_5V => ReferenceVoltage::Vdd,
                    ReferenceVoltage::Vdd => ReferenceVoltage::Vdd, // No wrap around
                };
            }
            Util::SetResolution => {
                self.adc_settings.resolution = match self.adc_settings.resolution {
                    Resolution::_10bit => Resolution::_8bit,
                    Resolution::_8bit => Resolution::_8bit, // No wrap around
                };
            }
            Util::ReadTemp | Util::ReadVext => {
                self.show_raw = !self.show_raw;
                return false;
            }
            _ => return false,
        };

        true
    }

    fn decrement_util_setting(&mut self) -> bool {
        match self.cur_util {
            Util::SetPrescaler => {
                self.adc_settings.clock_divider = match self.adc_settings.clock_divider {
                    ClockDivider::Factor256 => ClockDivider::Factor128,
                    ClockDivider::Factor128 => ClockDivider::Factor64,
                    ClockDivider::Factor64 => ClockDivider::Factor32,
                    ClockDivider::Factor32 => ClockDivider::Factor16,
                    ClockDivider::Factor16 => ClockDivider::Factor8,
                    ClockDivider::Factor8 => ClockDivider::Factor4,
                    ClockDivider::Factor4 => ClockDivider::Factor2,
                    ClockDivider::Factor2 => ClockDivider::Factor2, // No wrap around
                };
            }
            Util::SetVref => {
                self.adc_settings.ref_voltage = match self.adc_settings.ref_voltage {
                    ReferenceVoltage::Vdd => ReferenceVoltage::IntRef1_5V,
                    ReferenceVoltage::IntRef1_5V => ReferenceVoltage::IntRef4_34V,
                    ReferenceVoltage::IntRef4_34V => ReferenceVoltage::IntRef2_5V,
                    ReferenceVoltage::IntRef2_5V => ReferenceVoltage::IntRef1_1V,
                    ReferenceVoltage::IntRef1_1V => ReferenceVoltage::IntRef0_55V,
                    ReferenceVoltage::IntRef0_55V => ReferenceVoltage::IntRef0_55V, // No wrap around
                };
            }
            Util::SetResolution => {
                self.adc_settings.resolution = match self.adc_settings.resolution {
                    Resolution::_8bit => Resolution::_10bit,
                    Resolution::_10bit => Resolution::_10bit, // No wrap around
                };
            }
            Util::ReadTemp | Util::ReadVext => {
                self.show_raw = !self.show_raw;
                return false;
            }
            _ => return false,
        };

        true
    }
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
                    let next_util = match self.cur_util {
                        Util::ReadTemp => Util::ReadVext,
                        Util::ReadVext => Util::SetVref,
                        Util::SetVref => Util::SetPrescaler,
                        Util::SetPrescaler => Util::SetResolution,
                        Util::SetResolution => Util::ReadTemp,
                    };
                    self.cur_util = next_util;
                    self.util_init = false;
                    update = true;
                }
                Event::LeftReleased => {
                    update = self.decrement_util_setting();
                }
                Event::RightReleased => {
                    update = self.increment_util_setting();
                }
                _ => {}
            }
        }

        if update {
            let util_buf = self.format_util();
            display.print_ascii_bytes(util_buf).unwrap();
        } else if let Some(reading) = self.read_raw() {
            let mut reading_buf = [0; NUM_CHARS];
            self.format_raw(reading, &mut reading_buf);
            display.print_ascii_bytes(&reading_buf).unwrap();
        }
    }
}

//
// ADC stuff, eventually move to avr_hal impl
//

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
            ref_voltage: ReferenceVoltage::IntRef2_5V,
            resolution: Resolution::default(),
            samp_cap: true, // Vref default 1.1V > 1.0V
                            //samples: 0,
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
