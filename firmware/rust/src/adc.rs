use crate::{impl_enum_cycle, Adc0, Sigrow, Vref};
use avrxmega_hal::pac::{adc0, vref};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AdcChannel {
    Temp,
    Vext,
    Vref,
    Gnd,
}
pub type Resolution = adc0::ctrla::RESSEL_A;
pub type SampleNumber = adc0::ctrlb::SAMPNUM_A;
pub type Prescaler = adc0::ctrlc::PRESC_A;
pub type AdcReferenceVoltage = adc0::ctrlc::REFSEL_A;
pub type InitDelay = adc0::ctrld::INITDLY_A;
pub type IntReferenceVoltage = vref::ctrla::ADC0REFSEL_A;

// helper macro to simply inc/dec enum values for settings
impl_enum_cycle!(AdcChannel, 4);
impl_enum_cycle!(Resolution, 2);
impl_enum_cycle!(SampleNumber, 7);
impl_enum_cycle!(Prescaler, 8);
impl_enum_cycle!(AdcReferenceVoltage, 2); // skips VREFA=3
impl_enum_cycle!(InitDelay, 6);
impl_enum_cycle!(IntReferenceVoltage, 5);

const VREF_E5_VALUES: [u32; 5] = [55000, 110000, 250000, 434000, 150000];
const VREF_VDD_VALUE: u32 = 360000; // ~3.6V for Vdd assuming LIR2032 battery

#[derive(Clone, Copy)]
pub struct AdcSettings {
    // ADC0.CTRLA
    pub resolution: Resolution,
    //pub free_running: bool,
    // ADC0.CTRLB
    pub sample_number: SampleNumber,
    // ADC0.CTRLC
    pub samp_cap: bool,
    pub prescaler: Prescaler,
    pub adc_ref_voltage: AdcReferenceVoltage,
    // VREF.CTRLA
    pub int_ref_voltage: IntReferenceVoltage,
    // ADC0.CTRLD
    pub init_delay: InitDelay,
    pub asdv: bool,       // automatic sampling delay variation
    pub sample_delay: u8, // delay between samples, 4 bit
    // ADC0.CTRLE
    // pub win_comp_mode: WindowComparsionMode,
    // ADC0.SAMPCTRL
    pub sample_length: u8, // extends ADC sample length, 5 bit
}

impl Default for AdcSettings {
    fn default() -> Self {
        AdcSettings {
            resolution: Resolution::_10BIT,
            sample_number: SampleNumber::ACC1,
            samp_cap: true, // Vref default 1.1V > 1.0V
            prescaler: Prescaler::DIV128,
            adc_ref_voltage: AdcReferenceVoltage::INTREF,
            init_delay: InitDelay::DLY0,
            asdv: false,
            sample_delay: 0,
            sample_length: 0,
            int_ref_voltage: IntReferenceVoltage::_1V1,
        }
    }
}

pub struct Adc {
    channel: Option<AdcChannel>,
    settings: AdcSettings,

    adc0: Adc0,
    sigrow: Sigrow,
    vref: Vref,
}

impl Adc {
    const DECIMAL_PRECISION: u16 = 2; // X.YY
    const ADC_SETTINGS: AdcSettings = AdcSettings {
        resolution: Resolution::_10BIT,
        sample_number: SampleNumber::ACC64,
        samp_cap: true,
        prescaler: Prescaler::DIV256,
        adc_ref_voltage: AdcReferenceVoltage::INTREF,
        int_ref_voltage: IntReferenceVoltage::_1V1,
        init_delay: InitDelay::DLY256,
        asdv: false,
        sample_delay: 10,
        sample_length: 10,
    };

    pub fn new(adc0: Adc0, sigrow: Sigrow, vref: Vref) -> Self {
        let adc = Self {
            channel: None,
            settings: Self::ADC_SETTINGS,
            adc0,
            sigrow,
            vref,
        };

        adc.apply_settings(adc.settings);
        adc
    }

    pub fn disable(&mut self) {
        self.adc0.ctrla().write(|w| w.enable().clear_bit());
        self.channel = None;
    }

    pub fn read_raw_nonblocking(&mut self, channel: AdcChannel) -> Option<u16> {
        if self.adc0.command().read().stconv().bit_is_set() {
            return None;
        }
        if self.channel.is_none() {
            // TODO: force apply temp settings?
            let muxpos = match channel {
                AdcChannel::Temp => adc0::muxpos::MUXPOS_A::TEMPSENSE,
                AdcChannel::Vext => adc0::muxpos::MUXPOS_A::AIN10,
                AdcChannel::Vref => adc0::muxpos::MUXPOS_A::INTREF,
                AdcChannel::Gnd => adc0::muxpos::MUXPOS_A::GND,
            };
            self.adc0.muxpos().write(|w| w.muxpos().variant(muxpos));
            self.adc0.command().write(|w| w.stconv().set_bit());
            self.channel = Some(channel);
        }

        let acc_divisor = 1 << (self.settings.sample_number as u8); // TODO: VERIFY THIS
        let raw = self.adc0.res().read().bits() / acc_divisor;
        self.adc0.command().write(|w| w.stconv().set_bit());
        Some(raw)
    }

    pub fn read_temp_nonblocking(&mut self, use_f: bool) -> Option<u16> {
        if let Some(raw) = self.read_raw_nonblocking(AdcChannel::Temp) {
            Some(self.temp_from_raw(raw, use_f))
        } else {
            None
        }
    }

    pub fn read_voltage_nonblocking(&mut self, channel: AdcChannel) -> Option<u16> {
        if let Some(raw) = self.read_raw_nonblocking(channel) {
            Some(self.voltage_from_raw(raw))
        } else {
            None
        }
    }

    pub fn seed_rand(&mut self) {
        let mut sample_count: u32 = 0;
        let mut seed_value: u32 = 0;
        while sample_count < 4 {
            if let Some(reading) = self.read_raw_nonblocking(AdcChannel::Temp) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }
        sample_count = 0;
        while sample_count < 4 {
            if let Some(reading) = self.read_raw_nonblocking(AdcChannel::Vext) {
                seed_value = (seed_value << 4) | (reading as u32 & 0b1111);
                sample_count += 1;
            }
        }
        crate::Rand::seed(seed_value);
    }

    fn voltage_from_raw(&self, raw: u16) -> u16 {
        let raw = raw as u32;
        let vrefe5 = if self.settings.adc_ref_voltage == AdcReferenceVoltage::INTREF {
            VREF_E5_VALUES[self.settings.int_ref_voltage as usize]
        } else {
            VREF_VDD_VALUE
        };
        let raw_max = match self.settings.resolution {
            Resolution::_10BIT => 1023, // 2^10 - 1
            Resolution::_8BIT => 255,   // 2^8 - 1
        };
        let precision_divisor = 10u32.pow(5 - Self::DECIMAL_PRECISION as u32);

        (((raw * vrefe5) / raw_max) / precision_divisor) as u16
    }

    fn temp_from_raw(&mut self, raw: u16, use_f: bool) -> u16 {
        let sigrow_offset = self.sigrow.tempsense1().read().bits() as i8;
        let sigrow_gain = self.sigrow.tempsense0().read().bits() as u8;

        let mut temp: u32 = ((raw as i32) - (sigrow_offset as i32)) as u32;
        temp = (temp as i32 * sigrow_gain as i32) as u32;
        temp += 0x80;
        temp >>= 8;
        let temp_k = temp as u16;
        let temp_c = temp_k.saturating_sub(273); // TODO <0
        if use_f {
            let temp_f = (temp_c as u32 * 9 / 5) + 32;
            temp_f as u16
        } else {
            temp_c
        }
    }

    pub fn apply_settings(&self, settings: AdcSettings) {
        self.vref
            .ctrla()
            .modify(|_, w| w.adc0refsel().variant(self.settings.int_ref_voltage));

        self.adc0.ctrla().write(|w| {
            w.ressel().variant(self.settings.resolution);
            w.enable().set_bit()
        });

        self.adc0
            .ctrlb()
            .write(|w| w.sampnum().variant(self.settings.sample_number));

        self.adc0.ctrlc().write(|w| {
            w.sampcap().bit(settings.samp_cap);
            w.refsel().variant(settings.adc_ref_voltage);
            w.presc().variant(self.settings.prescaler)
        });

        self.adc0.ctrld().write(|w| {
            w.initdly().variant(self.settings.init_delay);
            w.asdv().bit(settings.asdv);
            w.sampdly().set(settings.sample_delay)
        });

        self.adc0
            .sampctrl()
            .write(|w| w.samplen().set(settings.sample_length));
    }
}
