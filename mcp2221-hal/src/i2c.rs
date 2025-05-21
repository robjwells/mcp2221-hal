//! I2C engine configuration.

/// Response from the MCP2221 after attempting to cancel an I2C transfer.
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

/// I2C bus speed.
///
/// Speeds up to 400,000 bps are supported. The slowest possible speed is 46,692 bps.
#[derive(Debug)]
pub struct I2cSpeed(u32);

impl I2cSpeed {
    // 12 MHz internal clock.
    const MCP_CLOCK: u32 = 12_000_000;
    // I2C engine maximum speed is 400kbps.
    const MAX_SPEED: u32 = 400_000;
    // I2C engine minimum speed limited by the u8 size.
    const MIN_SPEED: u32 = Self::divider_to_speed(255);

    // Compute standard I2C bus speeds at compile time.
    const STANDARD_DIVIDER: u8 = Self::speed_to_divider(100_000);
    const FAST_DIVIDER: u8 = Self::speed_to_divider(400_000);

    /// Transform an I2C bus speed (in Hz or bps) into an appropriate divider.
    const fn speed_to_divider(speed: u32) -> u8 {
        // Formula from Note 1 in table 3-1 in the datasheet.
        ((Self::MCP_CLOCK / speed) - 2) as u8
    }

    /// Transform a divider into a speed in kbps.
    const fn divider_to_speed(divider: u8) -> u32 {
        Self::MCP_CLOCK / (divider as u32 + 2)
    }

    /// Convert the speed mode into a clock divider suitable for the
    /// Status/Set Parameters HID command.
    pub(crate) fn to_clock_divider(&self) -> u8 {
        match self.0 {
            400_000 => Self::FAST_DIVIDER,
            100_000 => Self::STANDARD_DIVIDER,
            speed => Self::speed_to_divider(speed),
        }
    }

    /// Speed in bits per second (bps).
    pub fn speed(&self) -> u32 {
        self.0
    }

    /// Create a new I2cSpeed struct within the possible speed range.
    ///
    /// Note that the MCP2221 fastest I2C bus speed is 400,000 bps, and the slowest
    /// is 46,692 bps. Values outside this range will be clamped to it.
    ///
    /// The maximum speed is a limitation of the chip, the slowest due to the 8-bit
    /// width of the speed divider.
    pub fn new(speed: u32) -> Self {
        let speed = speed.clamp(Self::MIN_SPEED, Self::MAX_SPEED);
        Self(speed)
    }

    /// "Fast-mode" I2C bus speed of 400kbps.
    pub fn fast_400k() -> Self {
        Self(400_000)
    }

    /// "Standard-mode" I2C bus speed of 100kbps.
    pub fn standard_100k() -> Self {
        Self(100_000)
    }
}

/// Construct the bus speed from a divider.
///
/// Note that this doesn't round-trip invalid dividers (below that for 400kbps).
#[doc(hidden)]
impl From<u8> for I2cSpeed {
    fn from(divider: u8) -> Self {
        Self::new(Self::divider_to_speed(divider))
    }
}
