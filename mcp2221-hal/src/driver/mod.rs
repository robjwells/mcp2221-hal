use std::cell::Cell;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use crate::Error;
use crate::analog::{AdcReading, VoltageReference};
use crate::commands::{FlashDataSubCode, McpCommand, UsbReport};
use crate::constants::{COMMAND_SUCCESS, MCP2221_PID, MICROCHIP_VID};
use crate::gpio::{GpioChanges, GpioValues, Pins};
use crate::i2c::{I2cCancelTransferResponse, I2cSpeed, ReadType, WriteType};
use crate::settings::{
    ChipSettings, DeviceString, GpSettings, InterruptSettingsChanges, SramSettingsChanges,
};
use crate::status::Status;

mod i2c_eh;

/// Driver for the MCP2221.
///
/// # Quick start
///
/// Create the driver struct with default values by calling [`MCP2221::connect`], or
/// [`MCP2221::connect_with_vid_and_pid`] if you have changed either of the USB vendor
/// ID (VID) or product ID (VID).
///
/// For I2C communication, this struct implements the [blocking][blocking I2C] and
/// [async][async I2C] I2C traits from [`embedded_hal`]. It has no mutable state, so
/// you can pass a shared reference to drivers expecting `impl I2c`.
///
/// [blocking I2C]: embedded_hal::i2c::I2c
/// [async I2C]: embedded_hal_async::i2c::I2c
///
/// <!-- TODO: I2C example -->
///
/// For GPIO digital input and output, use the [`MCP2221::gpio_take_pins`] method, and
/// convert the [`GpPin`] objects into [`Input`] or [`Output`] types, which implement
/// the appropriate traits from [`embedded_hal::digital`].
///
/// [`GpPin`]: crate::gpio::GpPin
/// [`Input`]: crate::gpio::Input
/// [`Output`]: crate::gpio::Output
///
/// <!-- TODO: GPIO example -->
///
/// # Overview
///
/// <!-- TODO -->.
///
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
    /// `!Sync` anyway. See [`Self::gpio_take_pins`] for the only place it is used.
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

    ////////////////////////////////////////////////////////////////////////////////
    // USB report exchange with the MCP2221
    ////////////////////////////////////////////////////////////////////////////////

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
    // MCP2221 general commands
    ////////////////////////////////////////////////////////////////////////////////

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

    /// Read the status of the MCP2221.
    ///
    /// The returned structure includes the current status of the I2C engine, and the
    /// hardware and firmware revision numbers.
    ///
    /// It includes the raw ADC channel readings, but you should prefer to use
    /// [`MCP2221::analog_read`].
    ///
    /// It also contains the edge-triggered interrupt flag, but you should prefer to
    /// use [`MCP2221::interrupt_detected`].
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

    ////////////////////////////////////////////////////////////////////////////////
    // I2C
    ////////////////////////////////////////////////////////////////////////////////

    /// Set the speed of the I2C bus.
    ///
    /// The MCP2221 can communicate at speeds from just below 47 kbit/s to 400 kbit/s,
    /// though not every rate can be achieved exactly due to the way the speed is
    /// set in the device.
    ///
    /// # Errors
    ///
    /// An [`Error::I2cCouldNotChangeSpeed`] may be returned if an ongoing I2C transfer
    /// prevented the device from setting the bus speed.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.1 of the datasheet for the underlying Status/Set Parameters
    /// HID command.
    pub fn i2c_set_bus_speed(&self, speed: I2cSpeed) -> Result<(), Error> {
        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        // When this value is put in this field, the device will take the next command
        // field and interpret it as the system clock divider that will give the
        // I2C/SMBus communication clock.
        uc.set_data_byte(3, 0x20);
        uc.set_data_byte(4, speed.to_clock_divider());
        let read_buffer = self.transfer(&uc)?.expect("Always has response buffer.");
        match read_buffer[3] {
            0x20 => Ok(()),
            0x21 => Err(Error::I2cCouldNotChangeSpeed),
            _ => unreachable!("Invalid response from MCP2221 for I2C speed set command."),
        }
    }

    /// Cancel current I2C transfer.
    ///
    /// If the I2C engine is busy, the driver will attempt to cancel the current
    /// transfer.
    ///
    /// Microchip's Android Java driver for the MCP2221 describes this command
    /// as "forc\[ing\] a STOP condition into the SCL/SDA lines".
    ///
    /// <div class="warning">
    ///
    /// The driver will not instruct the MCP2221 to cancel a transfer if the I2C engine
    /// appears idle, as doing so appears to put the I2C engine into a busy state.
    ///
    /// </div>
    ///
    /// # Datasheet
    ///
    /// See section 3.11 of the datasheet for the underlying Status/Set Parameters
    /// HID command.
    pub fn i2c_cancel_transfer(&self) -> Result<I2cCancelTransferResponse, Error> {
        // Only issue the cancellation command if the I2Cengine is busy to avoid it
        // _becoming_ busy by issuing the cancellation.
        if self.status()?.i2c.communication_state.is_idle() {
            return Ok(I2cCancelTransferResponse::NoTransfer);
        }

        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        uc.set_data_byte(2, 0x10);
        let read_buffer = self.transfer(&uc)?.expect("Always has response buffer.");

        match read_buffer[2] {
            0x10 => Ok(I2cCancelTransferResponse::MarkedForCancellation),
            0x11 => Ok(I2cCancelTransferResponse::NoTransfer),
            0x00 => Ok(I2cCancelTransferResponse::Done),
            code => unreachable!("Unknown code received from I2C cancel attempt {code}"),
        }
    }

    /// Read data from an I2C target.
    ///
    /// The address must be the 7-bit address, not an 8-bit read or write address.
    /// 10-bit addresses are not currently supported.
    ///
    /// Zero-length transfers are not accepted, as they can cause the target to lock up
    /// the I2C bus if it holds SDA low for the first bit.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.8 for the underlying I2C Read Data HID command.
    pub fn i2c_read(&self, seven_bit_address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        self._i2c_read(seven_bit_address, read_buffer, ReadType::Normal)
    }

    /// Read data from an I2C target with a repeated START condition.
    ///
    /// It is unclear from the datasheet how this differs from the standard I2C read HID
    /// command or how it should be used. Formally, a repeated-START in I2C is just a
    /// START condition when the previous transfer has not been terminated by a STOP
    /// condition, so this _should_ be the same as issuing a normal read.
    ///
    /// In this library, this method is called after writing with no stop, in order to
    /// perform a write-read (ST, address-w, data-out, SR, address-r, data-in, SP). It
    /// is exposed to users for completeness with no guarantees or suggestions about its
    /// usage.
    ///
    /// In general, it appears that this exposes some of the internal details of the
    /// MCP2221 I2C engine, but without the explanation needed to make sense of it.
    ///
    /// The restrictions from [`MCP2221::i2c_read`] also apply: the address provided
    /// must be the 7-bit address, and zero-length transfers are not supported.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.9 for the underlying I2C Read Data Repeated-START HID command.
    pub fn i2c_read_repeated_start(
        &self,
        seven_bit_address: u8,
        read_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self._i2c_read(seven_bit_address, read_buffer, ReadType::RepeatedStart)
    }

    /// Cancel the I2C transfer if the target device did not acknowledge its address.
    ///
    /// This is an internal helper method.
    fn i2c_bail_for_nack(&self) -> Result<(), Error> {
        match self.status()?.i2c.target_acknowledged_address {
            true => Ok(()),
            false => {
                self.i2c_cancel_transfer()?;
                Err(Error::I2cAddressNack)
            }
        }
    }

    /// Perform an I2C read of the type specified.
    ///
    /// The I2C HID commands only differ in their command bytes (and their semantics),
    /// so this is the underlying implementation for the two i2c_read_ functions.
    ///
    /// Starts with an underscore purely so users can have the obvious `i2c_read()`
    /// for doing normal reads.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.8 (normal read) and 3.1.9 (repeated start).
    fn _i2c_read(
        &self,
        seven_bit_address: u8,
        read_buffer: &mut [u8],
        read_type: ReadType,
    ) -> Result<(), Error> {
        // Don't attempt to read if the transfer length is 0, as attempting a zero-length
        // read will lock up the bus if the peripheral pulls SDA low trying to transmit.
        // Note the MCP2221 will happily let you do that!
        if read_buffer.is_empty() {
            return Err(Error::I2cTransferEmpty);
        }
        let Ok(tx_len): Result<u16, _> = read_buffer.len().try_into() else {
            return Err(Error::I2cTransferTooLong);
        };

        use crate::i2c::I2cAddressing;
        let mut read_command = UsbReport::new(read_type.into());
        let [tx_len_low, tx_len_high] = tx_len.to_le_bytes();
        read_command.set_data_byte(1, tx_len_low);
        read_command.set_data_byte(2, tx_len_high);
        read_command.set_data_byte(3, seven_bit_address.into_read_address());
        self.transfer(&read_command)?;
        // Clean up if the target did not acknowledge.
        self.i2c_bail_for_nack()?;
        self.i2c_read_get_data(read_buffer)
    }

    /// Read I2C target data back from the MCP2221.
    ///
    /// This is called after requesting a read to get the actual data.
    ///
    /// It appears in the datasheet as a separate HID command but it is really just
    /// an implementation detail due to the way the MCP2221 does reads. Writes don't
    /// have the same issue-the-command/perform-the-command split.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.10 for the underlying I2c Read Data - Get I2C data command.
    fn i2c_read_get_data(&self, read_buffer: &mut [u8]) -> Result<(), Error> {
        // Retries are necessary because it is likely the host will request data from
        // the MCP2221 faster than it can process it, in which case it returns the
        // "error reading from the engine" 0x41 code.
        //
        // It may be necessary to turn this into a configuration option for the driver
        // if users encounter failed reads.
        const MAX_RETRIES: u8 = 20;
        // With my Pico test setup, for a read of 65,535 bytes, a 2ms delay upon
        // failing to read from the I2C engine appears to yield the shortest overall
        // time (compared to 1ms and 3ms). Adding a 1ms delay after each successful
        // read (hoping to avoid the 2ms failure delay) just seems to increase the
        // overall time taken.
        const RETRY_DELAY: Duration = Duration::from_millis(2);
        // Sanity check that the driver never tries to read zero bytes.
        if read_buffer.is_empty() {
            return Err(Error::I2cTransferEmpty);
        }

        let transfer_length = read_buffer.len();
        let mut read_so_far: usize = 0;

        let get_command = UsbReport::new(McpCommand::I2cGetData);
        let mut retries = MAX_RETRIES;

        while read_so_far < transfer_length {
            match self.transfer(&get_command) {
                Ok(Some(buffer)) => {
                    // Reset the number of retries.
                    retries = MAX_RETRIES;
                    if buffer[3] == 127 {
                        // Error occurred when reading the data, try again.
                        // This shouldn't occur when the status at byte 1 is OK.
                        continue;
                    }
                    let data_length = buffer[3] as usize;
                    read_buffer[read_so_far..read_so_far + data_length]
                        .copy_from_slice(&buffer[4..4 + data_length]);
                    read_so_far += data_length;
                }
                Ok(None) => unreachable!("Get Data always returns a buffer."),
                Err(Error::I2cEngineReadError) if retries > 0 => {
                    // Error reading target data from the I2C engine, just try again
                    // after a short delay.
                    retries -= 1;
                    std::thread::sleep(RETRY_DELAY);
                    continue;
                }
                e @ Err(_) => {
                    // Out of retries, just return the error.
                    e?;
                }
            }
        }

        Ok(())
    }

    /// Write data to an I2C target.
    ///
    /// The address must be the 7-bit address, not an 8-bit read or write address.
    /// 10-bit addresses are not currently supported.
    ///
    /// The given `data` buffer cannot be more than 65,535 bytes long, as this is the
    /// maximum transfer size supported by the MCP2221.
    ///
    /// Zero-length writes are not accepted, use [`MCP2221::i2c_check_address`] instead
    /// if you are trying to scan the bus.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.5 for the underlying I2C Write Data HID command.
    pub fn i2c_write(&self, seven_bit_address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        self._i2c_write(seven_bit_address, write_buffer, WriteType::Normal)
    }

    /// Write data to an I2C target with a repeated START condition.
    ///
    /// It is unclear from the datasheet how this differs from the standard I2C write
    /// HID command or how it should be used. Formally, a repeated-START in I2C is just a
    /// START condition when the previous transfer has not been terminated by a STOP
    /// condition, so this _should_ be the same as issuing a normal write.
    ///
    /// This method is not actually used in the implementation of this library, and is
    /// only exposed because the MCP2221 exposes it as a separate USB HID command. No
    /// guarantees or suggestions are made about its usage. (But if you discover
    /// something that might help others, please [file an issue].)
    ///
    /// [file an issue]: https://github.com/robjwells/mcp2221-hal/issues
    ///
    /// The restrictions from [`MCP2221::i2c_write`] also apply: the address provided
    /// must be the 7-bit address, and zero-length transfers are not supported.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.6 for the underlying I2C Write Data Repeated-START HID command.
    pub fn i2c_write_repeated_start(
        &self,
        seven_bit_address: u8,
        write_buffer: &[u8],
    ) -> Result<(), Error> {
        self._i2c_write(seven_bit_address, write_buffer, WriteType::RepeatedStart)
    }

    /// Write data to an I2C target without a final STOP condition.
    ///
    /// In this library, this is used to implement I2C write-read (ST, address-w,
    /// data-out, SR, address-r, data-in, SP) before a read with repeated-START.
    /// It is exposed to the user for completeness with no guarantees or suggestions
    /// about its usage outside of this scenario.
    ///
    /// The restrictions from [`MCP2221::i2c_write`] still apply: the address provided
    /// must be the 7-bit address, and zero-length transfers are not supported.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.7 for the underlying I2C Write Data NO STOP HID command.
    pub fn i2c_write_no_stop(
        &self,
        seven_bit_address: u8,
        write_buffer: &[u8],
    ) -> Result<(), Error> {
        self._i2c_write(seven_bit_address, write_buffer, WriteType::NoStop)
    }

    /// Perform an I2C write of the type specified.
    ///
    /// The I2C HID commands only differ in their command bytes (and their semantics),
    /// so this is the underlying implementation for the three i2c_write_ functions.
    ///
    /// Starts with an underscore purely so users can have the obvious `i2c_write()`
    /// for doing normal write.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.5 (normal write), 3.1.6 (repeated-START), and 3.1.7 (no STOP).
    fn _i2c_write(
        &self,
        seven_bit_address: u8,
        write_buffer: &[u8],
        write_type: WriteType,
    ) -> Result<(), Error> {
        let Ok([tx_len_low, tx_len_high]) = u16::try_from(write_buffer.len()).map(u16::to_le_bytes)
        else {
            return Err(Error::I2cTransferTooLong);
        };
        if write_buffer.is_empty() {
            return Err(Error::I2cTransferEmpty);
        }

        use crate::i2c::I2cAddressing;
        let mut command = UsbReport::new(write_type.into());
        command.set_data_byte(1, tx_len_low);
        command.set_data_byte(2, tx_len_high);
        command.set_data_byte(3, seven_bit_address.into_write_address());

        // Retries appear less necessary than when reading, but the host can still
        // attempt to write faster than the MCP2221 can accept them, so we set a retry
        // limit to avoid a potentially infinite loop.
        const MAX_RETRIES: u8 = 20;
        const RETRY_DELAY: Duration = Duration::from_millis(2);

        for (idx, chunk) in write_buffer.chunks(60).enumerate() {
            let mut retries = MAX_RETRIES;
            loop {
                command.write_buffer[4..4 + chunk.len()].copy_from_slice(chunk);
                match self.transfer(&command) {
                    Ok(_) => {
                        if idx == 0 {
                            // Check for address acknowledgement, otherwise clean up,
                            // but only for the first chunk. MCP2221 will happily
                            // take writes for a missing target.
                            self.i2c_bail_for_nack()?;
                        }
                        break;
                    }
                    Err(Error::I2cEngineBusy) if retries > 0 => {
                        retries -= 1;
                        std::thread::sleep(RETRY_DELAY);
                        continue;
                    }
                    e @ Err(_) => e?,
                };
            }
        }

        Ok(())
    }

    /// Perform an I2C write-read to the given target address.
    ///
    /// First the contents of `write_buffer` are written to the target, without a final
    /// STOP condition. Then a repeated-START is issued and enough bytes are read from
    /// the target to fill `read_buffer`.
    ///
    /// # Datasheet
    ///
    /// See sections 3.1.7 (write, no STOP) and 3.1.9 (read, repeated START) for the
    /// underlying HID commands.
    pub fn i2c_write_read(
        &self,
        seven_bit_address: u8,
        write_buffer: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.i2c_write_no_stop(seven_bit_address, write_buffer)?;
        self.i2c_read_repeated_start(seven_bit_address, read_buffer)
    }

    /// Check if an I2C target acknowledges the given address.
    ///
    /// This is a special-case of an I2C write, where no bytes are actually written
    /// to the target. It is a separate function because of the need to potentially
    /// also cancel the I2C transfer after the write if the device does not respond.
    pub fn i2c_check_address(&self, seven_bit_address: u8) -> Result<bool, Error> {
        use crate::i2c::I2cAddressing;
        let mut command = UsbReport::new(McpCommand::I2cWriteData);
        command.set_data_byte(3, seven_bit_address.into_write_address());

        const MAX_RETRIES: u8 = 20;
        const RETRY_DELAY: Duration = Duration::from_millis(2);
        for _ in 0..MAX_RETRIES {
            match self.transfer(&command) {
                Ok(_) => {
                    // The write was submitted, doesn't mean the target is there.
                    let address_ack = self.status()?.i2c.target_acknowledged_address;
                    // Clean up any incomplete transfer.
                    self.i2c_cancel_transfer()?;
                    return Ok(address_ack);
                }
                Err(Error::I2cEngineBusy) => {
                    // Just try again after a delay.
                    std::thread::sleep(RETRY_DELAY);
                    continue;
                }
                e @ Err(_) => {
                    // Some other error that we weren't expecting.
                    e?;
                }
            };
        }
        // If we get here we ran out of retries without checking the address.
        Err(Error::I2cOperationFailed)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // GPIO
    ////////////////////////////////////////////////////////////////////////////////

    /// Take the four GP pin structs for individual GPIO operation.
    ///
    /// This can only be done once, and will return `None` afterwards.
    pub fn gpio_take_pins(&self) -> Option<Pins> {
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
    /// current direction or set output level of a GPIO pin.
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
    /// requires an additional read command to the device to work around a firmware bug
    /// that resets analog voltage references (see the note in section 1.8.1.1 of the
    /// datasheet from revision D onwards).
    ///
    /// Note that this method will not change the mode of GP pins that are not set for
    /// GPIO operation. That must be done first by setting the pin mode, either
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
    /// [`MCP2221::sram_read_settings`]) will not reflect the current GPIO pin direction
    /// and output level. This appears to be a bug in the MCP2221 firmware and is not
    /// documented in the datasheet.
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

    ////////////////////////////////////////////////////////////////////////////////
    // Analog
    ////////////////////////////////////////////////////////////////////////////////

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
        let (chip_settings, gp) = self.sram_read_settings()?;
        let reading = AdcReading {
            vref: chip_settings.adc_reference,
            gp1: gp.gp1_mode.is_analog_input().then_some(raw.ch1),
            gp2: gp.gp2_mode.is_analog_input().then_some(raw.ch2),
            gp3: gp.gp3_mode.is_analog_input().then_some(raw.ch3),
        };
        Ok(reading)
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
        self.sram_write_settings(SramSettingsChanges::new().with_dac_value(value))?;
        Ok(())
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
    /// See the [`analog`](crate::analog) module for more information about the
    /// voltage references.
    ///
    /// # Datasheet
    ///
    /// See section 1.8.2 for information about the 10-bit ADC, section 1.8.1.1 for
    /// details about Vrm, and section 3.1.13 for the underlying Set SRAM Settings
    /// HID command.
    pub fn analog_set_input_reference(&self, source: VoltageReference) -> Result<(), Error> {
        self.sram_write_settings(SramSettingsChanges::new().with_adc_reference(source))?;
        Ok(())
    }

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
    /// See the [`analog`](crate::analog) module for more information about the
    /// voltage references.
    ///
    /// # Datasheet
    ///
    /// See section 1.8.3 for information about the 5-bit DAC, section 1.8.1.1 for
    /// details about Vrm (with the caveat listed above), and section 3.1.13 for
    /// the underlying Set SRAM Settings HID command.
    pub fn analog_set_output_reference(&self, source: VoltageReference) -> Result<(), Error> {
        self.sram_write_settings(SramSettingsChanges::new().with_dac_reference(source))?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Interrupt flag management
    ////////////////////////////////////////////////////////////////////////////////

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
    /// The flag indicates that an edge has been detected on GP1, when GP1 is in
    /// interrupt-detection mode. The interrupt detection conditions (positive edge,
    /// negative edge, or both) are not changed.
    ///
    /// ## Datasheet
    ///
    /// There is very little about interrupt detection in the MCP2221 datasheet. See
    /// byte 6 in table 3-36 for descriptions of the related settings. See section
    /// 1.10 for a very brief general overview.
    pub fn interrupt_clear(&self) -> Result<(), Error> {
        self.sram_write_settings(
            SramSettingsChanges::new()
                .with_interrupt_settings(InterruptSettingsChanges::clear_flag(true)),
        )
    }

    ////////////////////////////////////////////////////////////////////////////////
    // SRAM settings
    ////////////////////////////////////////////////////////////////////////////////

    /// Retrieve the chip and GP pin settings stored in SRAM.
    ///
    /// <div class="warning">
    ///
    /// Do not rely on the returned settings accurately reflecting the current
    /// state of the MCP2221. Some commands will (in practice) change these settings
    /// without those changes being shown when subsequently reading the SRAM.
    ///
    /// - GPIO pin direction and level after using the Set GPIO Output Values HID
    ///   command (implemented by [`MCP2221::gpio_write`]).
    /// - Vrm reference level set to "off" after setting GP pin settings via the Set
    ///   SRAM Settings HID command (implemented by [`MCP2221::sram_write_settings`])
    ///   _without_ also explicitly setting the Vrm level. See the note in section
    ///   1.8.1.1 of the datasheet, as well as the documentation for
    ///   [`SramSettingsChanges::with_gp_modes`].
    ///
    /// </div>
    ///
    /// ## Datasheet
    ///
    /// See section 3.1.14 of the datasheet for details about the underlying Get SRAM
    /// Settings HID command, and section 1.4 for information about the configuration
    /// process at power-up.
    pub fn sram_read_settings(&self) -> Result<(ChipSettings, GpSettings), Error> {
        let command = UsbReport::new(McpCommand::GetSRAMSettings);
        let buf = self
            .transfer(&command)?
            .expect("Always has response buffer.");
        Ok((
            ChipSettings::from_buffer(&buf),
            GpSettings::try_from_sram_buffer(&buf)?,
        ))
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
    pub fn sram_write_settings(&self, settings: &SramSettingsChanges) -> Result<(), Error> {
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
        let (chip_settings, _) = self.sram_read_settings()?;
        self.sram_write_settings(SramSettingsChanges::new().with_gp_modes(
            gp_settings,
            Some(chip_settings.dac_reference),
            Some(chip_settings.adc_reference),
        ))
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Flash settings
    ////////////////////////////////////////////////////////////////////////////////

    /// Read chip settings from flash memory.
    ///
    /// The chip settings collect several important but unrelated configuration options.
    /// See the fields of [`ChipSettings`] and table 3-15 of the datasheet for details
    /// about each one.
    ///
    /// Settings in flash memory take effect on power-up.
    ///
    /// # Datasheet
    ///
    /// See section 1.4 for information on the configuration process. See section
    /// 3.1.2 for the underlying Read Flash Data HID command and table 3-5 for the
    /// relevant subcommand.
    pub fn flash_read_chip_settings(&self) -> Result<ChipSettings, Error> {
        let command = McpCommand::ReadFlashData(FlashDataSubCode::ChipSettings);
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        Ok(ChipSettings::from_buffer(&buf))
    }

    /// Write chip settings to flash memory.
    ///
    /// The chip settings collect several important but unrelated configuration options.
    /// See the fields of [`ChipSettings`] and table 3-12 of the datasheet for details
    /// about each one.
    ///
    /// Settings stored in the flash memory of the MCP2221 take effect when the device
    /// is powered-up.
    ///
    /// # Datasheet
    ///
    /// See section 1.4 for information on the configuration process. See section
    /// 3.1.3 for the underlying Write Flash Data HID command and table 3-12 for the
    /// relevant subcommand.
    pub fn flash_write_chip_settings(&self, cs: ChipSettings) -> Result<(), Error> {
        let mut command =
            UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::ChipSettings));
        cs.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Read GP pin settings from flash memory.
    ///
    /// These are the initial settings for the GP pins when the device is powered-up.
    ///
    /// Settings in flash memory take effect on power-up.
    ///
    /// # Datasheet
    ///
    /// See section 1.4 for information on the configuration process. See section
    /// 3.1.2 for the underlying Read Flash Data HID command and table 3-6 for the
    /// relevant subcommand.
    pub fn flash_read_gp_settings(&self) -> Result<GpSettings, Error> {
        let command = McpCommand::ReadFlashData(FlashDataSubCode::GPSettings);
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        GpSettings::try_from_flash_buffer(&buf)
    }

    /// Write GP pin settings to flash memory.
    ///
    /// This can be used to set appropriate defaults for the pin functions for your
    /// use case, and further (temporary) changes can be made at run time via the
    /// methods [`MCP2221::sram_write_settings`] (for changing pin functions) and
    /// [`MCP2221::gpio_write`] (for changing digital output direction and level).
    ///
    /// Settings stored in the flash memory of the MCP2221 take effect when the device
    /// is powered-up.
    ///
    /// # Datasheet
    ///
    /// See section 1.4 for information on the configuration process. See section
    /// 3.1.3 for the underlying Write Flash Data HID command and table 3-13 for
    /// the relevant subcommand.
    pub fn flash_write_gp_settings(&self, gp: GpSettings) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::GPSettings));
        gp.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Read the USB manufacturer descriptor string from flash memory.
    ///
    /// The manufacturer descriptor string is used to identify a device to a
    /// USB host.
    ///
    /// If you wish to read the USB vendor ID number (VID), see
    /// [`MCP2221::flash_read_chip_settings`].
    ///
    /// # Datasheet
    ///
    /// See section 3.1.2 for the underlying Read Flash Data HID command, and
    /// table 3-7 for the relevant subcommand.
    pub fn usb_manufacturer(&self) -> Result<DeviceString, Error> {
        let command = McpCommand::ReadFlashData(FlashDataSubCode::UsbManufacturerDescriptor);
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        DeviceString::try_from_buffer(&buf)
    }

    /// Change the USB manufacturer descriptor string.
    ///
    /// The manufacturer descriptor string is used to identify a device to a
    /// USB host. This setting is stored in flash, so the MCP2221 will have to
    /// be reset (and re-enumerate) for the change to take effect.
    ///
    /// The manufacturer string can be at most 30 UTF-16 code points long.
    ///
    /// If you wish to change the USB vendor ID number (VID), see
    /// [`MCP2221::flash_write_chip_settings`].
    ///
    /// # Datasheet
    ///
    /// See section 3.1.3 for the underlying Write Flash Data HID command, and
    /// table 3-14 for the relevant subcommand.
    pub fn usb_change_manufacturer(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbManufacturerDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Read the USB product descriptor string from flash memory.
    ///
    /// The product descriptor string is used to identify a device to a USB host.
    ///
    /// If you wish to read the USB product ID number (VID), see
    /// [`MCP2221::flash_read_chip_settings`].
    ///
    /// # Datasheet
    ///
    /// See section 3.1.2 for the underlying Read Flash Data HID command, and
    /// table 3-8 for the relevant subcommand.
    pub fn usb_product(&self) -> Result<DeviceString, Error> {
        let command = McpCommand::ReadFlashData(FlashDataSubCode::UsbProductDescriptor);
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        DeviceString::try_from_buffer(&buf)
    }

    /// Change the USB product descriptor string.
    ///
    /// The product descriptor string is used to identify a device to a USB host.
    /// This setting is stored in flash, so the MCP2221 will have to be reset
    /// (and re-enumerate) for the change to take effect.
    ///
    /// The product string can be at most 30 UTF-16 code points long.
    ///
    /// If you wish to change the USB product ID number (PID), see
    /// [`MCP2221::flash_write_chip_settings`].
    ///
    /// # Datasheet
    ///
    /// See section 3.1.3 for the underlying Write Flash Data HID command, and
    /// table 3-15 for the relevant subcommand.
    pub fn usb_change_product(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbProductDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Read the USB serial number descriptor string from flash memory.
    ///
    /// The serial number descriptor string is used to identify a device to a USB host.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.2 for the underlying Read Flash Data HID command, and
    /// table 3-9 for the relevant subcommand.
    pub fn usb_serial_number(&self) -> Result<DeviceString, Error> {
        let command = McpCommand::ReadFlashData(FlashDataSubCode::UsbSerialNumberDescriptor);
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        DeviceString::try_from_buffer(&buf)
    }

    /// Change the USB serial number descriptor string.
    ///
    /// The serial number descriptor string is used to identify a device to a USB host.
    /// This setting is stored in flash, so the MCP2221 will have to be reset (and
    /// re-enumerate) for the change to take effect.
    ///
    /// The serial number string can be at most 30 UTF-16 code points long.
    ///
    /// # Datasheet
    ///
    /// See section 3.1.3 for the underlying Write Flash Data HID command, and
    /// table 3-16 for the relevant subcommand.
    pub fn usb_change_serial_number(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbSerialNumberDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(&command)?;
        Ok(())
    }

    /// Read chip factory serial number.
    ///
    /// Read the factory-set device serial number. For the MCP2221A, this appears to
    /// always be "01234567" in ASCII. It cannot be changed.
    ///
    /// This function uses [`String::from_utf8_lossy`], so if you read a serial number
    /// with Unicode replacement characters, your device has an unexpected, non-ASCII
    /// factory serial number and you should [file an issue].
    ///
    /// [file an issue]: https://github.com/robjwells/mcp2221-hal/issues
    ///
    /// # Datasheet
    ///
    /// See section 3.1.2 for the underlying Read Flash Data HID command, and
    /// table 3-10 for the relevant subcommand.
    pub fn factory_serial_number(&self) -> Result<String, Error> {
        let command = McpCommand::ReadChipFactorySerialNumber;
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        let length = buf[2] as usize;
        let serial_number_portion = &buf[4..(4 + length)];
        Ok(String::from_utf8_lossy(serial_number_portion).into())
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
