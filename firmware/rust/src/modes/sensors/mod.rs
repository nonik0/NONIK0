use super::ModeHandler;
use crate::{Adc0, Context, Display, Event, format::*, SavedSettings, Setting, Sigrow, Vref, NUM_CHARS};

mod adc;
mod settings;
use adc::*;
use settings::*;

pub struct Sensors {
    cur_reading: SensorReading,
    cur_setting: SensorSetting,
    settings_active: bool,
    last_update: u16,
    last_reading: u16,

    adc_settings: AdcSettings,
    adc0: Adc0,
    sigrow: Sigrow,
    vref: Vref,

    util_init: bool,
    show_raw: bool,
    show_tempf: bool,
}

impl Sensors {
    const DECIMAL_PRECISION: u16 = 2; // X.YY
    const ADC_SETTINGS: AdcSettings = AdcSettings {
        resolution: Resolution::_10bit,
        sample_number: SampleNumber::Acc64,
        samp_cap: true,
        ref_voltage: ReferenceVoltage::VRef2_5V,
        clock_divider: ClockDivider::Factor256,
        init_delay: DelayCycles::Delay256,
        asdv: false,
        sample_delay: 10,
        sample_length: 10,
    };

    pub fn new_with_settings(
        settings: &SavedSettings,
        //adc0: Adc0,
        // sigrow: Sigrow,
        // vref: Vref,
    ) -> Self {
        let saved_reading = match settings.read_setting_byte(Setting::SensorPage) {
            1 => SensorReading::Vext,
            2 => SensorReading::Vref,
            3 => SensorReading::Gnd,
            _ => SensorReading::Temp,
        };
        let hack = avrxmega_hal::Peripherals::take().unwrap();

        Sensors {
            cur_reading: saved_reading,
            cur_setting: SensorSetting::Resolution,
            settings_active: false,
            last_update: 0,
            last_reading: 0,

            adc_settings: Self::ADC_SETTINGS,
            adc0: hack.ADC0,
            sigrow: hack.SIGROW,
            vref: hack.VREF,

            util_init: false,
            show_raw: false,
            show_tempf: false,
            //display_buf: [0; NUM_CHARS],
        }
    }

    pub fn seed_rand(&mut self) {
        let mut sample_count = 0;
        let mut seed_value: u32 = 0;
        let adc_settings = Self::ADC_SETTINGS; // TODO: faster
                                               // get 4 bits of randomness from the 4 LSBs of 4 raw temp readings
        while sample_count < 4 {
            if let Some(reading) = self.read_raw(SensorReading::Temp, adc_settings) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }
        // same thing for Vexternal reading
        sample_count = 0;
        while sample_count < 4 {
            if let Some(reading) = self.read_raw(SensorReading::Vext, adc_settings) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }

        crate::Rand::seed(seed_value);
    }

    fn format_setting(&self, buf: &mut [u8; NUM_CHARS]) {
        match self.cur_setting {
            SensorSetting::Resolution => format_uint(
                buf,
                b"Res:",
                RESOLUTION_VALUES[self.adc_settings.resolution as usize],
                0,
                Some(b"b"),
            ),
            SensorSetting::SampleNumber => format_uint(
                buf,
                b"Snum:",
                SAMPLE_NUMBER_DIVISORS[self.adc_settings.sample_number as usize],
                0,
                None,
            ),
            SensorSetting::SampCap => format_buf(
                buf,
                b"Scap:",
                BOOL_STRINGS[self.adc_settings.samp_cap as usize],
            ),
            SensorSetting::RefVoltage => format_buf(
                buf,
                b"Vr:",
                REF_VOLTAGE_STRINGS[self.adc_settings.ref_voltage as usize],
            ),
            SensorSetting::Prescaler => format_uint(
                buf,
                b"Div:",
                CLOCK_DIVIDER_VALUES[self.adc_settings.clock_divider as usize],
                0,
                None,
            ),
            SensorSetting::InitDelay => format_uint(
                buf,
                b"Idly:",
                INIT_DELAY_VALUES[self.adc_settings.init_delay as usize],
                0,
                None,
            ),
            SensorSetting::SetAsdv => format_buf(
                buf,
                b"Asdv:",
                BOOL_STRINGS[self.adc_settings.asdv as usize],
            ),
            SensorSetting::SampleDelay => format_uint(
                buf,
                b"Sdly:",
                self.adc_settings.sample_delay as u16,
                0,
                None,
            ),
            SensorSetting::SampleLength => format_uint(
                buf,
                b"Slen:",
                self.adc_settings.sample_length as u16,
                0,
                None,
            ),
        }
    }

    fn format_reading(&mut self, raw: u16, buf: &mut [u8; NUM_CHARS]) {
        let (prefix, value, decimals, suffix) = match (self.cur_reading, self.show_raw) {
            (SensorReading::Temp, true) => (b"Tf:", raw, 0, None),
            (SensorReading::Temp, false) => (
                b"Tf:",
                self.temp_from_raw(raw),
                0,
                Some(if self.show_tempf { b"\x98F" } else { b"\x98C" }.as_slice()),
            ),
            (SensorReading::Vext, true) => (b"Ve:", raw, 0, None),
            (SensorReading::Vext, false) | (SensorReading::Vref, false) | (SensorReading::Gnd, false) => (
                match self.cur_reading {
                    SensorReading::Vext => b"Ve:",
                    SensorReading::Vref => b"Vr:",
                    SensorReading::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                self.voltage_from_raw(raw),
                Self::DECIMAL_PRECISION,
                Some(b"V".as_slice()),
            ),
            (SensorReading::Vref, true) => (b"Vr:", raw, 0, None),
            (SensorReading::Gnd, true) => (b"Vg:", raw, 0, None),
        };

        format_uint(buf, prefix, value, decimals, suffix);
    }

    fn read_raw(&mut self, reading: SensorReading, adc_settings: AdcSettings) -> Option<u16> {
        if self.adc0.command().read().stconv().bit_is_set() {
            return None; // Measurement ongoing
        }

        if !self.util_init {
            let (adc_settings, channel) = match reading {
                SensorReading::Temp => {
                    let mut temp_settings = adc_settings;
                    temp_settings.ref_voltage = ReferenceVoltage::VRef1_1V;
                    (
                        temp_settings,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::TEMPSENSE,
                    )
                }
                SensorReading::Vext => (
                    self.adc_settings,
                    avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::AIN10,
                ),
                SensorReading::Vref => (
                    self.adc_settings,
                    avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::INTREF,
                ),
                SensorReading::Gnd => (
                    self.adc_settings,
                    avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::GND,
                ),
            };
            self.apply_adc_settings(adc_settings);
            self.adc0.muxpos().write(|w| w.muxpos().variant(channel));
            self.adc0.command().write(|w| w.stconv().set_bit());
            self.util_init = true;
            return None;
        }

        // Measurement complete, get result
        let acc_divisor = SAMPLE_NUMBER_DIVISORS[self.adc_settings.sample_number as usize];
        let raw = self.adc0.res().read().bits() / acc_divisor;
        self.adc0.command().write(|w| w.stconv().set_bit());
        Some(raw)
    }

    fn voltage_from_raw(&self, raw: u16) -> u16 {
        let raw = raw as u32;
        let vrefe5 = VREF_E5_VALUES[self.adc_settings.ref_voltage as usize];
        let raw_max = match self.adc_settings.resolution {
            Resolution::_10bit => 1023, // 2^10 - 1
            Resolution::_8bit => 255,   // 2^8 - 1
        };
        let precision_divisor = 10u32.pow(5 - Self::DECIMAL_PRECISION as u32);

        (((raw * vrefe5) / raw_max) / precision_divisor) as u16
    }

    fn temp_from_raw(&mut self, raw: u16) -> u16 {
        let sigrow_offset = self.sigrow.tempsense1().read().bits() as i8;
        let sigrow_gain = self.sigrow.tempsense0().read().bits() as u8;

        let mut temp: u32 = ((raw as i32) - (sigrow_offset as i32)) as u32;
        temp = (temp as i32 * sigrow_gain as i32) as u32;
        temp += 0x80;
        temp >>= 8;
        let temp_k = temp as u16;
        let temp_c = temp_k.saturating_sub(273); // TODO <0
        if self.show_tempf {
            let temp_f = (temp_c as u32 * 9 / 5) + 32;
            temp_f as u16
        } else {
            temp_c
        }
    }

    fn apply_adc_settings(&mut self, settings: AdcSettings) {
        self.vref.ctrla().modify(|_, w| {
            w.adc0refsel()
                .variant(REF_VOLTAGE_VARIANTS[settings.ref_voltage as usize])
        });

        self.adc0.ctrla().write(|w| {
            w.ressel()
                .variant(RESOLUTION_VARIANTS[settings.resolution as usize]);
            w.enable().set_bit()
        });

        self.adc0.ctrlb().write(|w| {
            w.sampnum()
                .variant(SAMPLE_NUMBER_VARIANTS[settings.sample_number as usize])
        });

        self.adc0.ctrlc().write(|w| {
            w.sampcap().bit(settings.samp_cap);
            w.refsel().variant(match settings.ref_voltage {
                ReferenceVoltage::Vdd => avrxmega_hal::pac::adc0::ctrlc::REFSEL_A::VDDREF,
                _ => avrxmega_hal::pac::adc0::ctrlc::REFSEL_A::INTREF,
            });
            w.presc()
                .variant(CLOCK_DIVIDER_VARIANTS[settings.clock_divider as usize])
        });

        self.adc0.ctrld().write(|w| {
            w.initdly()
                .variant(INIT_DELAY_VARIANTS[settings.init_delay as usize]);
            w.asdv().bit(settings.asdv);
            w.sampdly().set(settings.sample_delay)
        });

        self.adc0
            .sampctrl()
            .write(|w| w.samplen().set(settings.sample_length));
    }

    fn decrement_cur_setting(&mut self) {
        match self.cur_setting {
            SensorSetting::Resolution => {
                self.adc_settings.resolution = self.adc_settings.resolution.prev();
            }
            SensorSetting::SampleNumber => {
                self.adc_settings.sample_number = self.adc_settings.sample_number.prev();
            }
            SensorSetting::SampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            SensorSetting::RefVoltage => {
                self.adc_settings.ref_voltage = self.adc_settings.ref_voltage.prev();
            }
            SensorSetting::Prescaler => {
                self.adc_settings.clock_divider = self.adc_settings.clock_divider.prev();
            }
            SensorSetting::InitDelay => {
                self.adc_settings.init_delay = self.adc_settings.init_delay.prev();
            }
            SensorSetting::SetAsdv => {
                self.adc_settings.asdv = !self.adc_settings.asdv;
            }
            SensorSetting::SampleDelay => {
                self.adc_settings.sample_delay = self.adc_settings.sample_delay.saturating_sub(1);
            }
            SensorSetting::SampleLength => {
                self.adc_settings.sample_length = self.adc_settings.sample_length.saturating_sub(1);
            }
        };
    }

    fn increment_cur_setting(&mut self) {
        match self.cur_setting {
            SensorSetting::Resolution => {
                self.adc_settings.resolution = self.adc_settings.resolution.next();
            }
            SensorSetting::SampleNumber => {
                self.adc_settings.sample_number = self.adc_settings.sample_number.next();
            }
            SensorSetting::SampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            SensorSetting::RefVoltage => {
                self.adc_settings.ref_voltage = self.adc_settings.ref_voltage.next();
            }
            SensorSetting::Prescaler => {
                self.adc_settings.clock_divider = self.adc_settings.clock_divider.next();
            }
            SensorSetting::InitDelay => {
                self.adc_settings.init_delay = self.adc_settings.init_delay.next();
            }
            SensorSetting::SetAsdv => {
                self.adc_settings.asdv = !self.adc_settings.asdv;
            }
            SensorSetting::SampleDelay => {
                self.adc_settings.sample_delay = (self.adc_settings.sample_delay + 1).min(15);
            }
            SensorSetting::SampleLength => {
                self.adc_settings.sample_length = (self.adc_settings.sample_length + 1).min(31);
            }
        };
    }

    fn toggle_reading_format(&mut self) {
        match self.cur_reading {
            SensorReading::Temp => {
                if !self.show_raw && !self.show_tempf {
                    self.show_tempf = true;
                } else if !self.show_raw && self.show_tempf {
                    self.show_tempf = false;
                    self.show_raw = true;
                } else {
                    self.show_raw = false;
                }
            }
            _ => self.show_raw = !self.show_raw,
        }
    }
}

impl ModeHandler for Sensors {
    #[inline(never)]
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    // exit from settings to readings if active, or exit mode if readings active
                    if self.settings_active {
                        self.settings_active = false;
                        self.util_init = false; // reapply adc settings
                        update = true;
                    } else {
                        // disable ADC when leaving utils mode
                        self.settings_active = false;
                        self.adc0.ctrla().write(|w| w.enable().clear_bit());
                        self.util_init = false;
                        context
                            .settings
                            .save_setting_byte(Setting::SensorPage, self.cur_reading as u8);
                        context.to_menu();
                        return;
                    }
                }
                Event::RightHeld => {
                    // toggle setting or toggle into settings
                    if self.settings_active {
                        self.cur_setting = self.cur_setting.next();
                    } else {
                        self.settings_active = true;
                    }
                    update = true;
                }
                Event::LeftReleased => {
                    if self.settings_active {
                        self.decrement_cur_setting();
                    } else {
                        self.toggle_reading_format();
                    }
                    update = true;
                }
                Event::RightReleased => {
                    if self.settings_active {
                        self.increment_cur_setting();
                    } else {
                        self.cur_reading = self.cur_reading.next();
                        self.util_init = false;
                    }
                    update = true;
                }
                _ => {}
            }
        }

        // check for new ADC reading
        if !update && !self.settings_active {
            if let Some(raw) = self.read_raw(self.cur_reading, self.adc_settings) {
                if raw != self.last_reading {
                    self.last_reading = raw;
                    update = true;
                }
            }
        }

        if update {
            let mut buf = [0; NUM_CHARS];
            if self.settings_active {
                self.format_setting(&mut buf);
            } else {
                self.format_reading(self.last_reading, &mut buf);
            }

            display.print_ascii_bytes(&buf).unwrap();
        }
    }
}
