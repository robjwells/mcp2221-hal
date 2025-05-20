//! Utility types for various settings.

use bit_field::BitField;

use crate::Error;

/// String with at most 30 UTF-16 code points.
///
/// The strings stored in the MCP2221 flash memory (used during USB enumeration)
/// are limited to at most 60 bytes of UTF-16-encoded text.
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

impl std::str::FromStr for DeviceString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_owned())
    }
}

impl DeviceString {
    pub(crate) fn try_from_buffer(buf: &[u8; 64]) -> Result<Self, Error> {
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

        String::from_utf16(str_utf16.as_slice())
            .map(Self)
            .map_err(Error::InvalidStringFromDevice)
    }

    /// Write the utf-16 string to the buffer to be written to the MCP2221.
    ///
    /// See table 3-14 in the datasheet. This function writes the appropriate
    /// count to byte 2, and the 0x03 constant to byte 3.
    pub(crate) fn apply_to_flash_buffer(&self, buf: &mut [u8; 64]) {
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

/// Clock output duty cycle.
///
/// Each case is the percentage of one clock period that is a high logic level.
#[derive(Debug, Default, Clone, Copy)]
pub enum DutyCycle {
    /// 75% duty cycle.
    P75,
    /// 50% duty cycle (factory default).
    #[default]
    P50,
    /// 25% duty cycle.
    P25,
    /// 0% duty cycle.
    P0,
}

#[doc(hidden)]
impl From<u8> for DutyCycle {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Invalid bit pattern for duty cycle");
        match value {
            0b11 => Self::P75,
            0b10 => Self::P50,
            0b01 => Self::P25,
            0b00 => Self::P0,
            _ => unreachable!("Precondition assert covers > 3."),
        }
    }
}

#[doc(hidden)]
impl From<DutyCycle> for u8 {
    fn from(value: DutyCycle) -> u8 {
        match value {
            DutyCycle::P75 => 0b11,
            DutyCycle::P50 => 0b10,
            DutyCycle::P25 => 0b01,
            DutyCycle::P0 => 0b00,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy)]
/// Clock output frequency.
// I am not wild about the names!
pub enum ClockFrequency {
    /// 375 kHz clock output.
    kHz375,
    /// 750 kHz clock output.
    kHz750,
    /// 1.5 MHz clock output.
    MHz1_5,
    /// 3 MHz clock output.
    MHz3,
    /// 6 MHz clock output.
    MHz6,
    /// 12 MHz clock output (factory default).
    #[default]
    MHz12,
    /// 24 MHz clock output.
    MHz24,
}

#[doc(hidden)]
impl From<u8> for ClockFrequency {
    /// Create a `ClockFrequency` from the 3 low bits of the raw "divider".
    ///
    /// # Panics
    ///
    /// Frequency pattern `0b000` is marked "Reserved" in the datasheet and attempting
    /// to create a `ClockFrequency` with this value will fail an assertion.
    ///
    /// Any value greater than `0b111` will fail an assertion.
    fn from(value: u8) -> Self {
        assert!(value <= 0b111, "Invalid bit pattern for clock speed.");
        assert_ne!(value, 0, "Use of Reserved clock speed bit pattern.");
        match value {
            0b111 => Self::kHz375,
            0b110 => Self::kHz750,
            0b101 => Self::MHz1_5,
            0b100 => Self::MHz3,
            0b011 => Self::MHz6,
            0b010 => Self::MHz12,
            0b001 => Self::MHz24,
            _ => unreachable!("Precondition asserts cover 0 and > 7."),
        }
    }
}

#[doc(hidden)]
impl From<ClockFrequency> for u8 {
    fn from(value: ClockFrequency) -> Self {
        match value {
            ClockFrequency::kHz375 => 0b111,
            ClockFrequency::kHz750 => 0b110,
            ClockFrequency::MHz1_5 => 0b101,
            ClockFrequency::MHz3 => 0b100,
            ClockFrequency::MHz6 => 0b011,
            ClockFrequency::MHz12 => 0b010,
            ClockFrequency::MHz24 => 0b001,
        }
    }
}

/// Clock output duty cycle and frequency.
///
/// See datasheet register 1-2 for details. In the USB command section the datasheet
/// is worded as if this is just a 5-bit divider, but really it is a 2-bit duty cycle
/// selection, and a 3-bit frequency selection.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClockSetting(pub DutyCycle, pub ClockFrequency);

#[doc(hidden)]
impl From<u8> for ClockSetting {
    /// Create a `ClockSetting` from the 5-bit "divider" read from the MCP2221.
    ///
    /// # Panics
    ///
    /// Frequency pattern `0b000` is marked "Reserved" in the datasheet and attempting
    /// to create a [`ClockFrequency`] with this value will fail an assertion.
    fn from(value: u8) -> Self {
        assert!(
            value <= 0b11111,
            "Raw clock 'divider' must be in the low 5 bits."
        );
        Self(
            DutyCycle::from(value.get_bits(3..=4)),
            ClockFrequency::from(value.get_bits(0..=2)),
        )
    }
}

#[doc(hidden)]
impl From<ClockSetting> for u8 {
    fn from(value: ClockSetting) -> Self {
        let ClockSetting(duty_cycle, frequency) = value;
        let mut byte = 0u8;
        // Set duty cycle.
        byte.set_bits(3..=4, duty_cycle.into());
        // Set frequency.
        byte.set_bits(0..=2, frequency.into());
        byte
    }
}
