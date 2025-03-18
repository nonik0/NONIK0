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
    const DECIMAL_PRECISION: u16 = 2; // X.YY
    const TEMP_ADC_SETTINGS: AdcSettings = AdcSettings {
        resolution: Resolution::_10bit,
        sample_number: SampleNumber::Acc1,
        samp_cap: true,
        ref_voltage: ReferenceVoltage::VRef1_1V,
        clock_divider: ClockDivider::Factor256,
        init_delay: DelayCycles::Delay32,
        asdv: false,
        sample_delay: 0,
        sample_length: 2,
    };

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
                ReferenceVoltage::VRef0_55V => b"Vr:0.55V",
                ReferenceVoltage::VRef1_1V => b"Vr: 1.1V",
                ReferenceVoltage::VRef2_5V => b"Vr: 2.5V",
                ReferenceVoltage::VRef4_34V => b"Vr:4.34V",
                ReferenceVoltage::VRef1_5V => b"Vr: 1.5V",
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

        let (prefix, mut value, mut decimals, suffix) = match self.cur_util {
            Util::ReadTemp => {
                if self.show_raw {
                    (b"Tf:", raw, 0, None)
                } else {
                    (
                        b"Tf:",
                        self.temp_from_raw(raw),
                        Self::DECIMAL_PRECISION,
                        Some(b'\x98'),
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
                        Some(b'V'),
                    )
                }
            }
            _ => return,
        };

        // copy prefix and suffix to buf (i.e. "Ve:____V")
        buf[..PREFIX_LEN].copy_from_slice(prefix);
        if let Some(suffix) = suffix {
            buf[NUM_CHARS - 1] = suffix;
        }

        let mut val_chars = 0;
        let reading_len = if suffix.is_some() {
            NUM_CHARS - PREFIX_LEN - 1
        } else {
            NUM_CHARS - PREFIX_LEN
        };
        for index in (PREFIX_LEN..PREFIX_LEN + reading_len).rev() {
            if decimals > 0 && val_chars == decimals {
                buf[index] = b'.';
                decimals = 0;
            } else if value > 0 {
                buf[index] = b'0' + (value % 10) as u8;
                value /= 10;
                val_chars += 1;
            } else {
                buf[index] = if val_chars < 3 {
                    val_chars += 1;
                    b'0'
                } else {
                    b' '
                }; // for leading 0 in 0.XX
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
                let (adc_settings, channel) = match self.cur_util {
                    Util::ReadTemp => (
                        Self::TEMP_ADC_SETTINGS,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::TEMPSENSE,
                    ),
                    Util::ReadVext => (
                        self.adc_settings,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::AIN10,
                    ),
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
        let precision_divider = 10u32.pow(5 - Self::DECIMAL_PRECISION as u32);

        (((raw * vrefe5) / raw_max) / precision_divider) as u16
    }

    fn temp_from_raw(&mut self, raw: u16) -> u16 {
        let sigrow_offset = self.sigrow.tempsense1.read().bits() as i16; // Read signed value from signature row
        let sigrow_gain = self.sigrow.tempsense0.read().bits() as u16; // Read unsigned value from signature row

        let mut raw_temp = (raw as i32) - (sigrow_offset as i32); // Perform subtraction with proper casting
        raw_temp *= sigrow_gain as i32; // Multiply with gain
        raw_temp += 0x80; // Add 1/2 to get correct rounding
        raw_temp >>= 8; // Divide result to get Kelvin
                        //let temperature_in_k = raw_temp as u16; // Cast back to u16
        let temp_f = ((raw_temp as i32 - 273) * 9 / 5 + 32) as u16;
        temp_f
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
                    ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef1_1V,
                    ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef1_5V,
                    ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef2_5V,
                    ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef4_34V,
                    ReferenceVoltage::VRef4_34V => ReferenceVoltage::Vdd,
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
                    ReferenceVoltage::Vdd => ReferenceVoltage::VRef4_34V,
                    ReferenceVoltage::VRef4_34V => ReferenceVoltage::VRef2_5V,
                    ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef1_5V,
                    ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef1_1V,
                    ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef0_55V,
                    ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef0_55V, // No wrap around
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
            ref_voltage: ReferenceVoltage::VRef2_5V,
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
