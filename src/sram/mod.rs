use crate::flash_data::{
    GpSettings,
    common::{ChipConfigurationSecurity, DacVoltageReferenceSource, VrmVoltageReference},
};

#[derive(Debug)]
pub struct SramSettings {
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
    /// (not 48), and the "divider" value is better thought of as an option.
    ///
    /// Datasheet description for reference (table 3-39):
    ///
    /// > If the GP pin (exposing the clock output) is enabled for clock output
    /// > operation, the divider value will be used on the 48 MHz USB internal
    /// > clock and its divided output will be sent to this pin.
    /// > (Bits[4:3] for duty cycle and bits[2:0] for the clock divider.)
    ///
    /// Byte 5 bits 4..=0.
    pub clock_output_divider: u8,
    /// DAC reference voltage (Vrm setting)
    ///
    /// Byte 6 bits 7 & 6.
    pub dac_reference_voltage: VrmVoltageReference,
    /// DAC reference source (Vrm or Vdd)
    ///
    /// Byte 6 bit 5.
    pub dac_reference_source: DacVoltageReferenceSource,
    /// DAC value.
    ///
    /// The datasheet calls this the "power-up DAC value" but it is the current DAC
    /// output value.
    ///
    /// Byte 6 bits 4..=0. Value in range 0..=31.
    pub dac_value: u8,
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
    pub adc_reference_voltage: VrmVoltageReference,
    /// ADC reference source (Vrm or Vdd)
    ///
    /// **NOTE** this is inverted compared to the corresponding flash setting,
    /// here 1 = Vrm, 0 = Vdd. I'm using a bool at the moment because the
    /// AdcVoltageReferenceSource struct was written to the flash setting
    /// description.
    ///
    /// Byte 7 bit 2.
    // pub adc_reference_source: AdcVoltageReferenceSource,
    pub adc_reference_option: bool,
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
    // TODO: support password (bytes 14..=21).
    /// GP pin settings.
    ///
    /// Bytes 22..=25.
    // TODO: GpSettings references the "power-up" settings in its field names.
    gp_settings: GpSettings,
}

impl SramSettings {
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            chip_configuration_security: buf[4].get_bits(0..=1).into(),
            clock_output_divider: buf[5].get_bits(0..=4),
            dac_reference_voltage: buf[6].get_bits(6..=7).into(),
            dac_reference_source: buf[6].get_bit(5).into(),
            dac_value: buf[6].get_bits(0..=4),
            interrupt_on_negative_edge: buf[7].get_bit(6),
            interrupt_on_positive_edge: buf[7].get_bit(5),
            adc_reference_voltage: buf[7].get_bits(3..=4).into(),
            adc_reference_option: buf[7].get_bit(2),
            usb_vendor_id: u16::from_le_bytes([buf[8], buf[9]]),
            usb_product_id: u16::from_le_bytes([buf[10], buf[11]]),
            usb_power_attributes: buf[12],
            usb_requested_number_of_ma: buf[13] as u16 * 2,
            gp_settings: GpSettings::from_sram_buffer(buf),
        }
    }
}
