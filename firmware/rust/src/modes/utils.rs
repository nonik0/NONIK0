use super::Mode;
use crate::{Adc0, Context, Display, Event, Sigrow, Vref, NUM_CHARS};

// TODO: setting to submenu
// tap left to change raw/converted
// tap right to change reading
// hold left to go back to main menu
// hold right to go into settings
// once in settings hold right to go to next setting, hold left to go back to readings
#[derive(Debug, Clone, Copy, PartialEq)]
enum Util {
    ReadTemp,
    ReadVext,
    ReadVint,
    SetResolution,
    SetSampleNumber,
    SetSampCap,
    SetRefVoltage,
    SetPrescaler,
    SetInitDelay,
    //SetAsdv,
    SetSampleDelay,
    SetSampleLength,
}

// helper function to format an unsigned integer with a prefix and suffix value
fn format_uint(buf: &mut [u8], prefix: &[u8], value: u16, decimal_digits: u16, suffix: Option<&[u8]>) {
    let num_chars = buf.len();
    let prefix_len = prefix.len();

    // copy prefix to buf (i.e. "Ve:____")
    buf[..prefix_len].copy_from_slice(prefix);

    // copy suffix to buf if provided
    if let Some(suffix) = suffix {
        let suffix_len = suffix.len();
        buf[num_chars - suffix_len..].copy_from_slice(suffix);
    }

    // now copy the value by digit into buf from the right
    let mut need_decimal = decimal_digits > 0;
    let mut digits_in_buf = 0;
    let mut value = value;
    let value_len = if let Some(suffix) = suffix {
        num_chars - prefix_len - suffix.len()
    } else {
        num_chars - prefix_len
    };
    for index in (prefix_len..prefix_len + value_len).rev() {
        if need_decimal && digits_in_buf == decimal_digits {
            buf[index] = b'.';
            need_decimal = false;
        } else if value > 0 {
            buf[index] = b'0' + (value % 10) as u8;
            value /= 10;
            digits_in_buf += 1;
        } else {
            buf[index] = if digits_in_buf < (1 + decimal_digits) {
                digits_in_buf += 1;
                b'0'
            } else {
                b' '
            };
        }
    }
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
    buf: [u8; NUM_CHARS],
}

impl Utils {
    const DECIMAL_PRECISION: u16 = 2; // X.YY
    const VOLTAGE_ADC_SETTINGS: AdcSettings = AdcSettings {
        resolution: Resolution::_10bit,
        sample_number: SampleNumber::Acc16,
        samp_cap: true,
        ref_voltage: ReferenceVoltage::VRef2_5V,
        clock_divider: ClockDivider::Factor256,
        init_delay: DelayCycles::Delay0,
        asdv: false,
        sample_delay: 0,
        sample_length: 0,
    };
    const TEMP_ADC_SETTINGS: AdcSettings = AdcSettings {
        resolution: Resolution::_10bit,
        sample_number: SampleNumber::Acc1,
        samp_cap: true,
        ref_voltage: ReferenceVoltage::VRef1_1V,
        clock_divider: ClockDivider::Factor256,
        init_delay: DelayCycles::Delay32,
        asdv: false,
        sample_delay: 0,
        sample_length: 4,
    };

    pub fn new_with_adc(adc0: Adc0, sigrow: Sigrow, vref: Vref) -> Self {
        Utils {
            cur_util: Util::ReadTemp,
            last_update: 0,

            adc_settings: Self::VOLTAGE_ADC_SETTINGS,
            adc0,
            sigrow,
            vref,

            util_init: false,
            show_raw: false,
            buf: [0; NUM_CHARS],
        }
    }

    fn format_util(&mut self) -> &[u8; NUM_CHARS] {
        match self.cur_util {
            Util::ReadTemp => b"Tf:....\x98",
            Util::ReadVext => b"Ve:....V",
            Util::ReadVint => b"Vi:....V",
            Util::SetResolution => match self.adc_settings.resolution {
                Resolution::_10bit => b"Res: 10b",
                Resolution::_8bit => b"Res:  8b",
            },
            Util::SetSampleNumber => match self.adc_settings.sample_number {
                SampleNumber::Acc1 => b"Snum:  1",
                SampleNumber::Acc2 => b"Snum:  2",
                SampleNumber::Acc4 => b"Snum:  4",
                SampleNumber::Acc8 => b"Snum:  8",
                SampleNumber::Acc16 => b"Snum: 16",
                SampleNumber::Acc32 => b"Snum: 32",
                SampleNumber::Acc64 => b"Snum: 64",
            },
            Util::SetSampCap => {
                if self.adc_settings.samp_cap {
                    b"Scap:yes"
                } else {
                    b"Scap: no"
                }
            }
            Util::SetRefVoltage => match self.adc_settings.ref_voltage {
                ReferenceVoltage::VRef0_55V => b"Vr:0.55V",
                ReferenceVoltage::VRef1_1V => b"Vr: 1.1V",
                ReferenceVoltage::VRef1_5V => b"Vr: 1.5V",
                ReferenceVoltage::VRef2_5V => b"Vr: 2.5V",
                ReferenceVoltage::VRef4_34V => b"Vr:4.34V",
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
            Util::SetInitDelay => match self.adc_settings.init_delay {
                DelayCycles::Delay0 => b"Idly:  0",
                DelayCycles::Delay16 => b"Idly: 16",
                DelayCycles::Delay32 => b"Idly: 32",
                DelayCycles::Delay64 => b"Idly: 64",
                DelayCycles::Delay128 => b"Idly:128",
                DelayCycles::Delay256 => b"Idly:256",
            },
            // Util::SetAsdv => {
            //     if self.adc_settings.asdv {
            //         b"As: True"
            //     } else {
            //         b"As:False"
            //     }
            // }
            Util::SetSampleDelay => {
                format_uint(
                    &mut self.buf,
                    b"Sdly:",
                    self.adc_settings.sample_delay as u16,
                    0,
                    None,
                );
                &self.buf
            }
            Util::SetSampleLength => {
                format_uint(
                    &mut self.buf,
                    b"Slen:",
                    self.adc_settings.sample_length as u16,
                    0,
                    None,
                );
                &self.buf
            }
        }
    }

    fn format_raw(&mut self, raw: u16, buf: &mut [u8; NUM_CHARS]) {
        let (prefix, value, decimals, suffix) = match self.cur_util {
            Util::ReadTemp => {
                if self.show_raw {
                    (b"Tf:", raw, 0, None)
                } else {
                    (
                        b"Tf:",
                        self.temp_from_raw(raw),
                        0,
                        Some(b"\x98C".as_slice()),
                    )
                }
            }
            Util::ReadVext => {
                if self.show_raw {
                    (b"Ve:", raw, 0, None)
                } else {
                    (
                        b"Ve:",
                        self.voltage_from_raw(raw),
                        Self::DECIMAL_PRECISION,
                        Some(b"V".as_slice()),
                    )
                }
            }
            Util::ReadVint => {
                if self.show_raw {
                    (b"Vi:", raw, 0, None)
                } else {
                    (
                        b"Vi:",
                        self.voltage_from_raw(raw),
                        Self::DECIMAL_PRECISION,
                        Some(b"V".as_slice()),
                    )
                }
            }
            _ => return,
        };

        format_uint(buf, prefix, value, decimals, suffix);
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
                let (adc_settings, channel) = match self.cur_util {
                    Util::ReadTemp => (
                        Self::TEMP_ADC_SETTINGS,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::TEMPSENSE,
                    ),
                    Util::ReadVext => (
                        self.adc_settings,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::AIN10,
                    ),
                    Util::ReadVint => (
                        self.adc_settings,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::INTREF,
                    ),                    
                    // TODO Read Vref
                    _ => return None,
                };
                self.apply_adc_settings(adc_settings);
                self.adc0.muxpos.write(|w| w.muxpos().variant(channel));
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
        // RAW_MAX = 2^RESOLUTION-1
        // RAW_RES = RAW_MAX * (Vin/Vref)
        // Vin = (RAW_RES/RAW_MAX) * Vref
        // Vin = ((RAW_RES * Vref) / RAW_MAX) (integer arithmetic order
        let raw = raw as u32;
        let vrefe5: u32 = match self.adc_settings.ref_voltage {
            ReferenceVoltage::VRef0_55V => 55000,  // 0.55 x 10^5
            ReferenceVoltage::VRef1_1V => 110000,  // 1.1 x 10^5
            ReferenceVoltage::VRef1_5V => 150000,  // 1.5 x 10^5
            ReferenceVoltage::VRef2_5V => 250000,  // 2.5 x 10^5
            ReferenceVoltage::VRef4_34V => 434000, // 4.34 x 10^5
            ReferenceVoltage::Vdd => 360000,       // ~3.6V nominal with LIR2032, refine later
        };
        let raw_max: u32 = match self.adc_settings.resolution {
            Resolution::_10bit => 1023, // 2^10 - 1
            Resolution::_8bit => 255,   // 2^8 - 1
        };
        let precision_divisor = 10u32.pow(5 - Self::DECIMAL_PRECISION as u32);
        let accumulation_divisor = match self.adc_settings.sample_number {
            SampleNumber::Acc1 => 1,
            SampleNumber::Acc2 => 2,
            SampleNumber::Acc4 => 4,
            SampleNumber::Acc8 => 8,
            SampleNumber::Acc16 => 16,
            SampleNumber::Acc32 => 32,
            SampleNumber::Acc64 => 64,
        };

        ((((raw * vrefe5) / raw_max) / precision_divisor) / accumulation_divisor) as u16
    }

    fn temp_from_raw(&mut self, raw: u16) -> u16 {
        let sigrow_offset = self.sigrow.tempsense1.read().bits() as i8;
        let sigrow_gain = self.sigrow.tempsense0.read().bits() as u8;

        let mut temp: u32 = ((raw as i32) - (sigrow_offset as i32)) as u32;
        temp = (temp as i32 * sigrow_gain as i32) as u32;
        temp += 0x80;
        temp >>= 8;
        let temp_k = temp as u16;
        let temp_c = temp_k.saturating_sub(273); // TODO <0
        temp_c
    }

    fn apply_adc_settings(&mut self, settings: AdcSettings) {
        self.vref.ctrla.modify(|_, w| match settings.ref_voltage {
            ReferenceVoltage::VRef0_55V => w.adc0refsel()._0v55(),
            ReferenceVoltage::VRef1_1V => w.adc0refsel()._1v1(),
            ReferenceVoltage::VRef2_5V => w.adc0refsel()._2v5(),
            ReferenceVoltage::VRef4_34V => w.adc0refsel()._4v34(),
            ReferenceVoltage::VRef1_5V => w.adc0refsel()._1v5(),
            _ => w,
        });
        //self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());

        self.adc0.ctrla.write(|w| {
            match settings.resolution {
                Resolution::_10bit => w.ressel()._10bit(),
                Resolution::_8bit => w.ressel()._8bit(),
            };
            w.enable().set_bit()
        });
        self.adc0.ctrlb.write(|w| match settings.sample_number {
            SampleNumber::Acc1 => w.sampnum().acc1(),
            SampleNumber::Acc2 => w.sampnum().acc2(),
            SampleNumber::Acc4 => w.sampnum().acc4(),
            SampleNumber::Acc8 => w.sampnum().acc8(),
            SampleNumber::Acc16 => w.sampnum().acc16(),
            SampleNumber::Acc32 => w.sampnum().acc32(),
            SampleNumber::Acc64 => w.sampnum().acc64(),
        });
        self.adc0.ctrlc.write(|w| {
            w.sampcap().bit(settings.samp_cap);
            match settings.ref_voltage {
                ReferenceVoltage::Vdd => w.refsel().vddref(),
                _ => w.refsel().intref(), // internal Vref
            };
            match settings.clock_divider {
                ClockDivider::Factor2 => w.presc().div2(),
                ClockDivider::Factor4 => w.presc().div4(),
                ClockDivider::Factor8 => w.presc().div8(),
                ClockDivider::Factor16 => w.presc().div16(),
                ClockDivider::Factor32 => w.presc().div32(),
                ClockDivider::Factor64 => w.presc().div64(),
                ClockDivider::Factor128 => w.presc().div128(),
                ClockDivider::Factor256 => w.presc().div256(),
            }
        });
        self.adc0.ctrld.write(|w| {
            match settings.init_delay {
                DelayCycles::Delay0 => w.initdly().dly0(),
                DelayCycles::Delay16 => w.initdly().dly16(),
                DelayCycles::Delay32 => w.initdly().dly32(),
                DelayCycles::Delay64 => w.initdly().dly64(),
                DelayCycles::Delay128 => w.initdly().dly128(),
                DelayCycles::Delay256 => w.initdly().dly256(),
            };
            w.asdv().bit(settings.asdv);
            w.sampdly().bits(settings.sample_delay) // bits() concats if too largeS
        });
        self.adc0
            .sampctrl
            .write(|w| w.samplen().bits(settings.sample_length));
    }

    fn decrement_util_setting(&mut self) -> bool {
        match self.cur_util {
            Util::ReadTemp | Util::ReadVext | Util::ReadVint => {
                self.show_raw = !self.show_raw;
                return false;
            }
            Util::SetResolution => {
                self.adc_settings.resolution = match self.adc_settings.resolution {
                    Resolution::_10bit => Resolution::_8bit,
                    Resolution::_8bit => Resolution::_8bit, // No wrap around
                };
            }
            Util::SetSampleNumber => {
                self.adc_settings.sample_number = match self.adc_settings.sample_number {
                    SampleNumber::Acc64 => SampleNumber::Acc32,
                    SampleNumber::Acc32 => SampleNumber::Acc16,
                    SampleNumber::Acc16 => SampleNumber::Acc8,
                    SampleNumber::Acc8 => SampleNumber::Acc4,
                    SampleNumber::Acc4 => SampleNumber::Acc2,
                    SampleNumber::Acc2 => SampleNumber::Acc1,
                    SampleNumber::Acc1 => SampleNumber::Acc1, // No wrap around
                };
            }
            Util::SetSampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            Util::SetRefVoltage => {
                self.adc_settings.ref_voltage = match self.adc_settings.ref_voltage {
                    ReferenceVoltage::Vdd => ReferenceVoltage::VRef4_34V,
                    ReferenceVoltage::VRef4_34V => ReferenceVoltage::VRef2_5V,
                    ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef1_5V,
                    ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef1_1V,
                    ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef0_55V,
                    ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef0_55V, // No wrap around
                };
            }
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
            Util::SetInitDelay => {
                self.adc_settings.init_delay = match self.adc_settings.init_delay {
                    DelayCycles::Delay256 => DelayCycles::Delay128,
                    DelayCycles::Delay128 => DelayCycles::Delay64,
                    DelayCycles::Delay64 => DelayCycles::Delay32,
                    DelayCycles::Delay32 => DelayCycles::Delay16,
                    DelayCycles::Delay16 => DelayCycles::Delay0,
                    DelayCycles::Delay0 => DelayCycles::Delay0, // No wrap around
                };
            }
            // Util::SetAsdv => {
            //     self.adc_settings.asdv = !self.adc_settings.asdv;
            // }
            Util::SetSampleDelay => {
                self.adc_settings.sample_delay = self.adc_settings.sample_delay.saturating_sub(1);
            }
            Util::SetSampleLength => {
                self.adc_settings.sample_length = self.adc_settings.sample_length.saturating_sub(1);
            }
        };

        true
    }

    fn increment_util_setting(&mut self) -> bool {
        match self.cur_util {
            Util::ReadTemp | Util::ReadVext | Util::ReadVint => {
                self.show_raw = !self.show_raw;
                return false;
            }
            Util::SetResolution => {
                self.adc_settings.resolution = match self.adc_settings.resolution {
                    Resolution::_8bit => Resolution::_10bit,
                    Resolution::_10bit => Resolution::_10bit, // No wrap around
                };
            }
            Util::SetSampleNumber => {
                self.adc_settings.sample_number = match self.adc_settings.sample_number {
                    SampleNumber::Acc1 => SampleNumber::Acc2,
                    SampleNumber::Acc2 => SampleNumber::Acc4,
                    SampleNumber::Acc4 => SampleNumber::Acc8,
                    SampleNumber::Acc8 => SampleNumber::Acc16,
                    SampleNumber::Acc16 => SampleNumber::Acc32,
                    SampleNumber::Acc32 => SampleNumber::Acc64,
                    SampleNumber::Acc64 => SampleNumber::Acc64, // No wrap around
                };
            }
            Util::SetSampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            Util::SetRefVoltage => {
                self.adc_settings.ref_voltage = match self.adc_settings.ref_voltage {
                    ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef1_1V,
                    ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef1_5V,
                    ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef2_5V,
                    ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef4_34V,
                    ReferenceVoltage::VRef4_34V => ReferenceVoltage::Vdd,
                    ReferenceVoltage::Vdd => ReferenceVoltage::Vdd, // No wrap around
                };
            }
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
            Util::SetInitDelay => {
                self.adc_settings.init_delay = match self.adc_settings.init_delay {
                    DelayCycles::Delay0 => DelayCycles::Delay16,
                    DelayCycles::Delay16 => DelayCycles::Delay32,
                    DelayCycles::Delay32 => DelayCycles::Delay64,
                    DelayCycles::Delay64 => DelayCycles::Delay128,
                    DelayCycles::Delay128 => DelayCycles::Delay256,
                    DelayCycles::Delay256 => DelayCycles::Delay256, // No wrap around
                };
            }
            // Util::SetAsdv => {
            //     self.adc_settings.asdv = !self.adc_settings.asdv;
            // }
            Util::SetSampleDelay => {
                self.adc_settings.sample_delay = (self.adc_settings.sample_delay + 1).min(15);
            }
            Util::SetSampleLength => {
                self.adc_settings.sample_length = (self.adc_settings.sample_length + 1).min(31);
            }
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
                    // disable ADC when leaving utils mode
                    self.cur_util = Util::ReadTemp; // reset to first util
                    self.adc0.ctrla.write(|w| w.enable().clear_bit());
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    let next_util = match self.cur_util {
                        Util::ReadTemp => Util::ReadVext,
                        Util::ReadVext => Util::ReadVint,
                        Util::ReadVint => Util::SetResolution,
                        Util::SetResolution => Util::SetSampleNumber,
                        Util::SetSampleNumber => Util::SetSampCap,
                        Util::SetSampCap => Util::SetRefVoltage,
                        Util::SetRefVoltage => Util::SetPrescaler,
                        Util::SetPrescaler => Util::SetInitDelay,
                        //Util::SetInitDelay => Util::SetAsdv,
                        Util::SetInitDelay => Util::SetSampleDelay,
                        //Util::SetAsdv => Util::SetSampleDelay,
                        Util::SetSampleDelay => Util::SetSampleLength,
                        Util::SetSampleLength => Util::ReadTemp,
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
            let buf = self.format_util();
            display.print_ascii_bytes(buf).unwrap();
        } else if let Some(raw) = self.read_raw() {
            let mut buf = [0; NUM_CHARS];
            self.format_raw(raw, &mut buf);
            display.print_ascii_bytes(&buf).unwrap();
        }
    }
}

//
// ADC stuff, eventually move to avr_hal impl
//

/// Configuration for the ADC peripheral.
#[derive(Clone, Copy)]
pub struct AdcSettings {
    // CTRLA
    pub resolution: Resolution,
    //pub free_running: bool,
    // CTRLB
    pub sample_number: SampleNumber,
    // CTRLC
    pub samp_cap: bool,
    pub ref_voltage: ReferenceVoltage, // overloaded with Vref settings
    pub clock_divider: ClockDivider,
    // CTRLD
    pub init_delay: DelayCycles,
    pub asdv: bool,       // automatic sampling delay variation
    pub sample_delay: u8, // delay between samples, 4 bit
    // CTRLE
    // pub win_comp_mode: WindowComparsionMode,
    // SAMPCTRL
    pub sample_length: u8, // extends ADC sample length, 5 bit
}

impl Default for AdcSettings {
    fn default() -> Self {
        AdcSettings {
            resolution: Resolution::default(),
            sample_number: SampleNumber::Acc1,
            samp_cap: true, // Vref default 1.1V > 1.0V
            ref_voltage: ReferenceVoltage::default(),
            clock_divider: ClockDivider::default(),
            init_delay: DelayCycles::Delay0,
            asdv: false,
            sample_delay: 0,
            sample_length: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Resolution {
    _10bit,
    _8bit,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::_10bit
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SampleNumber {
    Acc1, // 1 sample, no accumulation
    Acc2,
    Acc4,
    Acc8,
    Acc16,
    Acc32,
    Acc64,
}

#[derive(Clone, Copy)]
/// Select the voltage reference for the ADC peripheral, overloaded with Vref settings
pub enum ReferenceVoltage {
    // internal refs
    VRef0_55V,
    VRef1_1V,
    VRef1_5V,
    VRef2_5V,
    VRef4_34V,
    // external ref
    Vdd,
}

impl Default for ReferenceVoltage {
    fn default() -> Self {
        Self::VRef1_1V
    }
}

#[derive(Clone, Copy)]
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

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum DelayCycles {
    #[doc = "0: Delay 0 CLK_ADC cycles"]
    Delay0,
    Delay16,
    Delay32,
    Delay64,
    Delay128,
    Delay256,
}
