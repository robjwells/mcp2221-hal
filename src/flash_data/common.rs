#[derive(Debug, Clone, Copy)]
pub enum ChipConfigurationSecurity {
    PermanentlyLocked,
    PasswordProtected,
    Unsecured,
}

impl From<u8> for ChipConfigurationSecurity {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Unsecured,
            0b01 => Self::PasswordProtected,
            0b10 | 0b11 => Self::PermanentlyLocked,
            _ => unreachable!(),
        }
    }
}

impl From<ChipConfigurationSecurity> for u8 {
    fn from(value: ChipConfigurationSecurity) -> Self {
        match value {
            ChipConfigurationSecurity::PermanentlyLocked => 0b10,
            ChipConfigurationSecurity::PasswordProtected => 0b01,
            ChipConfigurationSecurity::Unsecured => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Setting of the internal voltage reference (VRM)
pub enum VrmVoltageReference {
    /// 4.096V
    ///
    /// Only available if VDD is above this voltage.
    V4_096,
    /// 2.048V
    V2_048,
    /// 1.024V
    V1_024,
    Off,
}

impl From<u8> for VrmVoltageReference {
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

impl From<VrmVoltageReference> for u8 {
    fn from(value: VrmVoltageReference) -> Self {
        match value {
            VrmVoltageReference::V4_096 => 0b11,
            VrmVoltageReference::V2_048 => 0b10,
            VrmVoltageReference::V1_024 => 0b01,
            VrmVoltageReference::Off => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DacVoltageReferenceSource {
    Vrm,
    Vdd,
}

impl From<bool> for DacVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::Vrm } else { Self::Vdd }
    }
}

impl From<DacVoltageReferenceSource> for bool {
    fn from(value: DacVoltageReferenceSource) -> Self {
        // This is the opposite to what the bit means when reading(!) and when reading &
        // writing SRAM. I've asked about it on the Microchip forum but for now I'll
        // follow the datasheet.
        //
        // Datasheet says:
        //
        // - Register 1-3, bit 5, DACREF:            1 = VRM, 0 = VDD
        // - Read flash (table 3-5), byte 6 bit 2:   1 = VRM, 0 = VDD
        // - Write flash (table 3-12), byte 5 bit 2: 1 = VDD, 0 = VRM
        // - Set SRAM (table 3-36), byte 3, bit 0:   1 = VRM, 0 = VDD
        // - Get SRAM (table 3-39), byte 6, bit 5:   1 = VRM, 0 = VDD
        match value {
            DacVoltageReferenceSource::Vrm => false,
            DacVoltageReferenceSource::Vdd => true,
        }
    }
}

// Necessary because the DAC and ADC have inverted meanings for
// voltage reference source = 1.
#[derive(Debug, Clone, Copy)]
pub enum AdcVoltageReferenceSource {
    Vrm,
    Vdd,
}

impl From<bool> for AdcVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::Vdd } else { Self::Vrm }
    }
}

impl From<AdcVoltageReferenceSource> for bool {
    fn from(value: AdcVoltageReferenceSource) -> Self {
        // Datasheet says:
        //
        // - Register 1-4, bit 2, ADCREF:               1 = VRM, 0 = VDD
        // - Read flash (table 3-5), byte 7 bit 2:      1 = VDD, 0 = VRM
        // - Write flash (table 3-12), byte 5 bit 2:    1 = VRM, 0 = VDD
        // - Set SRAM (table 3-36), byte 5, bit 0:      1 = VRM, 0 = VDD (says both!?)
        // - Get SRAM (table 3-39), byte 7, bit 2:      1 = VRM, 0 = VDD
        match value {
            AdcVoltageReferenceSource::Vrm => true,
            AdcVoltageReferenceSource::Vdd => false,
        }
    }
}
