//! Status read from the MCP2221.
// TODO: Improve the module-level documentation as it's publicly exposed.

use bit_field::BitField;

use crate::i2c::I2cStatus;

/// Current status of the MCP2221.
///
/// Bytes in documentation are numbered from 0 through 63 and correspond
/// to table 3-1 in section 3.1.1 (Status/Set Parameters) of the datasheet.
#[derive(Debug)]
pub struct Status {
    /// I2C engine status
    pub i2c: I2cStatus,
    /// Edge-detection interrupt state.
    ///
    /// True if an interrupt has been detected. Use [`ChangeSramSettings`] to clear
    /// the interrupt flag or alter the interrupt detection conditions.
    ///
    /// # Datasheet
    ///
    /// See byte 24 in table 3-2 for the source of this field. It's listed as being
    /// either 0 or 1, so we've made the assumption that 1 means an interrupt has
    /// been detected.
    ///
    /// See section 1.0 and 1.6.2.4 for general information about interrupt detection.
    ///
    /// [`ChangeSramSettings`]: crate::settings::ChangeSramSettings
    pub interrupt_detected: bool,
    /// MCP2221 hardware revision.
    pub hardware_revision: Revision,
    /// MCP2221 firmware revision.
    pub firmware_revision: Revision,
    /// Readings from the 3 channels of the 10-bit ADC.
    pub adc_values: RawAdcValues,
}

impl Status {
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        Self {
            i2c: I2cStatus {
                communication_state: buf[8].into(),
                transfer_requested_length: u16::from_le_bytes([buf[9], buf[10]]),
                transfer_completed_length: u16::from_le_bytes([buf[11], buf[12]]),
                internal_data_buffer_counter: buf[13],
                bus_speed: buf[14].into(),
                timeout_value: buf[15],
                target_address: u16::from_le_bytes([buf[16], buf[17]]),
                // Note that this is being inverted: 0 means ACK received.
                target_acknowledged_address: !buf[20].get_bit(6),
                scl_line_high: buf[22] == 0x01,
                sda_line_high: buf[23] == 0x01,
                read_pending_value: buf[25],
            },
            interrupt_detected: buf[24] == 0x01,
            hardware_revision: Revision::new(buf[46] as char, buf[47] as char),
            firmware_revision: Revision::new(buf[48] as char, buf[49] as char),
            adc_values: RawAdcValues::new(
                u16::from_le_bytes([buf[50], buf[51]]),
                u16::from_le_bytes([buf[52], buf[53]]),
                u16::from_le_bytes([buf[54], buf[55]]),
            ),
        }
    }
}

/// Two-part revision number.
///
/// Used for the hardware and firmware revisions in the MCP2221 Status report.
#[derive(Debug)]
pub struct Revision {
    /// Major component of the revision number. (x.0)
    pub major: char,
    /// Minor component of the revision number. (0.x)
    pub minor: char,
}

impl Revision {
    fn new(major: char, minor: char) -> Self {
        Self { major, minor }
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Raw three-channel reading from the ADC.
///
/// Prefer [`MCP2221::analog_read`] and [`AdcReading`] instead of interacting with
/// this directly. This struct is made available only for inspection of the internal
/// MCP2221 state.
///
/// [`MCP2221::analog_read`]: crate::MCP2221::analog_read
/// [`AdcReading`]: crate::analog::AdcReading
///
/// If the pin for a channel is not configured as an analog input, the value read
/// for that channel is formally undefined.
///
/// # Datasheet
///
/// See bytes `50..=55` in table 3-2 for the source of these values, table 1-1 and
/// table 1-5 for the mapping of ADC channels to GP pins, and section 1.8 for general
/// information about the ADC.
#[derive(Debug)]
pub struct RawAdcValues {
    /// ADC reading of channel 1 (GP1).
    pub ch1: u16,
    /// ADC reading of channel 2 (GP2).
    pub ch2: u16,
    /// ADC reading of channel 3 (GP3).
    pub ch3: u16,
}

impl RawAdcValues {
    fn new(ch1: u16, ch2: u16, ch3: u16) -> Self {
        Self { ch1, ch2, ch3 }
    }
}
