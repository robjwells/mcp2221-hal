//! Utility types for various settings.

use bit_field::BitField;

use crate::Error;

/// String limited to 30 UTF-16 code units.
///
/// The strings stored in the MCP2221 flash memory (used during USB enumeration)
/// are limited to at most 60 bytes of UTF-16-encoded text.
///
/// Create a `DeviceString` by calling [`str::parse`] on a string slice, or
/// [`DeviceString::try_from`] with an owned `String`.
///
/// ```rust
/// # use mcp2221_hal::settings::DeviceString;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manufacturer: DeviceString = "Acme Widgets Company (UK) Ltd".parse()?;
///
/// let product = String::from("Internet of Widgets Hub v3.0");
/// let product: DeviceString = product.try_into()?;
/// # Ok(())
/// # }
/// ```
///
/// Note that some characters require two UTF-16 code units to express (4 bytes).
///
/// ```rust
/// # use mcp2221_hal::settings::DeviceString;
/// let serial = "4 bytes each: 🫐🫑🫒🫓🫔🫕🫖🫗🫘🫙";
/// let result: Result<DeviceString, _> = serial.parse();
/// assert!(result.is_err(), "More than 60 bytes when UTF-16 encoded.");
/// ```
///
/// ## Datasheet
///
/// See table 3-7 and table 3-14 for details of how the device strings are read from
/// and written to the MCP2221, including the length limitation. (Those tables are
/// for the manufacturer string, but the following tables are identical except for
/// the subcommand code.)
#[derive(Debug, Clone)]
pub struct DeviceString(String);

impl TryFrom<String> for DeviceString {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Check the number of u16s (two bytes) is within the limit.
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
///
/// ## Datasheet
///
/// See register 1-2 (ChipSettings1) in the datasheet for the details of the duty cycle
/// options and bit pattern.
#[derive(Debug, Default, Clone, Copy)]
pub enum ClockDutyCycle {
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
impl From<u8> for ClockDutyCycle {
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
impl From<ClockDutyCycle> for u8 {
    fn from(value: ClockDutyCycle) -> u8 {
        match value {
            ClockDutyCycle::P75 => 0b11,
            ClockDutyCycle::P50 => 0b10,
            ClockDutyCycle::P25 => 0b01,
            ClockDutyCycle::P0 => 0b00,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy)]
/// Clock output frequency.
///
/// The frequency options and their 3-bit representation suggests this is a shift value
/// for an internal 48 MHz clock, but note that there is no 48 MHz clock output option
/// and the bit pattern that for that (`0b000`) is marked "reserved".
///
/// ## Datasheet
///
/// See register 1-2 (ChipSettings1) in the datasheet for the details of the frequency
/// options and bit pattern.
pub enum ClockFrequency {
    /// 375 kHz clock output.
    _375_kHz,
    /// 750 kHz clock output.
    _750_kHz,
    /// 1.5 MHz clock output.
    _1_5_MHz,
    /// 3 MHz clock output.
    _3_MHz,
    /// 6 MHz clock output.
    _6_MHz,
    /// 12 MHz clock output (factory default).
    #[default]
    _12_MHz,
    /// 24 MHz clock output.
    _24_MHz,
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
            0b111 => Self::_375_kHz,
            0b110 => Self::_750_kHz,
            0b101 => Self::_1_5_MHz,
            0b100 => Self::_3_MHz,
            0b011 => Self::_6_MHz,
            0b010 => Self::_12_MHz,
            0b001 => Self::_24_MHz,
            _ => unreachable!("Precondition asserts cover 0 and > 7."),
        }
    }
}

#[doc(hidden)]
impl From<ClockFrequency> for u8 {
    fn from(value: ClockFrequency) -> Self {
        match value {
            ClockFrequency::_375_kHz => 0b111,
            ClockFrequency::_750_kHz => 0b110,
            ClockFrequency::_1_5_MHz => 0b101,
            ClockFrequency::_3_MHz => 0b100,
            ClockFrequency::_6_MHz => 0b011,
            ClockFrequency::_12_MHz => 0b010,
            ClockFrequency::_24_MHz => 0b001,
        }
    }
}

/// Clock output duty cycle and frequency.
///
/// If GP1 is configured for clock output (see [`Gp1Mode::ClockOutput`]), this
/// setting determines the characteristics of the clock signal.
///
/// [`Gp1Mode::ClockOutput`]: crate::settings::Gp1Mode::ClockOutput
///
/// ## Datasheet
///
/// See register 1-2 for details. In the USB command section the datasheet is worded as
/// if this is just a 5-bit divider, but really it is a 2-bit duty cycle selection, and
/// a 3-bit frequency selection.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClockOutputSetting(
    /// The duty cycle (period high) of the clock output signal.
    pub ClockDutyCycle,
    /// The frequency of the clock output signal.
    pub ClockFrequency
);

#[doc(hidden)]
impl From<u8> for ClockOutputSetting {
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
            ClockDutyCycle::from(value.get_bits(3..=4)),
            ClockFrequency::from(value.get_bits(0..=2)),
        )
    }
}

#[doc(hidden)]
impl From<ClockOutputSetting> for u8 {
    fn from(value: ClockOutputSetting) -> Self {
        let ClockOutputSetting(duty_cycle, frequency) = value;
        let mut byte = 0u8;
        // Set duty cycle.
        byte.set_bits(3..=4, duty_cycle.into());
        // Set frequency.
        byte.set_bits(0..=2, frequency.into());
        byte
    }
}
