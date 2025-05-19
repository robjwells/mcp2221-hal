use crate::analog::VoltageReference;
use crate::common::ClockSetting;
use crate::security::ChipConfigurationSecurity;

use bit_field::BitField;

#[derive(Debug)]
/// Various chip configuration settings.
///
/// The chip settings gathers together several important but unrelated settings.
/// Consult the documentation for each field and table 3-12 of the datasheet for
/// information on each option.
///
/// The chip settings layout is the same in both flash and SRAM, though fewer things
/// can be changed in the SRAM chip settings.
///
/// <div class="warning">
///
/// If the GP pin settings are changed in SRAM without also setting the Vrm level for
/// the ADC and DAC, the [`adc_reference`](Self::adc_reference) and
/// [`dac_reference`](Self::dac_reference) fields may not correspond to the actual
/// setting (a Vrm level of "off"). This appears to be an MCP2221 firmware bug and
/// is noted in section 1.8.1.1 of the datasheet.
///
/// </div>
///
/// # Datasheet
///
/// See table 3-5 in section 3.1.2 (Read Flash Data) or table 3-39 in section 3.1.14
/// (Get SRAM Settings) for the datasheet's listing of each returned value.
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
    /// (not 48), and the "divider" value is better thought of selecting a duty
    /// cycle and a frequency.
    ///
    /// Datasheet description for reference (table 3-5):
    ///
    /// > If the GP pin (exposing the clock output) is enabled for clock
    /// > output operation, the divider value will be used on the 48 MHz USB
    /// > internal clock and its divided output will be sent to this pin.
    ///
    /// Bits 3 & 4 are the duty cycle, bits 0..=2 are the frequency.
    ///
    /// Byte 5 bits 4..=0.
    pub clock_output: ClockSetting,
    /// DAC reference source (Vrm or Vdd) and Vrm setting
    ///
    /// Note that setting this to Vrm will cause the MCP2221, on boot, to behave as if
    /// the DAC was configured for Vrm with its reference level set to "Off", regardless
    /// of what you have set the DAC Vrm voltage to (eg 1.024V or 2.048V). This persists
    /// until you reconfigure the DAC settings in SRAM.
    ///
    /// If set to Vdd, the DAC will behave as expected upon boot.
    ///
    /// Vrm setting at byte 6 bits 6 & 7; Vrm/Vdd selection at bit 5 (1 = Vrm).
    pub dac_reference: VoltageReference,
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
    /// ADC reference source (Vrm or Vdd) and Vrm setting
    ///
    /// Note the datasheet "effect" column says this is the DAC reference,
    /// but it appears to be a typo. The DAC and ADC have their own
    /// voltage references (see section 1.8.1.1 of the datasheet).
    ///
    /// Vrm setting at bits 3 & 4; Vrm/Vdd selection at bit 2.
    pub adc_reference: VoltageReference,
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
    /// Descriptor (power attributes alue) during the USB enumeration.
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
    /// Parse the buffer returned from the MCP2221.
    ///
    /// The flash and SRAM chip settings response buffers use the same layout.
    ///
    /// # Datasheet
    ///
    /// See table 3-5 for the flash response layout and table 3-39 for the SRAM response.
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            chip_configuration_security: buf[4].get_bits(0..=1).into(),
            clock_output: buf[5].get_bits(0..=4).into(),
            dac_reference: (buf[6].get_bit(5), buf[6].get_bits(6..=7)).into(),
            dac_power_up_value: buf[6].get_bits(0..=4),
            interrupt_on_negative_edge: buf[7].get_bit(6),
            interrupt_on_positive_edge: buf[7].get_bit(5),
            adc_reference: (buf[7].get_bit(2), buf[7].get_bits(3..=4)).into(),
            usb_vendor_id: u16::from_le_bytes([buf[8], buf[9]]),
            usb_product_id: u16::from_le_bytes([buf[10], buf[11]]),
            usb_power_attributes: buf[12],
            usb_requested_number_of_ma: buf[13] as u16 * 2,
        }
    }

    pub(crate) fn apply_to_flash_buffer(&self, buf: &mut [u8; 64]) {
        // Note the bytes positions when writing are -2 from the position when reading.
        buf[2].set_bit(7, self.cdc_serial_number_enumeration_enabled);
        // TODO: support security settings.
        // While unimplemented, the 0 bits correspond to the "unsecured" setting.
        buf[2].set_bits(0..=1, ChipConfigurationSecurity::Unsecured.into());

        // Byte 3 (write) / byte 5 (read)
        buf[3].set_bits(0..=4, self.clock_output.into());

        // Byte 4 (write) / byte 6 (read) -- DAC settings
        let (dac_vrm_vdd, dac_vrm_level) = self.dac_reference.into();
        buf[4].set_bits(6..=7, dac_vrm_level);
        buf[4].set_bit(5, dac_vrm_vdd);
        buf[4].set_bits(0..=4, self.dac_power_up_value);

        // Byte 5 (write) / byte 6 (read) -- Interrupts and ADC
        buf[5].set_bit(6, self.interrupt_on_negative_edge);
        buf[5].set_bit(5, self.interrupt_on_positive_edge);

        let (adc_vrm_vdd, adc_vrm_level) = self.adc_reference.into();
        buf[5].set_bits(3..=4, adc_vrm_level);
        buf[5].set_bit(2, adc_vrm_vdd);

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
        // While unimplemented the password is left at its default (all zeroes).
    }
}
