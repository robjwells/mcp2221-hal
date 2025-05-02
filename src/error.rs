#[derive(Debug)]
pub enum Error {
    NoDeviceFound,
    CommandFailed(u8),
    I2cTransferInProgress,
    MismatchedCommandCodeEcho { sent: u8, received: u8 },
    HidApi(hidapi::HidError),
}

impl From<hidapi::HidError> for Error {
    fn from(value: hidapi::HidError) -> Self {
        Self::HidApi(value)
    }
}
