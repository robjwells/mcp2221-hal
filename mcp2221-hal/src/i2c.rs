//! I2C engine status and configuration types.
//!
//! [`I2cSpeed`] is the only type in this module users will construct themselves,
//! and is used to set the I2C bus clock frequency (and therefore data rate)
//! via [`MCP2221::i2c_set_bus_speed`].
//!
//! [`I2cStatus`] and [`I2cCommunicationState`] form part of the [`Status`] response,
//! while [`I2cCancelTransferResponse`] is returned by [`MCP2221::i2c_cancel_transfer`].
//!
//! [`MCP2221::i2c_set_bus_speed`]: crate::MCP2221::i2c_set_bus_speed
//! [`MCP2221::i2c_cancel_transfer`]: crate::MCP2221::i2c_cancel_transfer
//! [`Status`]: crate::status::Status

use crate::commands::McpCommand;

/// I2C engine status
///
/// This struct bundles together the I2C-related fields of the Status command response.
///
/// ## Datasheet
///
/// See the return value of the Status/Set Parameters HID command in table 3-2 of
/// section 3.1.1 for the definition of the fields.
#[derive(Debug)]
pub struct I2cStatus {
    /// I2C engine communication state.
    ///
    /// Records whether the I2C engine is idle or a timeout has occurred.
    pub communication_state: I2cCommunicationState,
    /// The current requested I2C transfer length.
    pub transfer_requested_length: u16,
    /// Number of bytes already transferred in the current transfer.
    pub transfer_completed_length: u16,
    /// Internal I2C data buffer counter.
    ///
    /// This field is not explained in the datasheet.
    pub internal_data_buffer_counter: u8,
    /// I2C bus speed.
    pub bus_speed: I2cSpeed,
    /// I2C engine timeout value.
    ///
    /// It is not explained in the datasheet how this value should be interpreted.
    /// It does not seem to be a length of time after which timeout occurs, perhaps
    /// it is some internal enum value.
    pub timeout_value: u8,
    /// I2C target address used most recently.
    pub target_address: u16,
    /// I2C target acknowledged its address.
    ///
    /// This is not further explained in the datasheet, but is described in the Java
    /// driver as being the ACK to the I2C target ("slave") address.
    pub target_acknowledged_address: bool,
    /// I2C clock line is high.
    ///
    /// I2C is an idle-high bus, so this can be used to detect if an I2C device is
    /// holding the clock line (SCL) low.
    pub scl_line_high: bool,
    /// I2C data line is high.
    ///
    /// I2C is an idle-high bus, so this can be used to detect if an I2C device is
    /// holding the data line (SDA) low.
    pub sda_line_high: bool,
    /// I2C Read pending value.
    ///
    /// The datasheet describes this as being "used by the USB host to know if the
    /// MCP2221 still has to read from a slave device", with possible values of 0, 1
    /// or 2. The meaning of those values is not explained in the datasheet.
    ///
    /// Neither the Microchip C or Java drivers read this value.
    pub read_pending_value: u8,
}

/// State of the I2C engine.
///
/// Most of the cases are guesswork from constant names in the Microchip C and Android
/// Java drivers, and their meaning is not documented. The datasheet only says that a
/// state other than 0 is a timeout.
#[derive(Debug)]
pub enum I2cCommunicationState {
    /// Engine is idle.
    Idle,
    /// I2CM_SM_START_TOUT.
    StartTimeout,
    /// I2CM_SM_REPSTART_TOUT.
    RepeatedStartTimeout,
    /// I2CM_SM_WRADDL_WAITSEND.
    WriteAddressWaitSend,
    /// Target address timeout.
    ///
    /// - RESP_I2C_WRADDRL_TOUT in C.
    /// - I2CM_SM_WRADDRL_TOUT in Java.
    AddressTimeout,
    /// Target did not acknowledge its address.
    ///
    /// - RESP_ADDR_NACK in C.
    /// - I2CM_SM_WRADDL_NACK_STOP in Java.
    AddressNack,
    /// I2CM_SM_WRITEDATA_TOUT.
    WriteDataTimeout,
    /// Engine has finished sending data from an I2C Write Data No Stop command.
    ///
    /// I2CM_SM_WRITEDATA_END_NOSTOP.
    WriteDataEndNoStop,
    /// I2CM_SM_READDATA_TOUT.
    ReadDataTimeout,
    /// I2CM_SM_STOP_TOUT.
    StopTimeout,
    /// I2C engine is in some other state.
    Other(u8),
}

impl I2cCommunicationState {
    /// I2C engine is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// Turn the receive byte into the communication state.
///
/// The hex values are from the Microchip C and Java drivers.
#[doc(hidden)]
impl From<u8> for I2cCommunicationState {
    fn from(value: u8) -> Self {
        // Hex values taken from the Java driver.
        match value {
            0x00 => Self::Idle,
            0x12 => Self::StartTimeout,
            0x17 => Self::RepeatedStartTimeout,
            0x21 => Self::WriteAddressWaitSend,
            0x23 => Self::AddressTimeout,
            0x25 => Self::AddressNack,
            0x44 => Self::WriteDataTimeout,
            0x45 => Self::WriteDataEndNoStop,
            0x52 => Self::ReadDataTimeout,
            0x62 => Self::StopTimeout,
            n => Self::Other(n),
        }
    }
}

/// Response from the MCP2221 to an attempt to cancel an I2C transfer.
#[derive(Debug)]
pub enum I2cCancelTransferResponse {
    /// Cancellation successful.
    Done,
    /// The transfer cancellation was issued and the MCP2221 marked the transfer for
    /// cancellation. It may take some amount of time for the cancellation to occur.
    MarkedForCancellation,
    /// Either no transfer cancellation was issued by the driver because the I2C engine
    /// was already idle (to avoid what appears to be buggy behaviour of the MCP2221),
    /// or the cancellation was issued and the MCP2221 responded that no transfer was
    /// taking place.
    NoTransfer,
}

/// I2C bus speed.
///
/// Speeds up to 400 kbit/s are supported. The slowest possible speed is 46,692 bit/s
/// due to the MCP2221’s use of an 8-bit clock divider. Speeds outside this range
/// will be limited to the upper or lower limits as appropriate.
///
/// It is recommended that you use the [`I2cSpeed::fast_400k`] and [`I2cSpeed::standard_100k`]
/// constructors for those standardised bus speeds.
///
/// Note that, because of the use of the divider, not all desired bus speeds can be
/// achieved exactly.
///
/// ## Datasheet
///
/// The formula for computing the divider is given in a note under table 3-1:
///
/// ```plain
/// Divider = (12 MHz / I2C clock rate) - 2
/// ```
///
/// Note that Microchips’s own drivers (which predate the inclusion of the above
/// formula in the datasheet) instead subtract 3. It is unclear why they differ.
#[derive(Debug)]
pub struct I2cSpeed(
    /// Desired bus speed in bit/s
    u32,
);

impl I2cSpeed {
    // 12 MHz internal clock.
    //
    // Note that it's not clear that the internal clock is 12MHz (the clock output
    // divider suggests it is 48MHz), but this is the value given in the note under
    // table 3-1.
    const MCP_CLOCK: u32 = 12_000_000;
    // I2C engine maximum speed is 400 kbit/s.
    const MAX_SPEED: u32 = 400_000;
    // I2C engine minimum speed limited by the u8 size.
    const MIN_SPEED: u32 = Self::divider_to_speed(255);

    // Compute standard I2C bus speeds at compile time.
    const FAST_DIVIDER: u8 = Self::speed_to_divider(400_000);
    const STANDARD_DIVIDER: u8 = Self::speed_to_divider(100_000);

    /// Transform an I2C bus speed (in Hz or bit/s) into an appropriate divider.
    const fn speed_to_divider(speed: u32) -> u8 {
        // Formula from Note 1 in table 3-1 in the datasheet.
        ((Self::MCP_CLOCK / speed) - 2) as u8
    }

    /// Transform a divider into a speed in kbit/s.
    const fn divider_to_speed(divider: u8) -> u32 {
        Self::MCP_CLOCK / (divider as u32 + 2)
    }

    /// Convert the speed in bit/s into a clock divider suitable for the
    /// Status/Set Parameters HID command.
    pub(crate) fn to_clock_divider(&self) -> u8 {
        match self.0 {
            400_000 => Self::FAST_DIVIDER,
            100_000 => Self::STANDARD_DIVIDER,
            speed => Self::speed_to_divider(speed),
        }
    }

    /// "Fast-mode" I2C bus speed of 400 kbit/s.
    pub fn fast_400k() -> Self {
        Self(400_000)
    }

    /// "Standard-mode" I2C bus speed of 100 kbit/s.
    pub fn standard_100k() -> Self {
        Self(100_000)
    }

    /// Create a new `I2cSpeed` struct within the possible speed range.
    ///
    /// Note that the MCP2221’s fastest supported bus speed is 400 kbit/s, and the
    /// slowest is 46,692 bit/s. Values outside this range will be clamped to it.
    ///
    /// The maximum speed is a limitation of the chip, the slowest due to the 8-bit
    /// width of the speed divider.
    pub fn new(speed: u32) -> Self {
        let speed = speed.clamp(Self::MIN_SPEED, Self::MAX_SPEED);
        Self(speed)
    }

    /// Speed in bits per second.
    pub fn speed(&self) -> u32 {
        self.0
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

/// Convenience methods for adjusting a 7-bit I2C address into a byte containing the
/// address in the upper 7 bits and the read/_write bit at the bottom.
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
    /// ## Datasheet
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
    /// ## Datasheet
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
    /// ## Datasheet
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
    /// ## Datasheet
    ///
    /// See section 3.1.6 for the underlying HID command.
    RepeatedStart,
    /// Write with a START condition but _without_ a STOP condition.
    ///
    /// This is a component of a write-read where a write is issued to a device, then
    /// immediately afterwards a read (repeated-START), without releasing the bus.
    ///
    /// ## Datasheet
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
