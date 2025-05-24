use std::string::FromUtf16Error;

/// Wrapper for problems when communicating with the MCP2221.
#[derive(Debug)]
pub enum Error {
    /// A command issued to the MCP2221 via USB HID did not complete successfully.
    ///
    /// The enclosed `u8` is the value returned by the MCP2221 in place of the success
    /// code (0).
    CommandFailed(u8),
    /// An unsupported command was issued to the MCP2221.
    ///
    /// This can occur when reading or writing the flash data. This error indicates
    /// a bug in the `mcp2221-hal` library.
    CommandNotSupported,
    /// A disallowed command was issued to the MCP2221.
    ///
    /// This can occur when writing the flash data, and appears to indicate that the
    /// device is permanently locked after repeated failed password entries. See
    /// section 3.1.4.1 in the datasheet.
    CommandNotAllowed,
    /// The I2C bus speed could not be changed because a transfer was in progress.
    ///
    /// This can only occur when attempting to change the I2C bus speed.
    I2cTransferPreventedSpeedChange,
    /// The command code echoed by the MCP2221 was not the command code written to it.
    ///
    /// In practice this should not occur(!). Please report any occurrences.
    MismatchedCommandCodeEcho {
        /// Command code that was sent to the MCP2221.
        sent: u8,
        /// Command code echoed from the MCP2221.
        received: u8,
    },
    /// String received from the MCP2221 was not valid UTF-16.
    InvalidStringFromDevice(FromUtf16Error),
    /// Invalid pin mode bit pattern received from the MCP2221.
    InvalidPinModeFromDevice {
        /// MCP2221 pin name, eg "GP0".
        pin: &'static str,
        /// Invalid bit pattern received for the pin's mode/designation.
        mode: u8,
    },
    /// An error occurred when attempting to open the MCP2221 USB device.
    HidApi(hidapi::HidError),
    /// I2C engine could not handle the request because it was busy.
    I2cEngineBusy,
    /// I2C target data could not be read from the I2C engine.
    I2cEngineReadError,
    /// The number of bytes to read from or write to an I2C target was more than 65,535.
    I2cTransferTooLong,
    /// The number of bytes to write to or read from an I2C target was 0.
    I2cTransferEmpty,
    /// Exhausted retries for an I2C operation.
    I2cOperationFailed,
    /// I2C target address didn't acknowledge its address.
    I2cAddressNack,
    /// Attempt to perform an I2C transaction that is not possible via the MCP2221.
    ///
    /// Specifically, a read occurs before a write in the slice of operations passed
    /// to this library's implementation of [`embedded_hal::i2c::I2c::transaction`].
    ///
    /// The embedded-hal I2C transaction contract cannot be fulfilled in its most
    /// general form by the MCP2221, because it has no HID command to perform a read
    /// without a final STOP condition.
    ///
    /// This library does not attempt to "fake" the transaction because it would mean
    /// introducing STOP conditions that would violate the documented contract, and
    /// could be interpreted in an unexpected way by an I2C target.
    ///
    /// If you need to perform a transaction where a read takes place before a write
    /// without an STOP condition in between, you should use a device other than the
    /// MCP2221.
    I2cUnsupportedEmbeddedHalTransaction,
    /// A GP pin's mode was changed while a mode-specific wrapper type was in use,
    /// rendering it unable to perform its mode-specific functions.
    PinModeChanged,
}

#[doc(hidden)]
impl From<hidapi::HidError> for Error {
    fn from(value: hidapi::HidError) -> Self {
        Self::HidApi(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CommandFailed(e) => write!(
                f,
                "MCP2221 command did not complete successfully and returned error code {e:#X}.",
            ),
            Error::CommandNotSupported => write!(
                f,
                "command rejected by the MCP2221 because it is unsupported"
            ),
            Error::CommandNotAllowed => write!(
                f,
                "command rejected by the MCP2221 because it is not allowed"
            ),
            Error::I2cTransferPreventedSpeedChange => write!(
                f,
                "I2C bus speed could not be changed because a transfer is in progress",
            ),
            Error::MismatchedCommandCodeEcho { sent, received } => write!(
                f,
                "incorrect command code echo from the MCP2221 (got {received:#X}, expected {sent:#X})",
            ),
            Error::InvalidStringFromDevice(e) => {
                write!(f, "invalid utf-16 string received from the MCP2221: {e}")
            }
            Error::InvalidPinModeFromDevice { pin, mode } => write!(
                f,
                "invalid pin mode bit pattern {mode:#b} received for {pin}"
            ),
            Error::HidApi(hid_error) => write!(f, "HidApi error: {hid_error}"),
            Error::I2cEngineBusy => write!(f, "I2C engine busy"),
            Error::I2cEngineReadError => {
                write!(f, "could not read I2C target data from the I2C engine")
            }
            Error::I2cTransferTooLong => {
                write!(
                    f,
                    "attempt to transfer than 65,535 bytes to or from I2C target"
                )
            }
            Error::I2cTransferEmpty => {
                write!(f, "zero-length I2C transfers are not supported")
            }
            Error::I2cOperationFailed => {
                write!(f, "all retries exhausted attempt to perform I2C operation")
            }
            Error::I2cAddressNack => {
                write!(f, "I2C target didn't acknowledge its address")
            }
            Error::I2cUnsupportedEmbeddedHalTransaction => {
                write!(f, "I2C transaction operations mixed in an unsupported way")
            }
            Error::PinModeChanged => {
                write!(
                    f,
                    "pin mode was changed while a mode-specific wrapper was in use"
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidStringFromDevice(utf_error) => Some(utf_error),
            Error::HidApi(hid_error) => Some(hid_error),
            _ => None,
        }
    }
}
