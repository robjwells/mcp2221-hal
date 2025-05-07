#[derive(Debug, Clone)]
pub struct DeviceString(String);

impl TryFrom<String> for DeviceString {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.encode_utf16().count() <= 30 {
            Ok(Self(value))
        } else {
            Err("String must be 60 bytes or fewer when UTF-16-encoded.")
        }
    }
}

impl DeviceString {
    pub(crate) fn from_device_report(buf: &[u8; 64]) -> Self {
        assert_eq!(buf[3], 0x03, "String response sanity check.");

        let n_bytes = buf[2] as usize - 2;
        // Sanity-check the string length.
        assert!(n_bytes <= 60, "String longer than specified.");
        assert_eq!(n_bytes % 2, 0, "Odd number of utf-16 bytes received.");

        // (buf[2] - 2) UTF-16 characters laid out in little-endian order
        // from buf[4] onwards. These strings are at most 30 characters
        // (60 bytes) long. See table 3-7 in the datasheet.
        let n_utf16_chars = n_bytes / 2;
        let mut str_utf16 = Vec::with_capacity(n_utf16_chars);
        for char_number in 0..n_utf16_chars {
            let low_idx = 4 + 2 * char_number;
            let high_idx = 4 + 2 * char_number + 1;
            let utf16 = u16::from_le_bytes([buf[low_idx], buf[high_idx]]);
            str_utf16.push(utf16);
        }

        // TODO: Really this should be an error, not a panic.
        let s = String::from_utf16(str_utf16.as_slice())
            .expect("Invalid Unicode string received from device.");
        Self(s)
    }
}

impl crate::commands::WriteCommandData for DeviceString {
    /// Write the utf-16 string to the buffer to be written to the MCP2221.
    ///
    /// See table 3-14 in the datasheet. This function writes the appropriate
    /// count to byte 2, and the 0x03 constant to byte 3.
    fn apply_to_buffer(&self, buf: &mut [u8; 64]) {
        let mut byte_count = 0;
        let utf16_pairs = self.0.encode_utf16().map(u16::to_le_bytes);
        for (unicode_char_number, [low, high]) in utf16_pairs.enumerate() {
            let pos = 4 + (2 * unicode_char_number);
            buf[pos] = low;
            buf[pos + 1] = high;
            byte_count += 2;
        }
        buf[2] = byte_count + 2;
        buf[3] = 0x03; // Required constant. Perhaps marks the data as an LE UTF16 string.
    }
}

impl std::fmt::Display for DeviceString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
