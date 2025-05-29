//! MCP2221 status output.
//!
//! [`Status`] is returned by [`MCP2221::status`] and contains information about the
//! state of the I2C engine, interrupt detection, ADC readings, and hardware and
//! firmware revision numbers.
//!
//! Generally this is only used to inspect the internal state of the MCP2221, and
//! methods on the [`MCP2221`] driver will let you interact with that state in a
//! more convenient way.
//!
//! [`MCP2221::status`]: crate::MCP2221::status
//! [`MCP2221`]: crate::MCP2221

use bit_field::BitField;

use crate::i2c::I2cStatus;

/// Current status of the MCP2221.
///
/// The fields of this struct represent the current internal state of the MCP2221 (at
/// least as much is exposed).
///
/// ## Datasheet
///
/// See section 3.1.1 for the underlying Status/Set Parameters HID command. The field
/// descriptions of the response structure are the source for the documentation of
/// the fields here.
#[derive(Debug, Clone, Copy)]
pub struct Status {
    /// I2C engine status
    pub i2c: I2cStatus,
    /// Edge-detection interrupt state.
    ///
    /// True if an edge has been detected on GP1. Requires GP1 to be in interrupt
    /// detection mode, and for the appropriate edge-detection settings to be
    /// enabled (positive, negative, or both).
    ///
    /// Prefer to use [`MCP2221::interrupt_detected`] to read the flag. The flag can
    /// be cleared with [`MCP2221::interrupt_clear`].
    ///
    /// [`MCP2221::interrupt_detected`]: crate::MCP2221::interrupt_detected
    /// [`MCP2221::interrupt_clear`]: crate::MCP2221::interrupt_clear
    ///
    /// ## Datasheet
    ///
    /// In the Status/Set Parameters response, byte 24 is listed as being 1 or 0
    /// depending on the interrupt state. We've made the assumption that 1 means
    /// that an interrupt has been detected.
    // TODO: Actually test the interrupts.
    pub interrupt_detected: bool,
    /// Hardware revision.
    pub hardware_revision: Revision,
    /// Firmware revision.
    pub firmware_revision: Revision,
    /// Readings from the three channels of the 10-bit ADC.
    ///
    /// There will always be three readings, no matter the configuration of the
    /// corresponding GP pins. However, these readings are unspecified when the
    /// pins are not configured as analog inputs.
    pub adc_values: RawAdcValues,
}

impl Status {
    /// Parse the Status/Set Parameters response buffer.
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
            hardware_revision: Revision {
                major: buf[46] as char,
                minor: buf[47] as char,
            },
            firmware_revision: Revision {
                major: buf[48] as char,
                minor: buf[49] as char,
            },
            adc_values: RawAdcValues {
                ch1: u16::from_le_bytes([buf[50], buf[51]]),
                ch2: u16::from_le_bytes([buf[52], buf[53]]),
                ch3: u16::from_le_bytes([buf[54], buf[55]]),
            },
        }
    }
}

/// Two-part revision number.
///
/// Used for the hardware and firmware revisions in the MCP2221 status report.
#[derive(Clone, Copy)]
pub struct Revision {
    /// Major component of the revision number. (x.0)
    pub major: char,
    /// Minor component of the revision number. (0.x)
    pub minor: char,
}

impl std::fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Revision({}.{})", self.major, self.minor)
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
/// If the pin for a channel is not configured as an analog input, it is undefined
/// what the reading will be.
///
/// ## Datasheet
///
/// See bytes `50..=55` in table 3-2 for the source of these values, table 1-1 and
/// table 1-5 for the mapping of ADC channels to GP pins, and section 1.8 for general
/// information about the ADC.
#[derive(Debug, Clone, Copy)]
pub struct RawAdcValues {
    /// ADC reading of channel 1 (GP1).
    pub ch1: u16,
    /// ADC reading of channel 2 (GP2).
    pub ch2: u16,
    /// ADC reading of channel 3 (GP3).
    pub ch3: u16,
}
