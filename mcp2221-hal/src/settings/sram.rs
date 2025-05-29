//! SRAM settings.

use bit_field::BitField;

use crate::analog::VoltageReference;
use crate::settings::{ClockOutputSetting, GpSettings};

/// Changes to be applied to interrupt settings in SRAM.
///
/// GP1 can be configured to detect positive or negative edges and set an interrupt
/// flag when one occurs (see [`Gp1Mode::InterruptDetection`]).
///
/// [`Gp1Mode::InterruptDetection`]: crate::settings::Gp1Mode::InterruptDetection
///
/// This struct offers a builder-like interface to adjust the interrupt settings, to
/// be given as an argument to [`SramSettingsChanges::with_interrupt_settings`]. The
/// only required setting is whether to clear the interrupt flag or not, so the
/// constructor is [`Self::clear_flag`].
///
/// If you only want to clear the interrupt flag, prefer [`MCP2221::interrupt_clear`].
///
/// [`MCP2221::interrupt_clear`]: crate::MCP2221::interrupt_clear
#[derive(Debug, Clone, Copy)]
pub struct InterruptSettingsChanges {
    /// Clear the interrupt flag if true.
    clear_interrupt_flag: bool,
    /// If `Some`, set whether interrupts should trigger on a positive edge.
    interrupt_on_positive_edge: Option<bool>,
    /// If `Some`, set whether interrupts should trigger on a negative edge.
    interrupt_on_negative_edge: Option<bool>,
}

impl InterruptSettingsChanges {
    /// Create a new struct set to clear (or not) the interrupt flag.
    ///
    /// The "clear flag" argument is the only thing required when changing the
    /// interrupt settings in SRAM.
    pub fn clear_flag(clear: bool) -> Self {
        Self {
            clear_interrupt_flag: clear,
            interrupt_on_positive_edge: None,
            interrupt_on_negative_edge: None,
        }
    }

    /// Enable or disable interrupts when a positive edge is detected.
    pub fn interrupt_on_positive_edge(&mut self, v: bool) -> &mut Self {
        self.interrupt_on_positive_edge = Some(v);
        self
    }

    /// Enable or disable interrupts when a negative edge is detected.
    pub fn interrupt_on_negative_edge(&mut self, v: bool) -> &mut Self {
        self.interrupt_on_negative_edge = Some(v);
        self
    }
}

/// Changes to be applied to the settings in SRAM.
///
/// Only a subset of the full chip settings can be changed in SRAM. The rest can be
/// changed in flash memory, to take effect after the device is reset.
#[derive(Debug, Default, Clone, Copy)]
pub struct SramSettingsChanges {
    /// Clock output settings.
    clock_output: Option<ClockOutputSetting>,
    /// DAC voltage reference.
    dac_reference: Option<VoltageReference>,
    /// DAC output value `(0..=31)`.
    dac_value: Option<u8>,
    /// ADC voltage reference.
    adc_reference: Option<VoltageReference>,
    /// Interrupt settings.
    interrupt_settings: Option<InterruptSettingsChanges>,
    /// GP pin settings.
    gp_settings: Option<GpSettings>,
}

impl SramSettingsChanges {
    /// Create an empty set of changes to SRAM.
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the clock output (CLK_OUT or CLKR) duty cycle and frequency.
    pub fn with_clock_output(&mut self, clock: ClockOutputSetting) -> &mut Self {
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
    /// Values above the 5-bit range of the DAC (`0..=31`) are clamped to the maximum
    /// value of 31.
    pub fn with_dac_value(&mut self, value: u8) -> &mut Self {
        self.dac_value = Some(value & 31);
        self
    }

    /// Change the ADC voltage reference.
    pub fn with_adc_reference(&mut self, vref: VoltageReference) -> &mut Self {
        self.adc_reference = Some(vref);
        self
    }

    /// Change the interrupt settings or clear the interrupt flag.
    pub fn with_interrupt_settings(
        &mut self,
        interrupt_settings: InterruptSettingsChanges,
    ) -> &mut Self {
        self.interrupt_settings = Some(interrupt_settings);
        self
    }

    /// Change the GP pin modes.
    ///
    /// If you only want to change GPIO pin output level or direction, prefer to use
    /// [`MCP2221::gpio_write`](crate::MCP2221::gpio_write).
    ///
    /// <div class="warning">
    ///
    /// This function takes voltage references for the DAC and ADC because changing the
    /// GP pin settings causes “the reference voltage for Vrm” to be “reinitialized to
    /// the default value (Vdd) if not explicitly set” (section 1.8.11 in the
    /// datasheet). In practice, this sets the Vrm level to “off”, however this is not
    /// visible when reading the SRAM settings, only by reading the voltage output from
    /// the DAC. See the [`analog`] module documentation for more information about the
    /// effect of setting the Vrm to "off" (it is almost certainly not what you want).
    ///
    /// </div>
    ///
    /// Calling this method with a `None` value for either will not overwrite
    /// an ADC or DAC voltage reference (as appropriate) set previously via
    /// [`with_dac_reference`] or [`with_adc_reference`].
    ///
    /// [`analog`]: crate::analog
    /// [`with_dac_reference`]: Self::with_dac_reference
    /// [`with_adc_reference`]: Self::with_adc_reference
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
            // with_dac_value limits the DAC output value to 31.
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
            buf[8].set_bit(4, gp_settings.gp0_value.into());
            buf[8].set_bit(3, gp_settings.gp0_direction.into());
            buf[8].set_bits(0..=2, gp_settings.gp0_mode.into());

            // GP1 settings
            buf[9].set_bit(4, gp_settings.gp1_value.into());
            buf[9].set_bit(3, gp_settings.gp1_direction.into());
            buf[9].set_bits(0..=2, gp_settings.gp1_mode.into());

            // GP2 settings
            buf[10].set_bit(4, gp_settings.gp2_value.into());
            buf[10].set_bit(3, gp_settings.gp2_direction.into());
            buf[10].set_bits(0..=2, gp_settings.gp2_mode.into());

            // GP3 settings
            buf[11].set_bit(4, gp_settings.gp3_value.into());
            buf[11].set_bit(3, gp_settings.gp3_direction.into());
            buf[11].set_bits(0..=2, gp_settings.gp3_mode.into());
        }
    }
}
