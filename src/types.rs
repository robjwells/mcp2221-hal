#[derive(Debug)]
pub enum CancelI2cTransferResponse {
    MarkedForCancellation,
    NoTransfer,
}

#[allow(non_camel_case_types)]
pub enum I2cSpeed {
    /// I2c bus speed of 400kbps ("Fast-mode")
    Fast_400kbps,
    /// I2c bus speed of 100kbps ("Standard-mode")
    Standard_100kbps,
}

impl I2cSpeed {
    /// Convert the speed mode into a clock divider suitable for the
    /// STATUS/SET PARAMETERS command.
    pub(crate) fn to_clock_divider(&self) -> u8 {
        // 12 MHz internal clock.
        const MCP_CLOCK: u32 = 12_000_000;

        // The `-2` part is from Note 1 in Table 3-1 in the datasheet.
        const STANDARD_DIVIDER: u8 = (MCP_CLOCK / 100_000 - 2) as u8;
        const FAST_DIVIDER: u8 = (MCP_CLOCK / 400_000 - 2) as u8;

        match self {
            I2cSpeed::Fast_400kbps => FAST_DIVIDER,
            I2cSpeed::Standard_100kbps => STANDARD_DIVIDER,
        }
    }
}



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
