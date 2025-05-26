use super::MCP2221;
use crate::Error;
use crate::commands::{McpCommand, UsbReport};
use crate::gpio::{GpioChanges, GpioValues, Pins};

impl MCP2221 {
    /// Take the four GP pin structs for individual GPIO operation.
    ///
    /// This can only be done once, and will return `None` afterwards.
    pub fn take_pins(&self) -> Option<Pins> {
        if self.pins_taken.get() {
            None
        } else {
            self.pins_taken.set(true);
            Some(Pins::new(self))
        }
    }

    /// Get GPIO pin direction and current logic levels.
    ///
    /// The logic level listed for input pins is the value read at that pin, and for
    /// output pins it is the currently set output. Only pins that are configured for
    /// GPIO operation are present in the returned struct.
    ///
    /// <div class="warning">
    ///
    /// You should prefer this method over [`MCP2221::sram_read_settings`] to read the
    /// state of the GPIO pins as that does not provide input pin readings (the level
    /// listed for GPIO pins is the pin's set output level) and it may not show the
    /// current direction of a GPIO pin.
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 3.1.12 for the underlying Get GPIO Values HID command.
    pub fn gpio_read(&self) -> Result<GpioValues, Error> {
        let buf = self
            .transfer(&UsbReport::new(McpCommand::GetGpioValues))?
            .expect("Always has response buffer.");
        Ok(GpioValues::from_buffer(&buf))
    }

    /// Change GPIO pins' direction and output logic level.
    ///
    /// You should prefer this method over [`MCP2221::sram_write_settings`] to change
    /// GPIO pin direction or output level, and use that method for altering the pin
    /// function (eg, GPIO, ADC, etc). Changing GP pin settings with that method
    /// requires an additional read command to the device to work around a firmware
    /// bug that resets analog voltage references (see the note in section 1.8.1.1 of
    /// the datasheet from revision D onwards).
    ///
    /// Note that this method will not change the mode of GP pins that are not set
    /// for GPIO operation. That must be done first by setting the pin mode, either
    /// temporarily via [`MCP2221::sram_write_settings`], or persistently via
    /// [`MCP2221::flash_write_gp_settings`] (and then resetting the device).
    ///
    /// The ability to set a pin as an input while also setting its output logic level
    /// reflects the structure of the underlying MCP2221 command but is otherwise
    /// meaningless.
    ///
    /// <div class="warning">
    ///
    /// Using this method will mean that the SRAM settings (as read through
    /// [`MCP2221::sram_read_settings`]) will not reflect the current GPIO pin
    /// direction and output level. This appears to be a bug in the MCP2221
    /// firmware and is not documented in the datasheet.
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 3.1.11 of the datasheet for the underlying Set GPIO Output Values
    /// HID command.
    pub fn gpio_write(&self, changes: &GpioChanges) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::SetGpioOutputValues);
        changes.apply_to_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }
}
