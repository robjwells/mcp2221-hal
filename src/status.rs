/// Current status of the device.
///
/// Bytes in documentation are numbered from 0 through 63 and correspond
/// to table 3-1 in section 3.1.1 (STATUS/SET PARAMETERS) of the datasheet.
#[derive(Debug)]
pub struct Status {
    /// The requested I2C transfer length.
    ///
    /// Bytes 9 & 10.
    pub i2c_transfer_requested_length: u16,
    /// The already transferred (through I2C) number of bytes.
    ///
    /// Bytes 11 & 12.
    pub i2c_transfer_completed_length: u16,
    /// Byte 13.
    pub i2c_internal_data_buffer_counter: u8,
    /// Byte 14.
    pub i2c_communication_speed_divider: u8,
    /// Byte 15.
    pub i2c_timeout_value: u8,
    /// Bytes 16 & 17.
    pub i2c_address_being_used: u16,
    /// Byte 22.
    pub i2c_scl_line_high: bool,
    /// Byte 23.
    pub i2c_sda_line_high: bool,
    /// Byte 24.
    pub interrupt_edge_detector_state: u8,
    /// I2C Read pending value.
    ///
    /// Byte 25. This field is used by the USB host to know if the MCP2221A
    /// still has to read from a slave device. Value 0, 1 or 2.
    pub i2c_read_pending_value: u8,
    /// MCP2221A hardware revision (major, minor).
    ///
    /// Bytes 46 & 47.
    pub hardware_revision: (char, char),
    /// MCP2221A firmware revision (major, minor)
    ///
    /// Bytes 48 & 49.
    pub firmware_revision: (char, char),
    /// ADC Data (16-bit) values.
    ///
    /// 3x 16-bit ADC channel values (CH0, CH1, CH2).
    ///
    /// Bytes 50..=55.
    pub adc_values: (u16, u16, u16),
}

impl Status {
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        Self {
            i2c_transfer_requested_length: u16::from_le_bytes([buf[9], buf[10]]),
            i2c_transfer_completed_length: u16::from_le_bytes([buf[11], buf[12]]),
            i2c_internal_data_buffer_counter: buf[13],
            i2c_communication_speed_divider: buf[14],
            i2c_timeout_value: buf[15],
            i2c_address_being_used: u16::from_le_bytes([buf[16], buf[17]]),
            i2c_scl_line_high: buf[22] == 0x01,
            i2c_sda_line_high: buf[23] == 0x01,
            interrupt_edge_detector_state: buf[24],
            i2c_read_pending_value: buf[25],
            hardware_revision: (buf[46] as char, buf[47] as char),
            firmware_revision: (buf[48] as char, buf[49] as char),
            adc_values: (
                u16::from_le_bytes([buf[50], buf[51]]),
                u16::from_le_bytes([buf[52], buf[53]]),
                u16::from_le_bytes([buf[54], buf[55]]),
            ),
        }
    }
}

