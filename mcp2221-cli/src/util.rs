#[derive(Debug)]
pub(crate) enum ParseError {
    NotHex,
    InvalidInteger(std::num::ParseIntError),
    InvalidSevenBitAddress,
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidInteger(value)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidInteger(parse_int_error) => write!(f, "{parse_int_error}"),
            ParseError::InvalidSevenBitAddress => write!(f, "not a valid 7-bit I2C address"),
            ParseError::NotHex => write!(f, "hex values must start with 0x"),
        }
    }
}

impl std::error::Error for ParseError {}

pub(crate) fn u16_from_hex(value: &str) -> Result<u16, ParseError> {
    let s = if value.to_ascii_lowercase().starts_with("0x") {
        &value[2..]
    } else {
        return Err(ParseError::NotHex);
    };
    u16::from_str_radix(s, 16).map_err(ParseError::from)
}

pub(crate) fn u8_from_hex(value: &str) -> Result<u8, ParseError> {
    let s = if value.to_ascii_lowercase().starts_with("0x") {
        &value[2..]
    } else {
        return Err(ParseError::NotHex);
    };
    u8::from_str_radix(s, 16).map_err(ParseError::from)
}

pub(crate) fn seven_bit_address(value: &str) -> Result<u8, ParseError> {
    let address = u8_from_hex(value)?;
    if address > 127 {
        // Not a 7-bit address.
        eprintln!("{address} is not a valid seven-bit address");
        return Err(ParseError::InvalidSevenBitAddress);
    }
    Ok(address)
}
