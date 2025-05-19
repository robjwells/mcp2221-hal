//! Chip configuration protection settings.

/// Protection level of the chip configuration.
#[derive(Debug, Clone, Copy)]
pub enum ChipConfigurationSecurity {
    /// Chip settings are permanently locked and no changes can be made.
    ///
    /// This can be set on purpose, and it appears that repeated failed flash settings
    /// updates will also cause the MCP2221 to become permanently locked (see byte 1 of
    /// table 3-19 of the datasheet).
    PermanentlyLocked,
    /// Chip settings are protected by a password.
    PasswordProtected,
    /// No protection mechanism is in place.
    Unsecured,
}

#[doc(hidden)]
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

#[doc(hidden)]
impl From<ChipConfigurationSecurity> for u8 {
    fn from(value: ChipConfigurationSecurity) -> Self {
        match value {
            ChipConfigurationSecurity::PermanentlyLocked => 0b10,
            ChipConfigurationSecurity::PasswordProtected => 0b01,
            ChipConfigurationSecurity::Unsecured => 0b00,
        }
    }
}
