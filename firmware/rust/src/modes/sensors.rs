use super::ModeHandler;
use crate::{
    adc::*, impl_enum_cycle, utils::*, Context, Event, Peripherals, SavedSettings, Setting,
    NUM_CHARS,
};

pub const RESOLUTION_VALUES: [u16; 2] = [10, 8]; // 2^10 - 1, 2^8 - 1
                                                 //pub const SAMPLE_NUMBER_DIVISORS: [u16; 7] = [1, 2, 4, 8, 16, 32, 64];
pub const PRESCALER_VALUES: [u16; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
pub const INIT_DELAY_VALUES: [u16; 6] = [0, 16, 32, 64, 128, 256];
pub const INT_REF_VOLTAGE_STRINGS: [&[u8]; 5] = [b"0.55V", b"1.1V", b"2.5V", b"4.34V", b"1.5V"];
pub const VDD_REF_VOLTAGE_STRING: &[u8] = b"Vdd";
pub const BOOL_STRINGS: [&[u8]; 2] = [b" no", b"yes"];

#[cfg(not(feature = "board_v0"))]
pub const CONTINUITY_BLUE_READING_MASK: u16 = 0x0001;
#[cfg(not(feature = "board_v0"))]
pub const CONTINUITY_YELLOW_READING_MASK: u16 = 0x0002;
#[cfg(not(feature = "board_v0"))]
pub const CONTINUITY_BOTH_READING_MASK: u16 = 0x0003;

#[derive(Clone, Copy)]
pub enum SensorPage {
    AdcChannel(AdcChannel),
    ContinuityTest,
}

impl SensorPage {
    pub fn next(&self) -> Self {
        match self {
            SensorPage::AdcChannel(channel) => {
                let next_channel = channel.next_wrapping();
                if next_channel.to_u8() != 0 {
                    SensorPage::AdcChannel(next_channel)
                } else {
                    SensorPage::ContinuityTest
                }
            }
            SensorPage::ContinuityTest => SensorPage::AdcChannel(AdcChannel::from_u8(0)),
        }
    }
}

impl From<u8> for SensorPage {
    fn from(value: u8) -> Self {
        #[cfg(not(feature = "board_v0"))]
        match value {
            1 => SensorPage::AdcChannel(AdcChannel::Vsda),
            2 => SensorPage::AdcChannel(AdcChannel::Vscl),
            3 => SensorPage::AdcChannel(AdcChannel::Gnd),
            4 => SensorPage::AdcChannel(AdcChannel::Vref),
            5 => SensorPage::ContinuityTest,
            _ => SensorPage::AdcChannel(AdcChannel::Temp),
        }
        #[cfg(feature = "board_v0")]
        match value {
            1 => SensorPage::AdcChannel(AdcChannel::Vext),
            2 => SensorPage::AdcChannel(AdcChannel::Gnd),
            3 => SensorPage::AdcChannel(AdcChannel::Vref),
            4 => SensorPage::ContinuityTest,
            _ => SensorPage::AdcChannel(AdcChannel::Temp),
        };
    }
}

impl From<SensorPage> for u8 {
    fn from(value: SensorPage) -> Self {
        #[cfg(not(feature = "board_v0"))]
        match value {
            SensorPage::AdcChannel(AdcChannel::Vsda) => 1,
            SensorPage::AdcChannel(AdcChannel::Vscl) => 2,
            SensorPage::AdcChannel(AdcChannel::Gnd) => 3,
            SensorPage::AdcChannel(AdcChannel::Vref) => 4,
            SensorPage::ContinuityTest => 5,
            _ => 0,
        }
        #[cfg(feature = "board_v0")]
        match value {
            SensorPage::AdcChannel(AdcChannel::Vext) => 1,
            SensorPage::AdcChannel(AdcChannel::Gnd) => 2,
            SensorPage::AdcChannel(AdcChannel::Vref) => 3,
            SensorPage::ContinuityTest => 4,
            _ => 0,
        }
    }
}

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
    cur_page: SensorPage,
    cur_setting: SensorSetting,
    port_init: bool,
    tone_active: u8, // different values for continuity channel
    settings_active: bool,
    // for continuity, >0 is continuity, ==0 is none
    last_reading: u16,
    continuity_channel: bool, // false = blue/sda, true = yellow/scl

    show_raw: bool,
    show_tempf: bool,
}

impl Sensors {
    const DECIMAL_PRECISION: u16 = 2; // X.YY

    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let saved_page = settings.read_setting_byte(Setting::SensorPage);

        Sensors {
            cur_page: saved_page.into(),
            cur_setting: SensorSetting::Resolution,
            port_init: false,
            tone_active: 0,
            settings_active: false,
            last_reading: 0,
            continuity_channel: false,
            show_raw: false,
            show_tempf: false,
        }
    }

    #[inline(never)]
    fn format_setting(&self, buf: &mut [u8; NUM_CHARS], adc: &AdcSettings) {
        match self.cur_setting {
            SensorSetting::Resolution => format_uint(
                buf,
                b"Res:",
                RESOLUTION_VALUES[adc.resolution as usize],
                0,
                Some(b"b"),
            ),
            SensorSetting::SampleNumber => {
                format_uint(buf, b"Snum:", 1 << (adc.sample_number as u8), 0, None)
            }
            SensorSetting::SampCap => {
                format_buf(buf, b"Scap:", BOOL_STRINGS[adc.samp_cap as usize])
            }
            SensorSetting::RefVoltage => format_buf(
                buf,
                b"Vr:",
                if adc.adc_ref_voltage == AdcReferenceVoltage::INTREF {
                    INT_REF_VOLTAGE_STRINGS[adc.int_ref_voltage as usize]
                } else {
                    VDD_REF_VOLTAGE_STRING
                },
            ),
            SensorSetting::Prescaler => format_uint(
                buf,
                b"Div:",
                PRESCALER_VALUES[adc.prescaler as usize],
                0,
                None,
            ),
            SensorSetting::InitDelay => format_uint(
                buf,
                b"Idly:",
                INIT_DELAY_VALUES[adc.init_delay as usize],
                0,
                None,
            ),
            SensorSetting::SetAsdv => format_buf(buf, b"Asdv:", BOOL_STRINGS[adc.asdv as usize]),
            SensorSetting::SampleDelay => {
                format_uint(buf, b"Sdly:", adc.sample_delay as u16, 0, None)
            }
            SensorSetting::SampleLength => {
                format_uint(buf, b"Slen:", adc.sample_length as u16, 0, None)
            }
        }
    }

    fn format_reading(&mut self, value: u16, buf: &mut [u8; NUM_CHARS]) {
        let (prefix, value, decimals, suffix) = match (&self.cur_page, &self.show_raw) {
            (SensorPage::AdcChannel(AdcChannel::Temp), show_raw) => (
                b"Tf:",
                value,
                0,
                if *show_raw {
                    None
                } else {
                    Some(if self.show_tempf { b"\x98F" } else { b"\x98C" }.as_slice())
                },
            ),
            #[cfg(feature = "board_v0")]
            (SensorPage::AdcChannel(channel), false) => (
                match channel {
                    AdcChannel::Vext => b"Ve:",
                    AdcChannel::Vref => b"Vr:",
                    AdcChannel::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                value,
                Self::DECIMAL_PRECISION,
                Some(b"V".as_slice()),
            ),
            #[cfg(feature = "board_v0")]
            (SensorPage::AdcChannel(channel), true) => (
                match channel {
                    AdcChannel::Vext => b"Ve:",
                    AdcChannel::Vref => b"Vr:",
                    AdcChannel::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                value,
                0,
                None,
            ),
            #[cfg(not(feature = "board_v0"))]
            (SensorPage::AdcChannel(channel), false) => (
                match channel {
                    AdcChannel::Vsda => b"Vb:",
                    AdcChannel::Vscl => b"Vy:",
                    AdcChannel::Vref => b"Vr:",
                    AdcChannel::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                value,
                Self::DECIMAL_PRECISION,
                Some(b"V".as_slice()),
            ),
            #[cfg(not(feature = "board_v0"))]
            (SensorPage::AdcChannel(channel), true) => (
                match channel {
                    AdcChannel::Vsda => b"Vb:", // standard SDA wire is blue
                    AdcChannel::Vscl => b"Vy:", // standard SCL wire is yellow
                    AdcChannel::Vref => b"Vr:",
                    AdcChannel::Gnd => b"Vg:",
                    _ => unreachable!(),
                },
                value,
                0,
                None,
            ),
            #[cfg(feature = "board_v0")]
            (SensorPage::ContinuityTest, _) => {
                format_buf(buf, b"Cx:", if value > 0 { b"yes" } else { b" no" });
                return;
            }
            #[cfg(not(feature = "board_v0"))]
            (SensorPage::ContinuityTest, _) => {
                let reading = match value {
                    CONTINUITY_BLUE_READING_MASK => b" blue",
                    CONTINUITY_YELLOW_READING_MASK => b"yellw",
                    CONTINUITY_BOTH_READING_MASK => b"bl+yl",
                    _ => b" none",
                };
                format_buf(buf, b"Cx:", reading);
                return;
            }
        };

        format_uint(buf, prefix, value, decimals, suffix);
    }

    fn decrement_cur_setting(&mut self, adc: &mut AdcSettings) {
        self.adjust_cur_setting(adc, false);
    }

    fn increment_cur_setting(&mut self, adc: &mut AdcSettings) {
        self.adjust_cur_setting(adc, true);
    }

    fn adjust_cur_setting(&mut self, adc: &mut AdcSettings, increment: bool) {
        match self.cur_setting {
            SensorSetting::Resolution => adc.resolution = if increment { adc.resolution.next() } else { adc.resolution.prev() },
            SensorSetting::SampleNumber => adc.sample_number = if increment { adc.sample_number.next() } else { adc.sample_number.prev() },
            SensorSetting::SampCap => adc.samp_cap = !adc.samp_cap,
            SensorSetting::RefVoltage => {
                if increment {
                    if adc.adc_ref_voltage == AdcReferenceVoltage::INTREF {
                        if adc.int_ref_voltage == IntReferenceVoltage::_4V34 {
                            adc.adc_ref_voltage = AdcReferenceVoltage::VDDREF;
                        } else {
                            adc.int_ref_voltage = adc.int_ref_voltage.next();
                        }
                    }
                } else {
                    if adc.adc_ref_voltage == AdcReferenceVoltage::VDDREF {
                        adc.adc_ref_voltage = AdcReferenceVoltage::INTREF;
                        adc.int_ref_voltage = IntReferenceVoltage::_4V34;
                    } else {
                        adc.int_ref_voltage = adc.int_ref_voltage.prev();
                    }
                }
            }
            SensorSetting::Prescaler => adc.prescaler = if increment { adc.prescaler.next() } else { adc.prescaler.prev() },
            SensorSetting::InitDelay => adc.init_delay = if increment { adc.init_delay.next() } else { adc.init_delay.prev() },
            SensorSetting::SetAsdv => adc.asdv = !adc.asdv,
            SensorSetting::SampleDelay => adc.sample_delay = if increment { (adc.sample_delay + 1).min(15) } else { adc.sample_delay.saturating_sub(1) },
            SensorSetting::SampleLength => adc.sample_length = if increment { (adc.sample_length + 1).min(31) } else { adc.sample_length.saturating_sub(1) },
        }
    }

    fn toggle_reading_format(&mut self) {
        match self.cur_page {
            SensorPage::AdcChannel(AdcChannel::Temp) => {
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
        let mut update = context.need_update();

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    // in settings: exit to readings and apply settings, in readings: exit to menu
                    if self.settings_active {
                        self.settings_active = false;
                        peripherals.adc.apply_settings();
                        update = true;
                    } else {
                        // disable ADC when leaving utils mode
                        self.settings_active = false;
                        peripherals.adc.disable();
                        peripherals.i2c.pins_to_floating();
                        peripherals.buzzer.no_tone();
                        context.to_menu();
                        return;
                    }
                }
                Event::RightHeld => {
                    // toggle setting or toggle into settings
                    if self.settings_active {
                        self.cur_setting = self.cur_setting.next_wrapping();
                    } else {
                        self.settings_active = true;
                    }
                    update = true;
                }
                Event::LeftReleased => {
                    if self.settings_active {
                        self.decrement_cur_setting(&mut peripherals.adc.settings);
                    } else {
                        self.toggle_reading_format();
                    }
                    update = true;
                }
                Event::RightReleased => {
                    if self.settings_active {
                        self.increment_cur_setting(&mut peripherals.adc.settings);
                    } else {
                        self.cur_page = self.cur_page.next();
                        self.port_init = false;
                        self.last_reading = 0;
                        peripherals.buzzer.no_tone();
                        self.tone_active = 0;
                        context
                            .settings
                            .save_setting_byte(Setting::SensorPage, self.cur_page.into());
                    }
                    update = true;
                }
                _ => {}
            }
        }

        // set up pins based on cur page
        if !self.port_init {
            match self.cur_page {
                SensorPage::AdcChannel(_) => {
                    // floating for voltage test
                    peripherals.i2c.pins_to_floating();
                }
                SensorPage::ContinuityTest => {
                    // pullup for continuity test
                    peripherals.i2c.pins_to_pullup();
                }
            }
            self.port_init = true;
        }

        // check for new ADC reading
        if !update && !self.settings_active {
            let reading = match self.cur_page {
                SensorPage::AdcChannel(channel) => {
                    if self.show_raw {
                        peripherals.adc.read_raw_nonblocking(channel)
                    } else {
                        match channel {
                            AdcChannel::Temp => {
                                peripherals.adc.read_temp_nonblocking(self.show_tempf)
                            }
                            _ => peripherals.adc.read_voltage_nonblocking(channel),
                        }
                    }
                }
                #[cfg(feature = "board_v0")]
                SensorPage::ContinuityTest => peripherals
                    .adc
                    .read_raw_nonblocking(AdcChannel::Vext)
                    .map(|r| if r == 0u16 { 1 } else { 0 }),
                #[cfg(not(feature = "board_v0"))]
                SensorPage::ContinuityTest => {
                    let (channel, mask) = if self.continuity_channel {
                        (AdcChannel::Vsda, CONTINUITY_YELLOW_READING_MASK)
                    } else {
                        (AdcChannel::Vscl, CONTINUITY_BLUE_READING_MASK)
                    };

                    peripherals.adc.read_raw_nonblocking(channel).map(|r| {
                        self.continuity_channel = !self.continuity_channel;
                        if r == 0 { self.last_reading | mask } else { self.last_reading & !mask }
                    })
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
                self.format_setting(&mut buf, &peripherals.adc.settings);
            } else {
                // update buzzer for continuity test
                if matches!(self.cur_page, SensorPage::ContinuityTest) {
                    if (self.last_reading as u8) != self.tone_active {
                        #[cfg(feature = "board_v0")] {
                            if self.last_reading > 0 {
                                self.tone_active = 1;
                                peripherals.buzzer.tone(4000, 0);
                            }
                            else {
                                self.tone_active = 0;
                                peripherals.buzzer.no_tone();
                            }
                        }
                        #[cfg(not(feature = "board_v0"))]
                        {
                            let freq = match self.last_reading {
                                CONTINUITY_BLUE_READING_MASK => 3000,
                                CONTINUITY_YELLOW_READING_MASK => 4000,
                                CONTINUITY_BOTH_READING_MASK => 5000,
                                _ => 0,
                            };
                            self.tone_active = self.last_reading as u8;
                            peripherals.buzzer.tone(freq, 0);
                        }
                    }
                } else if self.tone_active > 0 {
                    peripherals.buzzer.no_tone();
                    self.tone_active = 0;
                }
                self.format_reading(self.last_reading, &mut buf);
            }

            peripherals.display.print_ascii_bytes(&buf).unwrap();
        }
    }
}
