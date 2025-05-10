#[derive(Debug)]
pub enum CancelI2cTransferResponse {
    /// The I2C transfer cancellation was issued and the MCP2221 marked the transfer
    /// for cancellation.
    MarkedForCancellation,
    /// Either no transfer cancellation was issued by the driver because the I2C engine
    /// was already idle (to avoid what appears to be buggy behaviour of the MCP2221),
    /// or the cancellation was issued and the MCP2221 responded that no transfer was
    /// taking place.
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
