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
