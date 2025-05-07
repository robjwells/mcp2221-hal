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
    Vrm,
    Vdd,
}

impl From<bool> for VoltageReference {
    fn from(value: bool) -> Self {
        if value { Self::Vrm } else { Self::Vdd }
    }
}

impl From<VoltageReference> for bool {
    fn from(value: VoltageReference) -> Self {
        // Note that table 3-12 byte 5 lists 1 = VDD, 0 = VRM. This is the opposite
        // to all other uses in the datasheet and appears to be an error, as is
        // table 3-5 byte 7 (read flash data ADC reference).
        match value {
            VoltageReference::Vrm => true,
            VoltageReference::Vdd => false,
        }
    }
}
