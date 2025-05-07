#[derive(Debug)]
pub enum CancelI2cTransferResponse {
    MarkedForCancellation,
    NoTransfer,
}

#[allow(non_camel_case_types)]
pub enum I2cSpeed {
    /// I2c bus speed of 400kbps ("Fast-mode")
    Fast_400kbps,
    /// I2c bus speed of 100kbps ("Standard-mode")
    Standard_100kbps,
}

impl I2cSpeed {
    /// Convert the speed mode into a clock divider suitable for the
    /// STATUS/SET PARAMETERS command.
    pub(crate) fn to_clock_divider(&self) -> u8 {
        // 12 MHz internal clock.
        const MCP_CLOCK: u32 = 12_000_000;

        // The `-2` part is from Note 1 in Table 3-1 in the datasheet.
        const STANDARD_DIVIDER: u8 = (MCP_CLOCK / 100_000 - 2) as u8;
        const FAST_DIVIDER: u8 = (MCP_CLOCK / 400_000 - 2) as u8;

        match self {
            I2cSpeed::Fast_400kbps => FAST_DIVIDER,
            I2cSpeed::Standard_100kbps => STANDARD_DIVIDER,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// GPIO pin level setting.
pub enum LogicLevel {
    High,
    Low,
}

impl From<bool> for LogicLevel {
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

impl From<LogicLevel> for bool {
    fn from(value: LogicLevel) -> Self {
        match value {
            LogicLevel::High => true,
            LogicLevel::Low => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// GPIO pin direction.
pub enum GpioDirection {
    Input,
    Output,
}

impl From<bool> for GpioDirection {
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}

impl From<GpioDirection> for bool {
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => true,
            GpioDirection::Output => false,
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
/// Setting of the internal voltage reference (VRM)
pub enum VrmVoltage {
    /// 4.096V
    ///
    /// Only available if VDD is above this voltage.
    V4_096,
    /// 2.048V
    V2_048,
    /// 1.024V
    V1_024,
    /// Reference voltage is off.
    ///
    /// This is useful for the case in which the DAC uses another reference other
    /// than Vrm DAC; eg Vdd.
    Off,
}

impl From<u8> for VrmVoltage {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Off,
            0b01 => Self::V1_024,
            0b10 => Self::V2_048,
            0b11 => Self::V4_096,
            _ => unreachable!(),
        }
    }
}

impl From<VrmVoltage> for u8 {
    fn from(value: VrmVoltage) -> Self {
        match value {
            VrmVoltage::V4_096 => 0b11,
            VrmVoltage::V2_048 => 0b10,
            VrmVoltage::V1_024 => 0b01,
            VrmVoltage::Off => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VoltageReference {
    Vrm,
    Vdd,
}

impl From<bool> for VoltageReference {
    fn from(value: bool) -> Self {
        if value { Self::Vrm } else { Self::Vdd }
    }
}

impl From<VoltageReference> for bool {
    fn from(value: VoltageReference) -> Self {
        // Note that table 3-12 byte 5 lists 1 = VDD, 0 = VRM. This is the opposite
        // to all other uses in the datasheet and appears to be an error, as is
        // table 3-5 byte 7 (read flash data ADC reference).
        match value {
            VoltageReference::Vrm => true,
            VoltageReference::Vdd => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChipConfigurationSecurity {
    PermanentlyLocked,
    PasswordProtected,
    Unsecured,
}

impl From<u8> for ChipConfigurationSecurity {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Unsecured,
            0b01 => Self::PasswordProtected,
            0b10 | 0b11 => Self::PermanentlyLocked,
            _ => unreachable!(),
        }
    }
}

impl From<ChipConfigurationSecurity> for u8 {
    fn from(value: ChipConfigurationSecurity) -> Self {
        match value {
            ChipConfigurationSecurity::PermanentlyLocked => 0b10,
            ChipConfigurationSecurity::PasswordProtected => 0b01,
            ChipConfigurationSecurity::Unsecured => 0b00,
        }
    }
}
