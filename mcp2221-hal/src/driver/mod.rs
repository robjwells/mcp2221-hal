use std::cell::Cell;

use hidapi::{HidApi, HidDevice};

use crate::commands::{McpCommand, UsbReport};
use crate::constants::{COMMAND_SUCCESS, MCP2221_PID, MICROCHIP_VID};
use crate::error::Error;
use crate::settings::{InterruptSettingsChanges, SramSettingsChanges};
use crate::status::Status;

mod analog;
mod flash;
mod gpio;
mod i2c;
mod i2c_eh;
mod sram;

/// Driver for the MCP2221.
///
/// # Overview
///
/// <!-- TODO -->.
///
/// # Creation
///
/// Create the driver struct with [`MCP2221::connect()`], which will use the first
/// device found with the default vendor ID (VID) and product ID (PID) numbers.
/// If you have changed either of them, use [`MCP2221::connect_with_vid_and_pid`].
#[derive(Debug)]
pub struct MCP2221 {
    /// Underlying [`hidapi`] device.
    ///
    /// The C hidapi library is not thread safe (`cargo test` will trigger a crash)
    /// and the `hidapi` types are appropriately `!Sync`.
    inner: HidDevice,
    /// Marker for whether the pin structs have been taken from the driver.
    ///
    /// This is used to fake "moving" the pins out of the driver, but really everything
    /// has a shared reference to the driver under the covers. The `Cell` is used to
    /// maintain requirement of only a shared reference. It is safe since the driver is
    /// `!Sync` anyway. See [`Self::take_pins`] for the only place it is used.
    pins_taken: Cell<bool>,
}

impl MCP2221 {
    ////////////////////////////////////////////////////////////////////////////////
    // Constructors - USB methods
    ////////////////////////////////////////////////////////////////////////////////

    /// Connect to the first USB device found with the default vendor and product ID.
    ///
    /// The default VID is 1240 (0x4D8) and PID 221 (0xDD) for both the original
    /// MCP2221 and the (more common) MCP2221A.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn connect() -> Result<Self, Error> {
        MCP2221::connect_with_vid_and_pid(MICROCHIP_VID, MCP2221_PID)
    }

    /// Connect to the first USB device found with the given vendor and product ID.
    ///
    /// Use this constructor if you have changed the USB VID or PID of your MCP2221.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn connect_with_vid_and_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self {
            inner: device,
            pins_taken: Cell::new(false),
        })
    }

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
            SramSettingsChanges::new()
                .with_interrupt_settings(InterruptSettingsChanges::clear_flag(true)),
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

    ////////////////////////////////////////////////////////////////////////////////
    // USB - miscellaneous
    ////////////////////////////////////////////////////////////////////////////////

    /// Get the USB HID device information from the host's USB interface.
    ///
    /// This is a thin wrapper around [`HidDevice::get_device_info`].
    ///
    /// # Errors
    ///
    /// An error will be returned if the device information cannot be obtained from the
    /// underlying USB interface.
    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        self.inner.get_device_info().map_err(Error::from)
    }
}
