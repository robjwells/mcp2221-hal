pub(crate) fn from_hex(value: &str) -> Result<u16, std::num::ParseIntError> {
    let s = if value.to_ascii_lowercase().starts_with("0x") {
        &value[2..]
    } else {
        value
    };
    u16::from_str_radix(s, 16)
}
