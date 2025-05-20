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
    /// An error occurred when attempting to open the MCP2221 USB device.
    HidApi(hidapi::HidError),
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
            Error::HidApi(hid_error) => write!(f, "HidApi error: {hid_error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::HidApi(hid_error) => Some(hid_error),
            _ => None,
        }
    }
}
