use super::MCP2221;
use crate::analog::{AdcReading, VoltageReference};
use crate::{ChangeSramSettings, Error};

impl MCP2221 {
    /// Configure the DAC voltage reference in SRAM.
    ///
    /// This will alter the current behaviour of the MCP2221 but will not persist
    /// across device reset.
    ///
    /// <div class="warning">
    ///
    /// Setting the DAC reference to Vrm with a level of "off" will cause the output
    /// voltage to be just above 0V at all output values. The datasheet suggests (in
    /// section 1.8.1.1) that "off" means that Vrm will reference Vdd (the supply
    /// voltage). This is true for the ADC but _not_ the DAC. Just use Vdd instead.
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 1.8.3 for information about the 5-bit DAC, section 1.8.1.1 for
    /// details about Vrm (with the caveat listed above), and section 3.1.13 for
    /// the underlying Set SRAM Settings HID command.
    pub fn dac_set_reference(&self, source: VoltageReference) -> Result<(), Error> {
        self.sram_write_settings(ChangeSramSettings::new().with_dac_reference(source))?;
        Ok(())
    }

    /// Perform an analog write to the DAC.
    ///
    /// This writes a 5-bit value to the MCP2221’s digital-to-analog converter, which
    /// outputs a corresponding voltage on appropriately configured pins. GP2 and GP3
    /// can be used for analog output pins, though they share the single DAC and will
    /// have the same voltage.
    ///
    /// Values above the 5-bit range of the DAC (`0..=31`) are clamped to the
    /// maximum value of 31.
    ///
    /// Note that the DAC output is not linear from 0V to the reference and (at least
    /// with 3.3V supply) does not reach the reference voltage. This is detailed in
    /// the crate readme.
    ///
    /// This setting is not persisted across reset. See [`MCP2221::flash_write_chip_settings`]
    /// to set the DAC to output a particular value at power-on.
    ///
    /// # Datasheet
    ///
    /// See section 1.8.3 for information about the 5-bit DAC, and section 3.1.13 for
    /// the underlying Set SRAM Settings HID command.
    pub fn analog_write(&self, value: u8) -> Result<(), Error> {
        // with_dac_value limits the value to 31.
        self.sram_write_settings(ChangeSramSettings::new().with_dac_value(value))?;
        Ok(())
    }

    /// Read the current values of the three-channel ADC.
    ///
    /// Pins GP1, GP2, and GP3 are connected to separate channels of the ADC, and the
    /// return value will contain the analog reading for each if that pin is configured
    /// as an analog input. The current ADC voltage reference is included so that you
    /// may convert a 10-bit reading to a voltage (`reading / 1023 * Vref`).
    ///
    /// # Internals
    ///
    /// The ADC readings are reported in the [`Status`] structure and are always
    /// available. In practice, these readings are what you'd expect no matter the
    /// set mode of the pin (GPIO output low is 0, and high 1023, for example).
    /// However, the datasheet makes no claims about behaviour in this state, so
    /// it's officially undefined and unsupported.
    ///
    /// [`Status`]: crate::status::Status
    ///
    /// # Datasheet
    ///
    /// See section 1.8.2 for information about the 10-bit ADC and section 3.1.1 for
    /// the underlying Status/Set Parameters HID command.
    pub fn analog_read(&self) -> Result<AdcReading, Error> {
        let raw = self.status()?.adc_values;
        let sram_settings = self.sram_read_settings()?;
        let vref = sram_settings.chip_settings.adc_reference;
        let gp = sram_settings.gp_settings;
        let reading = AdcReading {
            vref,
            gp1: gp.gp1.is_adc().then_some(raw.ch1),
            gp2: gp.gp2.is_adc().then_some(raw.ch2),
            gp3: gp.gp3.is_adc().then_some(raw.ch3),
        };
        Ok(reading)
    }

    /// Configure the ADC voltage reference in SRAM.
    ///
    /// This will alter the current behaviour of the MCP2221 but will not persist
    /// across device reset.
    ///
    /// Unlike with the DAC, setting the ADC reference to Vrm with a level of "off"
    /// results in a reference that seems to be equivalent to Vdd (as the datasheet
    /// suggests).
    ///
    /// # Datasheet
    ///
    /// See section 1.8.2 for information about the 10-bit ADC, section 1.8.1.1 for
    /// details about Vrm, and section 3.1.13 for the underlying Set SRAM Settings
    /// HID command.
    pub fn adc_set_reference(&self, source: VoltageReference) -> Result<(), Error> {
        self.sram_write_settings(ChangeSramSettings::new().with_adc_reference(source))?;
        Ok(())
    }
}
