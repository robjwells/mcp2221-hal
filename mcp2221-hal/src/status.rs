//! Status read from the MCP2221.
// TODO: Improve the module-level documentation as it's publicly exposed.

use bit_field::BitField;

/// I2C engine status
///
/// This struct bundles together the I2C-related fields of the Status command response.
///
/// # Datasheet
///
/// See the return value of the Status/Set Parameters HID command in table 3-2 of
/// section 3.1.1 for the definition of the fields.
///
/// Note that byte 8 of the Status response, "I2C communication state", is not included
/// because it's unclear how it should be interpreted.
#[derive(Debug)]
pub struct I2cStatus {
    /// I2C engine communication state.
    ///
    /// Records whether the I2C engine is idle or a timeout has occurred.
    pub communication_state: I2cCommunicationState,
    /// The current requested I2C transfer length.
    pub transfer_requested_length: u16,
    /// Number of bytes already transferred.
    // TODO: Is this _total_ transferred or just the _current_ transfer?
    //       Should be easy to test once I have I2C implemented.
    pub transfer_completed_length: u16,
    /// Internal I2C data buffer counter.
    ///
    /// This field is not explained in the datasheet.
    pub internal_data_buffer_counter: u8,
    /// I2C bus speed divider.
    // TODO: This should really be the I2cSpeed struct, but I need to allow for
    // non-standard speeds to be able to reliably parse this divider (which is
    // (more or less) actually a divider, unlike with the clock output divider).
    pub communication_speed_divider: u8,
    /// I2C engine timeout value.
    ///
    /// It is not explained in the datasheet how this value should be interpreted.
    /// It does not seem to be a length of time after which timeout occurs, perhaps
    /// it is some internal enum value.
    pub timeout_value: u8,
    /// I2C address being used.
    ///
    /// It is presumed that this is the address of the I2C target currently being
    /// communicated with, but it is not explained in the datasheet.
    ///
    /// Additionally, it appears from the Java driver that the target address used
    /// to be set via the status command, so perhaps this is a remnant of that?
    pub address_being_used: u16,
    /// I2C target acknowledged its address.
    ///
    /// This is not further explained in the datasheet, but is described in the Java
    /// driver as being the ACK to the I2C slave address.
    pub ack_received: bool,
    /// I2C clock line is high.
    ///
    /// I2C is an idle-high bus, so perhaps this could be used to detect a device on
    /// the bus holding the SCL line low? It is not explained in the datasheet.
    pub scl_line_high: bool,
    /// I2C data line is high.
    ///
    /// I2C is an idle-high bus, so perhaps this could be used to detect a device on
    /// the bus holding the SDA line low? It is not explained in the datasheet.
    pub sda_line_high: bool,
    /// I2C Read pending value.
    ///
    /// The datasheet describes this as being "used by the USB host to know if the
    /// MCP2221 still has to read from a slave device", with possible values of 0, 1
    /// or 2. The meaning of those values is not explained in the datasheet.
    ///
    /// See byte 25 in table 3-2.
    pub read_pending_value: u8,
}

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
    /// [`ChangeSramSettings`]: crate::ChangeSramSettings
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
                communication_speed_divider: buf[14],
                timeout_value: buf[15],
                address_being_used: u16::from_le_bytes([buf[16], buf[17]]),
                // Note that this is being inverted: 0 means ACK received.
                ack_received: !buf[20].get_bit(6),
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

/// State of the I2C engine.
///
/// Most of the cases are guesswork from constant names in the Microchip C and Android
/// Java drivers, and their meaning is not documented. The datasheet only says that a
/// state other than 0 is a timeout.
#[derive(Debug)]
pub enum I2cCommunicationState {
    /// Engine is idle.
    Idle,
    /// I2CM_SM_START_TOUT.
    StartTimeout,
    /// I2CM_SM_REPSTART_TOUT.
    RepeatedStartTimeout,
    /// I2CM_SM_WRADDL_WAITSEND.
    WriteAddressWaitSend,
    /// Target address timeout.
    ///
    /// - RESP_I2C_WRADDRL_TOUT in C.
    /// - I2CM_SM_WRADDRL_TOUT in Java.
    AddressTimeout,
    /// Target did not acknowledge its address.
    ///
    /// - RESP_ADDR_NACK in C.
    /// - I2CM_SM_WRADDL_NACK_STOP in Java.
    AddressNack,
    /// I2CM_SM_WRITEDATA_TOUT.
    WriteDataTimeout,
    /// Engine has finished sending data from an I2C Write Data No Stop command.
    ///
    /// I2CM_SM_WRITEDATA_END_NOSTOP.
    WriteDataEndNoStop,
    /// I2CM_SM_READDATA_TOUT.
    ReadDataTimeout,
    /// I2CM_SM_STOP_TOUT.
    StopTimeout,
    /// I2C engine is in some other state.
    Other(u8),
}

impl I2cCommunicationState {
    /// I2C engine is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

impl From<u8> for I2cCommunicationState {
    fn from(value: u8) -> Self {
        // Hex values taken from the Java driver.
        match value {
            0x00 => Self::Idle,
            0x12 => Self::StartTimeout,
            0x17 => Self::RepeatedStartTimeout,
            0x21 => Self::WriteAddressWaitSend,
            0x23 => Self::AddressTimeout,
            0x25 => Self::AddressNack,
            0x44 => Self::WriteDataTimeout,
            0x45 => Self::WriteDataEndNoStop,
            0x52 => Self::ReadDataTimeout,
            0x62 => Self::StopTimeout,
            n => Self::Other(n),
        }
    }
}
