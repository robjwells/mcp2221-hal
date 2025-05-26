use crate::analog::VoltageReference;
use crate::settings::common::ClockOutputSetting;

use bit_field::BitField;

#[derive(Debug)]
/// Various chip configuration settings.
///
/// This struct gathers together several important but somewhat unrelated settings.
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
/// In this driver, this only applies when changing the GP pin settings via
/// [`MCP2221::sram_write_settings`], which is why [`SramSettingsChanges::with_gp_modes`]
/// takes an option DAC and ADC voltage reference.
///
/// [`MCP2221::sram_write_settings`]: crate::MCP2221::sram_write_settings
/// [`SramSettingsChanges::with_gp_modes`]: crate::settings::SramSettingsChanges::with_gp_modes
///
/// </div>
///
/// ## Datasheet
///
/// See table 3-5 in section 3.1.2 (Read Flash Data) or table 3-39 in section 3.1.14
/// (Get SRAM Settings) for the datasheet's listing of each returned value.
pub struct ChipSettings {
    /// Whether the USB serial number string will be presented during USB enumeration.
    ///
    /// This can be used to provide a stable name to the MCP2221 serial port on the
    /// USB host.
    pub cdc_serial_number_enumeration_enabled: bool,
    /// Clock Output setting.
    ///
    /// If GP1 is set to clock output (aka CLK_OUT or CLKR), this value determines its
    /// duty cycle and frequency.
    ///
    /// ## Datasheet
    ///
    /// See register 1-2 for the specification of this value. Elsewhere in the datasheet
    /// it is described simply as a "divider", which is not quite accurate as the bit
    /// pattern selects both a duty cycle and frequency.
    pub clock_output: ClockOutputSetting,
    /// DAC reference source (Vrm or Vdd) and Vrm setting.
    ///
    /// <div class="warning">
    ///
    /// If set in flash memory to Vrm (at any voltage level), the DAC will behave
    /// strangely on power-up. See the [`analog`] and [`settings`] module documentation
    /// for more details
    ///
    /// </div>
    ///
    /// [`analog`]: crate::analog
    /// [`settings`]: crate::settings
    pub dac_reference: VoltageReference,
    /// DAC output value.
    ///
    /// The 5-bit value that determines the voltage output of the DAC. The upper three
    /// bits are ignored.
    pub dac_value: u8,
    /// Set the interrupt flag when a negative edge is detected on GP1.
    ///
    /// Note GP1 must be configured for interrupt detection for this to have an effect.
    /// See [`Gp1Mode::InterruptDetection`].
    ///
    /// [`Gp1Mode::InterruptDetection`]: crate::settings::Gp1Mode
    pub interrupt_on_negative_edge: bool,
    /// Set the interrupt flag when a positive edge is detected on GP1.
    ///
    /// Note GP1 must be configured for interrupt detection for this to have an effect.
    /// See [`Gp1Mode::InterruptDetection`].
    ///
    /// [`Gp1Mode::InterruptDetection`]: crate::settings::Gp1Mode
    pub interrupt_on_positive_edge: bool,
    /// ADC reference source (Vrm or Vdd) and Vrm setting.
    pub adc_reference: VoltageReference,
    /// USB Vendor ID (VID).
    pub usb_vendor_id: u16,
    /// USB Product ID (PID).
    pub usb_product_id: u16,
    /// USB power attributes.
    ///
    /// ## Datasheet
    ///
    /// This is not explained in the datasheet beyond the following description
    /// in table 3-12:
    ///
    /// > This value will be used by the MCP2221A's USB Configuration Descriptor (power
    /// > attributes value) during USB enumeration.
    ///
    /// And this note under table 3-5:
    ///
    /// > Please consult the USB 2.0 specification for details on the correct values for
    /// > power and attributes.
    pub usb_power_attributes: u8,
    /// Current requested during USB enumeration in milliamps.
    ///
    /// Note that this is stored as halved value in a single byte, so the maximum
    /// current request is 510mA.
    pub usb_requested_number_of_ma: u16,
}

impl ChipSettings {
    /// Parse the buffer returned from the MCP2221.
    ///
    /// The flash and SRAM chip settings response buffers use the same layout.
    ///
    /// ## Datasheet
    ///
    /// See table 3-5 for the flash response layout and table 3-39 for the SRAM response.
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            clock_output: buf[5].get_bits(0..=4).into(),
            dac_reference: (buf[6].get_bit(5), buf[6].get_bits(6..=7)).into(),
            dac_value: buf[6].get_bits(0..=4),
            interrupt_on_negative_edge: buf[7].get_bit(6),
            interrupt_on_positive_edge: buf[7].get_bit(5),
            adc_reference: (buf[7].get_bit(2), buf[7].get_bits(3..=4)).into(),
            usb_vendor_id: u16::from_le_bytes([buf[8], buf[9]]),
            usb_product_id: u16::from_le_bytes([buf[10], buf[11]]),
            usb_power_attributes: buf[12],
            usb_requested_number_of_ma: buf[13] as u16 * 2,
        }
    }

    /// Apply the settings to the flash output buffer for writing to the MCP2221.
    ///
    /// Note there is no corresponding method for the SRAM settings, as they must
    /// be set in a different manner.
    ///
    /// ## Datasheet
    ///
    /// See table 3-12 in section 3.1.3 (Write Flash Data) for the layout of the
    /// settings to be written.
    pub(crate) fn apply_to_flash_buffer(&self, buf: &mut [u8; 64]) {
        // Note the bytes positions when writing are -2 from the position when reading.
        buf[2].set_bit(7, self.cdc_serial_number_enumeration_enabled);

        // Byte 3 (write) / byte 5 (read)
        buf[3].set_bits(0..=4, self.clock_output.into());

        // Byte 4 (write) / byte 6 (read) -- DAC settings
        let (dac_vrm_vdd, dac_vrm_level) = self.dac_reference.into();
        buf[4].set_bits(6..=7, dac_vrm_level);
        buf[4].set_bit(5, dac_vrm_vdd);
        // Limit the DAC value to a maximum of 31 to avoid panicking.
        // Because we don't use setters for ChipSettings, this has to be done here.
        buf[4].set_bits(0..=4, self.dac_value & 31);

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
        // (Though perhaps not those who have spent too long reading the datasheet!)
        buf[11] = (self.usb_requested_number_of_ma / 2) as u8;
    }
}
