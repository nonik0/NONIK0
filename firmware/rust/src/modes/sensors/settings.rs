pub const REF_VOLTAGE_VARIANTS: [avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A; 5] = [
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_0V55,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_1V1,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_2V5,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_4V34,
    avrxmega_hal::pac::vref::ctrla::ADC0REFSEL_A::_1V5,
];
pub const RESOLUTION_VARIANTS: [avrxmega_hal::pac::adc0::ctrla::RESSEL_A; 2] = [
    avrxmega_hal::pac::adc0::ctrla::RESSEL_A::_10BIT,
    avrxmega_hal::pac::adc0::ctrla::RESSEL_A::_8BIT,
];
pub const SAMPLE_NUMBER_VARIANTS: [avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A; 7] = [
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC1,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC2,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC4,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC8,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC16,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC32,
    avrxmega_hal::pac::adc0::ctrlb::SAMPNUM_A::ACC64,
];
pub const CLOCK_DIVIDER_VARIANTS: [avrxmega_hal::pac::adc0::ctrlc::PRESC_A; 8] = [
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV2,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV4,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV8,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV16,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV32,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV64,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV128,
    avrxmega_hal::pac::adc0::ctrlc::PRESC_A::DIV256,
];
pub const INIT_DELAY_VARIANTS: [avrxmega_hal::pac::adc0::ctrld::INITDLY_A; 6] = [
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY0,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY16,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY32,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY64,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY128,
    avrxmega_hal::pac::adc0::ctrld::INITDLY_A::DLY256,
];
pub const VREF_E5_VALUES: [u32; 6] = [55000, 110000, 150000, 250000, 434000, 360000];
pub const RESOLUTION_VALUES: [u16; 2] = [10, 8]; // 2^10 - 1, 2^8 - 1
pub const SAMPLE_NUMBER_DIVISORS: [u16; 7] = [1, 2, 4, 8, 16, 32, 64];
pub const CLOCK_DIVIDER_VALUES: [u16; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
pub const INIT_DELAY_VALUES: [u16; 6] = [0, 16, 32, 64, 128, 256];

pub const REF_VOLTAGE_STRINGS: [&[u8]; 6] = [b"0.55V", b"1.1V", b"1.5V", b"2.5V", b"4.34V", b"Vdd"];
pub const BOOL_STRINGS: [&[u8]; 2] = [b" no", b"yes"];

#[derive(Clone, Copy)]
pub enum SensorReading {
    Temp = 0,
    Vext = 1,
    Vref = 2,
    Gnd = 3,
}

impl SensorReading {
    pub fn next(&self) -> Self {
        match self {
            SensorReading::Temp => SensorReading::Vext,
            SensorReading::Vext => SensorReading::Vref,
            SensorReading::Vref => SensorReading::Gnd,
            SensorReading::Gnd => SensorReading::Temp,
        }
    }
}

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

impl SensorSetting {
    pub fn next(&self) -> Self {
        match self {
            SensorSetting::Resolution => SensorSetting::SampleNumber,
            SensorSetting::SampleNumber => SensorSetting::SampCap,
            SensorSetting::SampCap => SensorSetting::RefVoltage,
            SensorSetting::RefVoltage => SensorSetting::Prescaler,
            SensorSetting::Prescaler => SensorSetting::InitDelay,
            SensorSetting::InitDelay => SensorSetting::SetAsdv,
            SensorSetting::SetAsdv => SensorSetting::SampleDelay,
            SensorSetting::SampleDelay => SensorSetting::SampleLength,
            SensorSetting::SampleLength => SensorSetting::Resolution,
        }
    }
}

