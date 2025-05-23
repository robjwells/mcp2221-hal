//! I2C engine configuration.

use crate::commands::McpCommand;

/// Response from the MCP2221 after attempting to cancel an I2C transfer.
#[derive(Debug)]
pub enum CancelI2cTransferResponse {
    /// Cancellation successful.
    Done,
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

#[allow(dead_code)]
pub(crate) trait I2cAddressing {
    /// Shift a seven-bit address over, with the R/_W bit set to 1 (read).
    fn into_read_address(self) -> u8;
    /// Shift a seven-bit address over, with the R/_W bit set to 0 (write).
    fn into_write_address(self) -> u8;
}

impl I2cAddressing for u8 {
    fn into_read_address(self) -> u8 {
        (self << 1) + 1
    }

    fn into_write_address(self) -> u8 {
        self << 1
    }
}

/// Specific I2C read type that has a corresponding HID command.
///
/// The HID commands have different command bytes but otherwise identical arguments.
pub(crate) enum ReadType {
    /// Read with a START and STOP condition.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.8 for the underlying HID command.
    Normal,
    /// Read with a repeated START condition and a STOP condition.
    ///
    /// In the I2C protocol, a repeated start is just a START condition where a STOP
    /// condition has not terminated the previous transfer. Formally, it _should_
    /// be no different from a "normal" read, but it is not clear from the datasheet
    /// if the MCP2221 treats a repeated start in a special manner.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.9 for the underlying I2C command.
    RepeatedStart,
}

impl From<ReadType> for McpCommand {
    fn from(value: ReadType) -> Self {
        match value {
            ReadType::Normal => McpCommand::I2cReadData,
            ReadType::RepeatedStart => McpCommand::I2cReadDataRepeatedStart,
        }
    }
}

/// Specific I2C write type that has a corresponding HID command.
///
/// The HID commands have different command bytes but otherwise identical arguments.
pub(crate) enum WriteType {
    /// Write with a START and STOP condition.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.5 for the underlying HID command.
    Normal,
    /// Write with a repeated START condition and a STOP condition.
    ///
    /// In the I2C protocol, a repeated start is just a START condition where a STOP
    /// condition has not terminated the previous transfer. Formally, it _should_
    /// be no different from a "normal" write, but it is not clear from the datasheet
    /// if the MCP2221 treats a repeated start in a special manner.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.6 for the underlying HID command.
    RepeatedStart,
    /// Write with a START condition but _without_ a STOP condition.
    ///
    /// This is a component of a write-read where a write is issued to a device, then
    /// immediately afterwards a read (repeated-START), without releasing the bus.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.7 for the underlying HID command.
    NoStop,
}

impl From<WriteType> for McpCommand {
    fn from(value: WriteType) -> Self {
        match value {
            WriteType::Normal => McpCommand::I2cWriteData,
            WriteType::RepeatedStart => McpCommand::I2cWriteDataRepeatedStart,
            WriteType::NoStop => McpCommand::I2cWriteDataNoStop,
        }
    }
}
