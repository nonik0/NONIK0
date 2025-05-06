use super::Mode;
use crate::{Adc0, Context, Display, Event, SavedSettings, Setting, Sigrow, Vref, NUM_CHARS};

const REF_VOLTAGE_VARIANTS: [avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A; 5] = [
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_0V55,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_1V1,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_2V5,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_4V34,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_1V5,
];
const RESOLUTION_VARIANTS: [avrxmega_hal::pac::adc0::ctrla::RESSEL_A; 2] = [
    avrxmega_hal::pac::adc0::ctrla::RESSEL_A::_10BIT,
    avrxmega_hal::pac::adc0::ctrla::RESSEL_A::_8BIT,
];
const SAMPLE_NUMBER_VARIANTS: [avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A; 7] = [
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC1,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC2,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC4,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC8,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC16,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC32,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC64,
];
const CLOCK_DIVIDER_VARIANTS: [avrxmega_hal::pac::adc0::ctrlc::PRESC_A; 8] = [
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV2,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV4,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV8,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV16,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV32,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV64,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV128,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV256,
];
const INIT_DELAY_VARIANTS: [avrxmega_hal::pac::adc0::ctrld::INITDLY_A; 6] = [
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY0,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY16,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY32,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY64,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY128,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY256,
];
const VREF_E5_VALUES: [u32; 6] = [55000, 110000, 150000, 250000, 434000, 360000];
const RESOLUTION_VALUES: [u16; 2] = [10, 8]; // 2^10 - 1, 2^8 - 1
const SAMPLE_NUMBER_DIVISORS: [u16; 7] = [1, 2, 4, 8, 16, 32, 64];
const CLOCK_DIVIDER_VALUES: [u16; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
const INIT_DELAY_VALUES: [u16; 6] = [0, 16, 32, 64, 128, 256];

const REF_VOLTAGE_STRINGS: [&[u8]; 6] = [b"0.55V", b"1.1V", b"1.5V", b"2.5V", b"4.34V", b"Vdd"];
const BOOL_STRINGS: [&[u8]; 2] = [b" no", b"yes"];

#[derive(Clone, Copy)]
enum AdcReading {
    Temp = 0,
    Vext = 1,
    Vref = 2,
    Gnd = 3,
}

impl AdcReading {
    fn next(&self) -> Self {
        match self {
            AdcReading::Temp => AdcReading::Vext,
            AdcReading::Vext => AdcReading::Vref,
            AdcReading::Vref => AdcReading::Gnd,
            AdcReading::Gnd => AdcReading::Temp,
        }
    }
}

enum AdcSetting {
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

impl AdcSetting {
    fn next(&self) -> Self {
        match self {
            AdcSetting::Resolution => AdcSetting::SampleNumber,
            AdcSetting::SampleNumber => AdcSetting::SampCap,
            AdcSetting::SampCap => AdcSetting::RefVoltage,
            AdcSetting::RefVoltage => AdcSetting::Prescaler,
            AdcSetting::Prescaler => AdcSetting::InitDelay,
            AdcSetting::InitDelay => AdcSetting::SetAsdv,
            AdcSetting::SetAsdv => AdcSetting::SampleDelay,
            AdcSetting::SampleDelay => AdcSetting::SampleLength,
            AdcSetting::SampleLength => AdcSetting::Resolution,
        }
    }
}

pub struct Sensors {
    cur_reading: AdcReading,
    cur_setting: AdcSetting,
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
    display_buf: [u8; NUM_CHARS],
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
        adc0: Adc0,
        sigrow: Sigrow,
        vref: Vref,
    ) -> Self {
        let saved_reading = match settings.read_setting_byte(Setting::SensorPage) {
            1 => AdcReading::Vext,
            2 => AdcReading::Vref,
            3 => AdcReading::Gnd,
            _ => AdcReading::Temp,
        };

        Sensors {
            cur_reading: saved_reading,
            cur_setting: AdcSetting::Resolution,
            settings_active: false,
            last_update: 0,
            last_reading: 0,

            adc_settings: Self::ADC_SETTINGS,
            adc0,
            sigrow,
            vref,

            util_init: false,
            show_raw: false,
            show_tempf: false,
            display_buf: [0; NUM_CHARS],
        }
    }

    pub fn seed_rand(&mut self) {
        let mut sample_count = 0;
        let mut seed_value: u32 = 0;
        let adc_settings = Self::ADC_SETTINGS; // TODO: faster
                                               // get 4 bits of randomness from the 4 LSBs of 4 raw temp readings
        while sample_count < 4 {
            if let Some(reading) = self.read_raw(AdcReading::Temp, adc_settings) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }
        // same thing for Vexternal reading
        sample_count = 0;
        while sample_count < 4 {
            if let Some(reading) = self.read_raw(AdcReading::Vext, adc_settings) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }

        crate::Rand::seed(seed_value);
    }

    fn format_setting(&self, buf: &mut [u8; NUM_CHARS]) {
        match self.cur_setting {
            AdcSetting::Resolution => format_uint(
                buf,
                b"Res:",
                RESOLUTION_VALUES[self.adc_settings.resolution as usize],
                0,
                Some(b"b"),
            ),
            AdcSetting::SampleNumber => format_uint(
                buf,
                b"Snum:",
                SAMPLE_NUMBER_DIVISORS[self.adc_settings.sample_number as usize],
                0,
                None,
            ),
            AdcSetting::SampCap => format_buf(
                buf,
                b"Scap:",
                BOOL_STRINGS[self.adc_settings.samp_cap as usize],
            ),
            AdcSetting::RefVoltage => format_buf(
                buf,
                b"Vr:",
                REF_VOLTAGE_STRINGS[self.adc_settings.ref_voltage as usize],
            ),
            AdcSetting::Prescaler => format_uint(
                buf,
                b"Div:",
                CLOCK_DIVIDER_VALUES[self.adc_settings.clock_divider as usize],
                0,
                None,
            ),
            AdcSetting::InitDelay => format_uint(
                buf,
                b"Idly:",
                INIT_DELAY_VALUES[self.adc_settings.init_delay as usize],
                0,
                None,
            ),
            AdcSetting::SetAsdv => format_buf(
                buf,
                b"Asdv:",
                BOOL_STRINGS[self.adc_settings.asdv as usize],
            ),
            AdcSetting::SampleDelay => format_uint(
                buf,
                b"Sdly:",
                self.adc_settings.sample_delay as u16,
                0,
                None,
            ),
            AdcSetting::SampleLength => format_uint(
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
            (AdcReading::Temp, true) => (b"Tf:", raw, 0, None),
            (AdcReading::Temp, false) => (
                b"Tf:",
                self.temp_from_raw(raw),
                0,
                Some(if self.show_tempf { b"\x98F" } else { b"\x98C" }.as_slice()),
            ),
            (AdcReading::Vext, true) => (b"Ve:", raw, 0, None),
            (AdcReading::Vext, false) | (AdcReading::Vref, false) | (AdcReading::Gnd, false) => (
                match self.cur_reading {
                    AdcReading::Vext => b"Ve:",
                    AdcReading::Vref => b"Vr:",
                    AdcReading::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                self.voltage_from_raw(raw),
                Self::DECIMAL_PRECISION,
                Some(b"V".as_slice()),
            ),
            (AdcReading::Vref, true) => (b"Vr:", raw, 0, None),
            (AdcReading::Gnd, true) => (b"Vg:", raw, 0, None),
        };

        format_uint(buf, prefix, value, decimals, suffix);
    }

    fn read_raw(&mut self, reading: AdcReading, adc_settings: AdcSettings) -> Option<u16> {
        if self.adc0.command().read().stconv().bit_is_set() {
            return None; // Measurement ongoing
        }

        if !self.util_init {
            let (adc_settings, channel) = match reading {
                AdcReading::Temp => {
                    let mut temp_settings = adc_settings;
                    temp_settings.ref_voltage = ReferenceVoltage::VRef1_1V;
                    (
                        temp_settings,
                        avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::TEMPSENSE,
                    )
                }
                AdcReading::Vext => (
                    self.adc_settings,
                    avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::AIN10,
                ),
                AdcReading::Vref => (
                    self.adc_settings,
                    avrxmega_hal::pac::adc0::muxpos::MUXPOS_A::INTREF,
                ),
                AdcReading::Gnd => (
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
            AdcSetting::Resolution => {
                self.adc_settings.resolution = self.adc_settings.resolution.prev();
            }
            AdcSetting::SampleNumber => {
                self.adc_settings.sample_number = self.adc_settings.sample_number.prev();
            }
            AdcSetting::SampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            AdcSetting::RefVoltage => {
                self.adc_settings.ref_voltage = self.adc_settings.ref_voltage.prev();
            }
            AdcSetting::Prescaler => {
                self.adc_settings.clock_divider = self.adc_settings.clock_divider.prev();
            }
            AdcSetting::InitDelay => {
                self.adc_settings.init_delay = self.adc_settings.init_delay.prev();
            }
            AdcSetting::SetAsdv => {
                self.adc_settings.asdv = !self.adc_settings.asdv;
            }
            AdcSetting::SampleDelay => {
                self.adc_settings.sample_delay = self.adc_settings.sample_delay.saturating_sub(1);
            }
            AdcSetting::SampleLength => {
                self.adc_settings.sample_length = self.adc_settings.sample_length.saturating_sub(1);
            }
        };
    }

    fn increment_cur_setting(&mut self) {
        match self.cur_setting {
            AdcSetting::Resolution => {
                self.adc_settings.resolution = self.adc_settings.resolution.next();
            }
            AdcSetting::SampleNumber => {
                self.adc_settings.sample_number = self.adc_settings.sample_number.next();
            }
            AdcSetting::SampCap => {
                self.adc_settings.samp_cap = !self.adc_settings.samp_cap;
            }
            AdcSetting::RefVoltage => {
                self.adc_settings.ref_voltage = self.adc_settings.ref_voltage.next();
            }
            AdcSetting::Prescaler => {
                self.adc_settings.clock_divider = self.adc_settings.clock_divider.next();
            }
            AdcSetting::InitDelay => {
                self.adc_settings.init_delay = self.adc_settings.init_delay.next();
            }
            AdcSetting::SetAsdv => {
                self.adc_settings.asdv = !self.adc_settings.asdv;
            }
            AdcSetting::SampleDelay => {
                self.adc_settings.sample_delay = (self.adc_settings.sample_delay + 1).min(15);
            }
            AdcSetting::SampleLength => {
                self.adc_settings.sample_length = (self.adc_settings.sample_length + 1).min(31);
            }
        };
    }

    fn toggle_reading_format(&mut self) {
        match self.cur_reading {
            AdcReading::Temp => {
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

impl Mode for Sensors {
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

//
// Helper function to format an unsigned integer with a prefix and suffix value
//
fn format_uint(
    buf: &mut [u8],
    prefix: &[u8],
    value: u16,
    decimal_digits: u16,
    suffix: Option<&[u8]>,
) {
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

fn format_buf(buf: &mut [u8], left: &[u8], right: &[u8]) {
    let num_chars = buf.len();
    let left_len = left.len();
    let right_len = right.len();

    if left_len + right_len > num_chars {
        panic!("Left and right strings are too long to fit in the buffer");
    }

    buf[..left_len].copy_from_slice(left);
    buf[num_chars - right_len..].copy_from_slice(right);
    for i in left_len..num_chars - right_len {
        buf[i] = b' ';
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
    _10bit = 0,
    _8bit,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::_10bit
    }
}

impl Resolution {
    fn next(&self) -> Self {
        match self {
            Resolution::_8bit => Resolution::_10bit,
            Resolution::_10bit => Resolution::_10bit, // No wrap around
        }
    }

    fn prev(&self) -> Self {
        match self {
            Resolution::_10bit => Resolution::_8bit,
            Resolution::_8bit => Resolution::_8bit, // No wrap around
        }
    }
}

#[derive(Clone, Copy)]
pub enum SampleNumber {
    Acc1 = 0, // 1 sample, no accumulation
    Acc2,
    Acc4,
    Acc8,
    Acc16,
    Acc32,
    Acc64,
}

impl SampleNumber {
    fn next(&self) -> Self {
        match self {
            SampleNumber::Acc1 => SampleNumber::Acc2,
            SampleNumber::Acc2 => SampleNumber::Acc4,
            SampleNumber::Acc4 => SampleNumber::Acc8,
            SampleNumber::Acc8 => SampleNumber::Acc16,
            SampleNumber::Acc16 => SampleNumber::Acc32,
            SampleNumber::Acc32 => SampleNumber::Acc64,
            SampleNumber::Acc64 => SampleNumber::Acc64, // No wrap around
        }
    }

    fn prev(&self) -> Self {
        match self {
            SampleNumber::Acc64 => SampleNumber::Acc32,
            SampleNumber::Acc32 => SampleNumber::Acc16,
            SampleNumber::Acc16 => SampleNumber::Acc8,
            SampleNumber::Acc8 => SampleNumber::Acc4,
            SampleNumber::Acc4 => SampleNumber::Acc2,
            SampleNumber::Acc2 => SampleNumber::Acc1,
            SampleNumber::Acc1 => SampleNumber::Acc1, // No wrap around
        }
    }
}

#[derive(Clone, Copy)]
/// Select the voltage reference for the ADC peripheral, overloaded with Vref settings
pub enum ReferenceVoltage {
    // internal refs
    VRef0_55V = 0,
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

impl ReferenceVoltage {
    fn next(&self) -> Self {
        match self {
            ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef1_1V,
            ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef1_5V,
            ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef2_5V,
            ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef4_34V,
            ReferenceVoltage::VRef4_34V => ReferenceVoltage::Vdd,
            ReferenceVoltage::Vdd => ReferenceVoltage::Vdd, // No wrap around
        }
    }

    fn prev(&self) -> Self {
        match self {
            ReferenceVoltage::Vdd => ReferenceVoltage::VRef4_34V,
            ReferenceVoltage::VRef4_34V => ReferenceVoltage::VRef2_5V,
            ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef1_5V,
            ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef1_1V,
            ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef0_55V,
            ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef0_55V, // No wrap around
        }
    }
}

#[derive(Clone, Copy)]
pub enum ClockDivider {
    Factor2,
    Factor4,
    Factor8,
    Factor16,
    Factor32,
    Factor64,
    /// (default)
    Factor128,
    Factor256,
}

impl Default for ClockDivider {
    fn default() -> Self {
        Self::Factor128
    }
}

impl ClockDivider {
    fn next(&self) -> Self {
        match self {
            ClockDivider::Factor2 => ClockDivider::Factor4,
            ClockDivider::Factor4 => ClockDivider::Factor8,
            ClockDivider::Factor8 => ClockDivider::Factor16,
            ClockDivider::Factor16 => ClockDivider::Factor32,
            ClockDivider::Factor32 => ClockDivider::Factor64,
            ClockDivider::Factor64 => ClockDivider::Factor128,
            ClockDivider::Factor128 => ClockDivider::Factor256,
            ClockDivider::Factor256 => ClockDivider::Factor256, // No wrap around
        }
    }

    fn prev(&self) -> Self {
        match self {
            ClockDivider::Factor256 => ClockDivider::Factor128,
            ClockDivider::Factor128 => ClockDivider::Factor64,
            ClockDivider::Factor64 => ClockDivider::Factor32,
            ClockDivider::Factor32 => ClockDivider::Factor16,
            ClockDivider::Factor16 => ClockDivider::Factor8,
            ClockDivider::Factor8 => ClockDivider::Factor4,
            ClockDivider::Factor4 => ClockDivider::Factor2,
            ClockDivider::Factor2 => ClockDivider::Factor2, // No wrap around
        }
    }
}

#[derive(Clone, Copy)]
pub enum DelayCycles {
    Delay0 = 0,
    Delay16,
    Delay32,
    Delay64,
    Delay128,
    Delay256,
}

impl DelayCycles {
    fn next(&self) -> Self {
        match self {
            DelayCycles::Delay0 => DelayCycles::Delay16,
            DelayCycles::Delay16 => DelayCycles::Delay32,
            DelayCycles::Delay32 => DelayCycles::Delay64,
            DelayCycles::Delay64 => DelayCycles::Delay128,
            DelayCycles::Delay128 => DelayCycles::Delay256,
            DelayCycles::Delay256 => DelayCycles::Delay256, // No wrap around
        }
    }

    fn prev(&self) -> Self {
        match self {
            DelayCycles::Delay256 => DelayCycles::Delay128,
            DelayCycles::Delay128 => DelayCycles::Delay64,
            DelayCycles::Delay64 => DelayCycles::Delay32,
            DelayCycles::Delay32 => DelayCycles::Delay16,
            DelayCycles::Delay16 => DelayCycles::Delay0,
            DelayCycles::Delay0 => DelayCycles::Delay0, // No wrap around
        }
    }
}
