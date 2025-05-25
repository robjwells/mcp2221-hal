use super::MCP2221;
use crate::Error;
use crate::commands::{McpCommand, UsbReport};
use crate::settings::{ChangeSramSettings, GpSettings, SramSettings};

impl MCP2221 {
    /// Retrieve the chip and GP pin settings stored in SRAM.
    ///
    /// The settings read from SRAM match the structure of the [`ChipSettings`] stored
    /// in flash, with the addition of the [`GpSettings`].
    ///
    /// [`ChipSettings`]: crate::settings::ChipSettings
    ///
    /// <div class="warning">
    ///
    /// Do not rely on the returned [`SramSettings`] accurately reflecting the current
    /// state of the MCP2221. Some commands will (in practice) change these settings
    /// without those changes being shown when subsequently reading the SRAM.
    ///
    /// - GPIO pin direction and level after using the Set GPIO Output Values HID
    ///   command (implemented by [`MCP2221::gpio_write`]).
    /// - Vrm reference level set to "off" after setting GP pin settings via the Set
    ///   SRAM Settings HID command (implemented by [`MCP2221::sram_write_settings`])
    ///   _without_ also explicitly setting the Vrm level. See the note in section
    ///   1.8.1.1 of the datasheet, as well as the documentation for
    ///   [`ChangeSramSettings::with_gp_modes`].
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 3.1.14 of the datasheet for details about the underlying Get SRAM
    /// Settings HID command, and section 1.4 for information about the configuration
    /// process at power-up.
    pub fn sram_read_settings(&self) -> Result<SramSettings, Error> {
        let command = UsbReport::new(McpCommand::GetSRAMSettings);
        let buf = self
            .transfer(&command)?
            .expect("Always has response buffer.");
        SramSettings::try_from_buffer(&buf)
    }

    /// Change run-time chip and GP pin settings.
    ///
    /// This will alter the current behaviour of the MCP2221 but will not persist
    /// across device reset. Note that only a subset of the settings read from SRAM
    /// can be changed.
    ///
    /// If you only need to change GPIO pin direction or output level, you should
    /// prefer to use [`MCP2221::gpio_write`].
    ///
    /// <div class="warning">
    ///
    /// Changing the GP pin settings without also setting Vrm levels for the ADC and
    /// DAC will result in the Vrm level for each being reset to "off". This appears
    /// to be a MCP2221 firmware bug and is noted in section 1.8.1.1 of the datasheet.
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 3.1.13 of the datasheet for details about the underlying Set SRAM
    /// Settings HID command.
    pub fn sram_write_settings(&self, settings: &ChangeSramSettings) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::SetSRAMSettings);
        settings.apply_to_sram_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Change the GP pin settings in SRAM while preserving the ADC and DAC references.
    ///
    /// This is a convenience wrapper around [`Self::sram_write_settings`] that does
    /// the work of reading the current ADC & DAC voltage references and re-writing
    /// them, to avoid the Vrm reset bug.
    pub fn sram_write_gp_settings(&self, gp_settings: GpSettings) -> Result<(), Error> {
        let current = self.sram_read_settings()?;
        self.sram_write_settings(ChangeSramSettings::new().with_gp_modes(
            gp_settings,
            Some(current.chip_settings.dac_reference),
            Some(current.chip_settings.adc_reference),
        ))
    }
}
