#[derive(Debug)]
pub(crate) enum ParseError {
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
        }
    }
}

impl std::error::Error for ParseError {}
pub(crate) fn from_hex(value: &str) -> Result<u16, std::num::ParseIntError> {
    let s = if value.to_ascii_lowercase().starts_with("0x") {
        &value[2..]
    } else {
        value
    };
    u16::from_str_radix(s, 16)
}

pub(crate) fn seven_bit_address(value: &str) -> Result<u8, ParseError> {
    let as_u16 = from_hex(value)?;
    if as_u16 > 127 {
        // Not a 7-bit address.
        eprintln!("{value} is not a valid seven-bit address");
        return Err(ParseError::InvalidSevenBitAddress);
    }
    Ok(as_u16 as u8)
}
