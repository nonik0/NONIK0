use super::ModeHandler;
use crate::{impl_enum_cycle, adc::*, utils::*, Context, Event, Peripherals, SavedSettings, Setting, NUM_CHARS};

pub const RESOLUTION_VALUES: [u16; 2] = [10, 8]; // 2^10 - 1, 2^8 - 1
                                                 //pub const SAMPLE_NUMBER_DIVISORS: [u16; 7] = [1, 2, 4, 8, 16, 32, 64];
pub const PRESCALER_VALUES: [u16; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
pub const INIT_DELAY_VALUES: [u16; 6] = [0, 16, 32, 64, 128, 256];
pub const INT_REF_VOLTAGE_STRINGS: [&[u8]; 5] = [b"0.55V", b"1.1V", b"2.5V", b"4.34V", b"1.5V"];
pub const VDD_REF_VOLTAGE_STRING: &[u8] = b"Vdd";
pub const BOOL_STRINGS: [&[u8]; 2] = [b" no", b"yes"];

#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SensorSetting {
    Resolution,
    SampleNumber,
    SampCap,
    RefVoltage,
    Prescaler,
    InitDelay,
    SetAsdv,
    SampleDelay,
    SampleLength,
}

impl_enum_cycle!(SensorSetting, 9);

pub struct Sensors {
    cur_channel: AdcChannel,
    cur_setting: SensorSetting,
    settings_active: bool,
    last_update: u16,
    last_reading: u16,

    adc_settings: AdcSettings,
    show_raw: bool,
    show_tempf: bool,
}

impl Sensors {
    const DECIMAL_PRECISION: u16 = 2; // X.YY

    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let saved_reading = match settings.read_setting_byte(Setting::SensorPage) {
            1 => AdcChannel::Vext,
            2 => AdcChannel::Vref,
            3 => AdcChannel::Gnd,
            _ => AdcChannel::Temp,
        };

        Sensors {
            cur_channel: saved_reading,
            cur_setting: SensorSetting::Resolution,
            settings_active: false,
            last_update: 0,
            last_reading: 0,
            adc_settings: AdcSettings::default(), // TODO
            show_raw: false,
            show_tempf: false,
        }
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
                //[self.adc_settings.sample_number as usize],
                1 << (self.adc_settings.sample_number as u8),
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
                if self.adc_settings.adc_ref_voltage == AdcReferenceVoltage::INTREF {
                    INT_REF_VOLTAGE_STRINGS[self.adc_settings.int_ref_voltage as usize]
                } else {
                    VDD_REF_VOLTAGE_STRING
                },
            ),
            SensorSetting::Prescaler => format_uint(
                buf,
                b"Div:",
                PRESCALER_VALUES[self.adc_settings.prescaler as usize],
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
            SensorSetting::SetAsdv => {
                format_buf(buf, b"Asdv:", BOOL_STRINGS[self.adc_settings.asdv as usize])
            }
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

    fn format_reading(&mut self, value: u16, buf: &mut [u8; NUM_CHARS]) {
        let (prefix, value, decimals, suffix) = match (self.cur_channel, self.show_raw) {
            (AdcChannel::Temp, true) => (b"Tf:", value, 0, None),
            (AdcChannel::Temp, false) => (
                b"Tf:",
                value,
                0,
                Some(if self.show_tempf { b"\x98F" } else { b"\x98C" }.as_slice()),
            ),
            (AdcChannel::Vext, true) => (b"Ve:", value, 0, None),
            (AdcChannel::Vext, false) | (AdcChannel::Vref, false) | (AdcChannel::Gnd, false) => (
                match self.cur_channel {
                    AdcChannel::Vext => b"Ve:",
                    AdcChannel::Vref => b"Vr:",
                    AdcChannel::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                value,
                Self::DECIMAL_PRECISION,
                Some(b"V".as_slice()),
            ),
            (AdcChannel::Vref, true) => (b"Vr:", value, 0, None),
            (AdcChannel::Gnd, true) => (b"Vg:", value, 0, None),
        };

        format_uint(buf, prefix, value, decimals, suffix);
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
                // decrement to internal reference voltage if at VDD reference voltage
                if self.adc_settings.adc_ref_voltage == AdcReferenceVoltage::VDDREF {
                    // self.adc_settings.adc_ref_voltage == AdcReferenceVoltage::VDD
                    self.adc_settings.adc_ref_voltage = AdcReferenceVoltage::INTREF;
                    self.adc_settings.int_ref_voltage = IntReferenceVoltage::_1V5;
                } else {
                    self.adc_settings.int_ref_voltage = self.adc_settings.int_ref_voltage.prev();
                }
            }
            SensorSetting::Prescaler => {
                self.adc_settings.prescaler = self.adc_settings.prescaler.prev();
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
                if self.adc_settings.adc_ref_voltage == AdcReferenceVoltage::INTREF {
                    // advance to VDD reference voltage if at last internal reference voltage
                    if self.adc_settings.int_ref_voltage == IntReferenceVoltage::_1V5 {
                        // 1.5V is the last option, so switch to VDD
                        self.adc_settings.adc_ref_voltage = AdcReferenceVoltage::VDDREF;
                    } else {
                        self.adc_settings.int_ref_voltage =
                            self.adc_settings.int_ref_voltage.next();
                    }
                }
            }
            SensorSetting::Prescaler => {
                self.adc_settings.prescaler = self.adc_settings.prescaler.next();
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
        match self.cur_channel {
            AdcChannel::Temp => {
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
    fn update(
        &mut self,
        event: &Option<Event>,
        context: &mut Context,
        peripherals: &mut Peripherals,
    ) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    // in settings: exit to readings and apply settings, in readings: exit to menu
                    if self.settings_active {
                        self.settings_active = false;
                        peripherals.adc.apply_settings(self.adc_settings);
                        update = true;
                    } else {
                        // disable ADC when leaving utils mode
                        self.settings_active = false;
                        peripherals.adc.disable();
                        context
                            .settings
                            .save_setting_byte(Setting::SensorPage, self.cur_channel as u8);
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
                        self.cur_channel = self.cur_channel.next_wrapping();
                    }
                    update = true;
                }
                _ => {}
            }
        }

        // check for new ADC reading
        if !update && !self.settings_active {
            let reading = if self.show_raw {
                peripherals.adc.read_raw_nonblocking(self.cur_channel)
            } else {
                match self.cur_channel {
                    AdcChannel::Temp => peripherals.adc.read_temp_nonblocking(self.show_tempf),
                    _ => peripherals.adc.read_voltage_nonblocking(self.cur_channel),
                }
            };

            if let Some(reading) = reading {
                if reading != self.last_reading {
                    self.last_reading = reading;
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

            peripherals.display.print_ascii_bytes(&buf).unwrap();
        }
    }
}
