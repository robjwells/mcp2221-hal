/// Three-channel reading of the 10-bit ADC.
///
/// Each channel reading is optional as their values are not defined if the
/// corresponding pin is not configured for ADC operation.
///
/// The channels are named here to match their GP pin (1-3) as given in table 1-1
/// of the datasheet (where channel 1 is read from GP1). Note that in table 3-2
/// the channels are named 0-2.
#[derive(Debug, Clone, Copy)]
pub struct AdcReading {
    /// ADC voltage reference setting in SRAM at the time of the reading.
    pub vref: VoltageReference,
    /// Analog reading from GP1.
    pub gp1: Option<u16>,
    /// Analog reading from GP2.
    pub gp2: Option<u16>,
    /// Analog reading from GP3.
    pub gp3: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
/// Setting of the internal voltage reference (VRM)
pub enum VrmVoltage {
    /// 4.096V
    ///
    /// Only available if VDD is above this voltage.
    V4_096,
    /// 2.048V
    V2_048,
    /// 1.024V
    V1_024,
    /// Reference voltage is off.
    ///
    /// This is useful for the case in which the DAC uses another reference other
    /// than Vrm DAC; eg Vdd.
    Off,
}

impl From<u8> for VrmVoltage {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Off,
            0b01 => Self::V1_024,
            0b10 => Self::V2_048,
            0b11 => Self::V4_096,
            _ => unreachable!(),
        }
    }
}

impl From<VrmVoltage> for u8 {
    fn from(value: VrmVoltage) -> Self {
        match value {
            VrmVoltage::V4_096 => 0b11,
            VrmVoltage::V2_048 => 0b10,
            VrmVoltage::V1_024 => 0b01,
            VrmVoltage::Off => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VoltageReference {
    Vrm(VrmVoltage),
    Vdd,
}

impl From<(bool, u8)> for VoltageReference {
    fn from((source_bit, vrm_level): (bool, u8)) -> Self {
        match source_bit {
            true => Self::Vrm(VrmVoltage::from(vrm_level)),
            false => Self::Vdd,
        }
    }
}

impl From<VoltageReference> for (bool, u8) {
    fn from(value: VoltageReference) -> Self {
        match value {
            VoltageReference::Vrm(vrm_level) => (true, vrm_level.into()),
            VoltageReference::Vdd => (false, VrmVoltage::Off.into()),
        }
    }
}
