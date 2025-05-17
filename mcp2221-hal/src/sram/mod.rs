use bit_field::BitField;

use crate::analog::VoltageReference;
use crate::common::ClockSetting;
use crate::gpio::GpSettings;
use crate::security::ChipConfigurationSecurity;

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
    /// (not 48), and the "divider" value is better thought of selecting a duty
    /// cycle and a frequency.
    ///
    /// Datasheet description for reference (table 3-39):
    ///
    /// > If the GP pin (exposing the clock output) is enabled for clock output
    /// > operation, the divider value will be used on the 48 MHz USB internal
    /// > clock and its divided output will be sent to this pin.
    /// > (Bits[4:3] for duty cycle and bits[2:0] for the clock divider.)
    ///
    /// Byte 5 bits 4..=0.
    pub clock_output: ClockSetting,
    /// DAC reference source (Vrm or Vdd) and Vrm setting
    ///
    /// Vrm setting at byte 6, bits 6 & 7. Vrm/Vdd selection at bit 5.
    pub dac_reference: VoltageReference,
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
    /// ADC reference source (Vrm or Vdd) and Vrm setting
    ///
    /// Vrm setting at byte 7 bits 3 & 4, Vrm/Vdd selection at bit 2.
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
    pub gp_settings: GpSettings,
}

impl SramSettings {
    /// Create [`SramSettings`] from a 64-byte report read from the MCP2221.
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            chip_configuration_security: buf[4].get_bits(0..=1).into(),
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
            gp_settings: GpSettings::from_sram_buffer(buf),
        }
    }
}

#[derive(Debug)]
pub struct InterruptSettings {
    pub clear_interrupt_flag: bool,
    pub interrupt_on_positive_edge: Option<bool>,
    pub interrupt_on_negative_edge: Option<bool>,
}

impl InterruptSettings {
    pub fn clear_flag(clear: bool) -> Self {
        Self {
            clear_interrupt_flag: clear,
            interrupt_on_positive_edge: None,
            interrupt_on_negative_edge: None,
        }
    }
}

/// Changes to be applied to the settings in SRAM.
#[derive(Debug, Default)]
pub struct ChangeSramSettings {
    /// Clock output settings.
    clock_output: Option<ClockSetting>,
    /// DAC voltage reference.
    dac_reference: Option<VoltageReference>,
    /// DAC output value `(0..=31)`
    dac_value: Option<u8>,
    /// ADC voltage reference
    adc_reference: Option<VoltageReference>,
    /// Interrupt settings
    interrupt_settings: Option<InterruptSettings>,
    /// GP pin settings
    gp_settings: Option<GpSettings>,
}

impl ChangeSramSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the clock output (CLKR) duty cycle and frequency.
    pub fn with_clock_output(&mut self, clock: ClockSetting) -> &mut Self {
        self.clock_output = Some(clock);
        self
    }

    /// Change the DAC voltage reference.
    pub fn with_dac_reference(&mut self, vref: VoltageReference) -> &mut Self {
        self.dac_reference = Some(vref);
        self
    }

    /// Change the DAC output value.
    ///
    /// `value` must be a valid 5-bit value `(0..=31)`.
    pub fn with_dac_value(&mut self, value: u8) -> &mut Self {
        // TODO: If this is publicly exposed it should probably return an
        // error rather than panicking. Or perhaps clamp to 31?
        assert!(
            value < 32,
            "DAC output value is out of range ({value} > 31)"
        );
        self.dac_value = Some(value);
        self
    }

    /// Change the ADC voltage reference.
    pub fn with_adc_reference(&mut self, vref: VoltageReference) -> &mut Self {
        self.adc_reference = Some(vref);
        self
    }

    /// Change the interrupt settings or clear the interrupt status.
    pub fn with_interrupt_settings(&mut self, interrupt_settings: InterruptSettings) -> &mut Self {
        self.interrupt_settings = Some(interrupt_settings);
        self
    }

    /// Change the GP pin modes.
    ///
    /// If you only want to change GPIO pin output level or direction, prefer to use
    /// [`crate::MCP2221::set_gpio_values`].
    ///
    /// <div class="warning">
    /// This function takes voltage references for the DAC and ADC because changing
    /// the GP pin settings causes “the reference voltage for Vrm” to be “reinitialized
    /// to the default value (Vdd) if not explicitly set” (section 1.8.11 in the
    /// datasheet). In practice, this sets the Vrm level to “off”, however this is not
    /// visible when reading the SRAM settings, only by reading the voltage output
    /// from the DAC.
    /// </div>
    ///
    /// Calling this function with a `None` value for either after using
    /// [`Self::with_dac_reference()`] or [`Self::with_adc_reference`]
    /// **will not** overwrite the previous to-be-set value.
    // TODO: Find out what the datasheet actually means when it says "Vrm is always
    // reinit to Vdd".
    pub fn with_gp_modes(
        &mut self,
        gp_settings: GpSettings,
        dac_reference: Option<VoltageReference>,
        adc_reference: Option<VoltageReference>,
    ) -> &mut Self {
        self.gp_settings = Some(gp_settings);
        if dac_reference.is_some() {
            self.dac_reference = dac_reference;
        }
        if adc_reference.is_some() {
            self.adc_reference = adc_reference;
        }
        self
    }

    pub(crate) fn apply_to_sram_buffer(&self, buf: &mut [u8; 64]) {
        // Byte 2: Clock output duty cycle & frequency
        if let Some(clock) = self.clock_output {
            // Enable loading of a new clock "divider".
            buf[2].set_bit(7, true);
            buf[2].set_bits(0..=4, clock.into());
        }
        // Byte 3: DAC voltage reference
        if let Some(dac_vref) = self.dac_reference {
            let (vrm_vdd, vrm_level) = dac_vref.into();
            // Enable loading of a new DAC reference.
            buf[3].set_bit(7, true);
            buf[3].set_bits(1..=2, vrm_level);
            buf[3].set_bit(0, vrm_vdd);
        }
        // Byte 4: DAC output value
        if let Some(value) = self.dac_value {
            // Enable loading of a new DAC value.
            buf[4].set_bit(7, true);
            // TODO: This will panic if `value` is out of range.
            buf[4].set_bits(0..=4, value);
        }
        // Byte 5: ADC voltage reference
        if let Some(adc_vref) = self.adc_reference {
            let (vrm_vdd, vrm_level) = adc_vref.into();
            // Enable loading of a new ADC reference.
            buf[5].set_bit(7, true);
            buf[5].set_bits(1..=2, vrm_level);
            buf[5].set_bit(0, vrm_vdd);
        }
        // Byte 6: Interrupt settings
        if let Some(ref interrupts) = self.interrupt_settings {
            // Enable the modification of the interrupt detection conditions.
            buf[6].set_bit(7, true);
            if let Some(trigger) = interrupts.interrupt_on_positive_edge {
                // Enable the modification of the positive edge detection.
                buf[6].set_bit(4, true);
                buf[6].set_bit(3, trigger);
            }
            if let Some(trigger) = interrupts.interrupt_on_negative_edge {
                // Enable the modification of the negative edge detection.
                buf[6].set_bit(2, true);
                buf[6].set_bit(1, trigger);
            }
            // Clear the interrupt detection flag?
            buf[6].set_bit(0, interrupts.clear_interrupt_flag);
        }
        // Byte 7..=11: GP pin settings
        if let Some(ref gp_settings) = self.gp_settings {
            // Alter GPIO configuration?
            buf[7].set_bit(7, true);

            // GP0 settings
            buf[8].set_bit(4, gp_settings.gp0.value.into());
            buf[8].set_bit(3, gp_settings.gp0.direction.into());
            buf[8].set_bits(0..=2, gp_settings.gp0.designation.into());

            // GP1 settings
            buf[9].set_bit(4, gp_settings.gp1.value.into());
            buf[9].set_bit(3, gp_settings.gp1.direction.into());
            buf[9].set_bits(0..=2, gp_settings.gp1.designation.into());

            // GP2 settings
            buf[10].set_bit(4, gp_settings.gp2.value.into());
            buf[10].set_bit(3, gp_settings.gp2.direction.into());
            buf[10].set_bits(0..=2, gp_settings.gp2.designation.into());

            // GP3 settings
            buf[11].set_bit(4, gp_settings.gp3.value.into());
            buf[11].set_bit(3, gp_settings.gp3.direction.into());
            buf[11].set_bits(0..=2, gp_settings.gp3.designation.into());
        }
    }
}
