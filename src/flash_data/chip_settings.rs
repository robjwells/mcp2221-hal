use crate::analog::{VoltageReference, VrmVoltage};
use crate::security::ChipConfigurationSecurity;

use bit_field::BitField;

#[derive(Debug)]
/// Chip settings stored in the MCP2221's flash memory.
///
/// Byte and bit addresses in this documentation refer to their position when _reading_
/// from the MCP2221. For their position in the write buffer, subtract two from
/// the byte address.
pub struct ChipSettings {
    /// Whether a serial number descriptor will be presented during the
    /// USB enumeration of the CDC interface.
    ///
    /// Byte 4 bit 7.
    pub cdc_serial_number_enumeration_enabled: bool,
    /// Chip configuration security option.
    ///
    /// Byte 4 bits 1 and 0.
    pub chip_configuration_security: ChipConfigurationSecurity,
    /// Clock Output settings.
    ///
    /// If GP1 is set to clock output, this value determines its duty cycle
    /// and frequency. See register 1-2 in the datasheet for the meaning of
    /// this value.
    ///
    /// Note that the datasheet's description of this setting in the USB HID
    /// command section appears to be incorrect. The internal clock is 12 MHz
    /// (not 48) and this value is not just a divider but also a duty-cycle
    /// setting.
    ///
    /// Datasheet description for reference (table 3-5):
    ///
    /// > If the GP pin (exposing the clock output) is enabled for clock
    /// > output operation, the divider value will be used on the 48 MHz USB
    /// > internal clock and its divided output will be sent to this pin.
    ///
    /// Byte 5 bits 4..=0.
    pub clock_output_divider: u8,
    /// DAC reference voltage (Vrm setting)
    ///
    /// Note that the MCP2221 appears to ignore this setting on boot.
    ///
    /// Byte 6 bits 7 & 6.
    pub dac_reference_voltage: VrmVoltage,
    /// DAC reference source (Vrm or Vdd)
    ///
    /// Note that setting this to Vrm will cause the MCP2221, on boot, to behave as if
    /// the DAC was configured for Vrm with its reference level set to "Off", regardless
    /// of what you have set the DAC Vrm reference voltage to (eg 1.024V or 2.048V).
    /// This persists until you reconfigure the DAC settings in SRAM.
    ///
    /// If set to Vdd, the DAC will behave as expected upon boot.
    ///
    /// Byte 6 bit 5.
    pub dac_reference_source: VoltageReference,
    /// Power-up DAC value.
    ///
    /// Byte 6 bits 4..=0. Value in range 0..=31.
    pub dac_power_up_value: u8,
    /// Interrupt detection for negative edge.
    ///
    /// Byte 7 bit 6.
    pub interrupt_on_negative_edge: bool,
    /// Interrupt detection for positive edge.
    ///
    /// Byte 7 bit 5.
    pub interrupt_on_positive_edge: bool,
    /// ADC reference voltage (Vrm setting)
    ///
    /// Byte 7 bits 4 & 3.
    pub adc_reference_voltage: VrmVoltage,
    /// ADC reference source (Vrm or Vdd)
    ///
    /// Note the datasheet "effect" column says this is the DAC reference,
    /// but it appears to be a typo. The DAC and ADC have their own
    /// voltage references (see section 1.8.1.1 of the datasheet).
    ///
    /// Byte 7 bit 2.
    pub adc_reference_source: VoltageReference,
    /// USB Vendor ID (VID)
    ///
    /// Byte 8 and 9.
    pub usb_vendor_id: u16,
    /// USB Product ID (PID)
    ///
    /// Byte 10 and 11.
    pub usb_product_id: u16,
    /// USB power attributes.
    ///
    /// This value will be used by the MCP2221's USB Configuration
    /// Descriptor (power attributes value) during the USB enumeration.
    ///
    /// Please consult the USB 2.0 specification on the correct values
    /// for power and attributes.
    ///
    /// Byte 12.
    pub usb_power_attributes: u8,
    /// USB requested number of mA.
    ///
    /// The requested mA value during the USB enumeration. Please consult the USB 2.0
    /// specification on the correct values for power and attributes.
    ///
    /// Note the datasheet says the actual value is the byte value multiplied by 2.
    /// The value in this struct has already been multiplied by 2 for convenience.
    ///
    /// As the halved value is stored as a single byte by the MCP2221, the maximum
    /// possible value is 510 mA (stored as `255u8` on the chip);
    ///
    /// Byte 13.
    pub usb_requested_number_of_ma: u16,
}

impl ChipSettings {
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            chip_configuration_security: buf[4].get_bits(0..=1).into(),
            clock_output_divider: buf[5].get_bits(0..=4),
            dac_reference_voltage: buf[6].get_bits(6..=7).into(),
            dac_reference_source: buf[6].get_bit(5).into(),
            dac_power_up_value: buf[6].get_bits(0..=4),
            interrupt_on_negative_edge: buf[7].get_bit(6),
            interrupt_on_positive_edge: buf[7].get_bit(5),
            adc_reference_voltage: buf[7].get_bits(3..=4).into(),
            adc_reference_source: buf[7].get_bit(2).into(),
            usb_vendor_id: u16::from_le_bytes([buf[8], buf[9]]),
            usb_product_id: u16::from_le_bytes([buf[10], buf[11]]),
            usb_power_attributes: buf[12],
            usb_requested_number_of_ma: buf[13] as u16 * 2,
        }
    }
}

impl crate::commands::WriteCommandData for ChipSettings {
    fn apply_to_buffer(&self, buf: &mut [u8; 64]) {
        // Note the bytes positions when writing are -2 from the position when reading.
        buf[2].set_bit(7, self.cdc_serial_number_enumeration_enabled);
        // TODO: support security settings.
        buf[2].set_bits(0..=1, ChipConfigurationSecurity::Unsecured.into());

        // Byte 3 (write) / byte 5 (read)
        buf[3].set_bits(0..=4, self.clock_output_divider);

        // Byte 4 (write) / byte 6 (read) -- DAC settings
        buf[4].set_bits(6..=7, self.dac_reference_voltage.into());
        buf[4].set_bit(5, self.dac_reference_source.into());
        buf[4].set_bits(0..=4, self.dac_power_up_value);

        // Byte 5 (write) / byte 6 (read) -- Interrupts and ADC
        buf[5].set_bit(6, self.interrupt_on_negative_edge);
        buf[5].set_bit(5, self.interrupt_on_positive_edge);
        buf[5].set_bits(3..=4, self.adc_reference_voltage.into());
        buf[5].set_bit(2, self.adc_reference_source.into());

        // Bytes 6 & 7 -- USB Vendor ID (VID)
        let vid_bytes = self.usb_vendor_id.to_le_bytes();
        // At one point the VID & PID were set to 0 and it's unclear how.
        assert_ne!(vid_bytes[0], 0, "VID low byte is 0.");
        buf[6] = vid_bytes[0];
        buf[7] = vid_bytes[1];

        // Bytes 8 & 9 -- USB Product ID (PID)
        let pid_bytes = self.usb_product_id.to_le_bytes();
        // At one point the VID & PID were set to 0 and it's unclear how.
        assert_ne!(pid_bytes[0], 0, "PID low byte is 0.");
        buf[8] = pid_bytes[0];
        buf[9] = pid_bytes[1];

        // Bytes 10 & 11 -- USB power settings
        buf[10] = self.usb_power_attributes;
        // Note that the stored value is _half_ the actual requested mA.
        // When reading we double the value to be less confusing to users.
        buf[11] = (self.usb_requested_number_of_ma / 2) as u8;

        // TODO: Password support (bytes 12..=19).
    }
}
