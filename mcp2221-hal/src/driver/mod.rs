use std::cell::Cell;

use hidapi::HidDevice;

use crate::commands::{McpCommand, UsbReport};
use crate::constants::COMMAND_SUCCESS;
use crate::error::Error;
use crate::settings::{ChangeSramSettings, ChangeInterruptSettings};
use crate::status::Status;

mod analog;
mod flash;
mod gpio;
mod i2c;
mod i2c_eh;
mod sram;
mod usb;

/// Driver for the MCP2221.
#[derive(Debug)]
pub struct MCP2221 {
    inner: HidDevice,
    pins_taken: Cell<bool>,
}

/// # HID Commands
///
/// Unless specifically noted, all methods that return a `Result` return an error
/// if there is a problem communicating with the MCP2221.
impl MCP2221 {
    /// Read the status of the MCP2221.
    ///
    /// The returned structure includes the current status of the I2C engine, the
    /// interrupt detection flag, ADC readings, and the device's hardware and
    /// firmware revision numbers.
    ///
    /// # Datasheet
    ///
    /// See section 3.11 of the datasheet for the underlying Status/Set Parameters
    /// HID command.
    pub fn status(&self) -> Result<Status, Error> {
        let buf = self
            .transfer(&UsbReport::new(McpCommand::StatusSetParameters))?
            .expect("Always has response buffer.");
        Ok(Status::from_buffer(&buf))
    }

    /// Reset the MCP2221.
    ///
    /// This can be useful after changing settings in the device's flash memory,
    /// which only take effect on power-up.
    ///
    /// Resetting the chip causes the device to re-enumerate with the USB host,
    /// so you will need to create a new driver struct afterwards.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.15 for the underlying Reset Chip HID command, and section
    /// 4.2.3 for reset timings.
    pub fn reset(self) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::ResetChip);
        command.set_data_byte(2, 0xCD);
        command.set_data_byte(3, 0xEF);

        self.transfer(&command)?;
        Ok(())
    }

    /// Read the edge-triggered interrupt flag.
    ///
    /// GP1 can be configured to detect external interrupts on rising or falling edges.
    /// If so, the interrupt flag will be set when an edge is detected.
    ///
    /// ## Datasheet
    ///
    /// There is very little about interrupt detection in the MCP2221 datasheet. See
    /// byte 6 in table 3-36 for descriptions of the related settings. See section
    /// 1.10 for a very brief general overview.
    pub fn interrupt_detected(&self) -> Result<bool, Error> {
        self.status().map(|s| s.interrupt_detected)
    }

    /// Clear the edge-triggered interrupt flag.
    ///
    /// Clears the flag indicating that an edge has been detected on GP1, when GP1
    /// is in interrupt-detection mode. The interrupt detection conditions (positive
    /// edge, negative edge, or both) are not changed.
    ///
    /// ## Datasheet
    ///
    /// There is very little about interrupt detection in the MCP2221 datasheet. See
    /// byte 6 in table 3-36 for descriptions of the related settings. See section
    /// 1.10 for a very brief general overview.
    pub fn clear_interrupt_flag(&self) -> Result<(), Error> {
        self.sram_write_settings(
            ChangeSramSettings::new().with_interrupt_settings(ChangeInterruptSettings::clear_flag(true)),
        )
    }

    /// Write the given command to the MCP and read the 64-byte response.
    ///
    /// Returning an optional buffer is not great for the callers' ergonomics
    /// but it's the most straightforward way of representing the non-response
    /// from Reset Chip that doesn't return an empty array.
    fn transfer(&self, command: &UsbReport) -> Result<Option<[u8; 64]>, Error> {
        let out_command_byte = command.write_buffer[0];
        let written = self.inner.write(&command.report_bytes())?;
        if command.has_no_response() {
            return Ok(None);
        }

        let mut read_buffer = [0u8; 64];
        let read = self.inner.read(&mut read_buffer)?;
        let read_command_byte = read_buffer[0];

        // Check length written and read.
        assert_eq!(written, 65, "Didn't write full report.");
        assert_eq!(read, 64, "Didn't read full report.");

        // Check command-code echo.
        if read_command_byte != out_command_byte {
            return Err(Error::MismatchedCommandCodeEcho {
                sent: out_command_byte,
                received: read_command_byte,
            });
        }

        let status_code = read_buffer[1];
        if status_code == COMMAND_SUCCESS {
            Ok(Some(read_buffer))
        } else {
            // Command has failed, so we check the command to see if there is a more
            // specific Error case, otherwise we return the most general one, and
            // enclose the failure code.
            command
                .check_error_code(status_code)
                .and(Err(Error::CommandFailed(status_code)))
        }
    }
}
