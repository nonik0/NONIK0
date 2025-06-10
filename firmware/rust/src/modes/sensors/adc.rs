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
    pub fn next(&self) -> Self {
        match self {
            Resolution::_8bit => Resolution::_10bit,
            Resolution::_10bit => Resolution::_10bit, // No wrap around
        }
    }

    pub fn prev(&self) -> Self {
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
    pub fn next(&self) -> Self {
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

    pub fn prev(&self) -> Self {
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
    pub fn next(&self) -> Self {
        match self {
            ReferenceVoltage::VRef0_55V => ReferenceVoltage::VRef1_1V,
            ReferenceVoltage::VRef1_1V => ReferenceVoltage::VRef1_5V,
            ReferenceVoltage::VRef1_5V => ReferenceVoltage::VRef2_5V,
            ReferenceVoltage::VRef2_5V => ReferenceVoltage::VRef4_34V,
            ReferenceVoltage::VRef4_34V => ReferenceVoltage::Vdd,
            ReferenceVoltage::Vdd => ReferenceVoltage::Vdd, // No wrap around
        }
    }

    pub fn prev(&self) -> Self {
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
    pub fn next(&self) -> Self {
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

    pub fn prev(&self) -> Self {
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
    pub fn next(&self) -> Self {
        match self {
            DelayCycles::Delay0 => DelayCycles::Delay16,
            DelayCycles::Delay16 => DelayCycles::Delay32,
            DelayCycles::Delay32 => DelayCycles::Delay64,
            DelayCycles::Delay64 => DelayCycles::Delay128,
            DelayCycles::Delay128 => DelayCycles::Delay256,
            DelayCycles::Delay256 => DelayCycles::Delay256, // No wrap around
        }
    }

    pub fn prev(&self) -> Self {
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
