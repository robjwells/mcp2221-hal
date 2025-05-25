//! I2C-related driver public methods and helpers.
use std::time::Duration;

use super::MCP2221;
use crate::Error;
use crate::commands::{McpCommand, UsbReport};
use crate::i2c::{CancelI2cTransferResponse, I2cSpeed, ReadType, WriteType};

/// I2C-related commands.
impl MCP2221 {
    /// Cancel current I2C transfer.
    ///
    /// If the I2C engine is busy, the driver will attempt to cancel the current
    /// transfer.
    ///
    /// <div class="warning">
    ///
    /// The driver will not instruct the MCP2221 to cancel a transfer if the I2C
    /// engine appears idle, as doing so appears to then put the I2C engine into
    /// a persistent busy state.
    ///
    /// </div>
    ///
    /// Microchip's Android Java driver for the MCP2221 describes this command
    /// as "forc\[ing\] a STOP condition into the SCL/SDA lines".
    ///
    /// # Datasheet
    ///
    /// See section 3.11 of the datasheet for the underlying Status/Set Parameters
    /// HID command.
    pub fn i2c_cancel_transfer(&self) -> Result<CancelI2cTransferResponse, Error> {
        // Only issue the cancellation command if the I2Cengine is busy to avoid it
        // _becoming_ busy by issuing the cancellation.
        if self.status()?.i2c.communication_state.is_idle() {
            return Ok(CancelI2cTransferResponse::NoTransfer);
        }

        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        uc.set_data_byte(2, 0x10);
        let read_buffer = self.transfer(&uc)?.expect("Always has response buffer.");

        match read_buffer[2] {
            0x10 => Ok(CancelI2cTransferResponse::MarkedForCancellation),
            0x11 => Ok(CancelI2cTransferResponse::NoTransfer),
            0x00 => Ok(CancelI2cTransferResponse::Done),
            code => unreachable!("Unknown code received from I2C cancel attempt {code}"),
        }
    }

    /// Set the speed of the I2C bus.
    ///
    /// # Limitations
    ///
    /// Only "standard-mode" of 100 kbps and "fast-mode" of 400 kbps are supported by
    /// this crate. The MCP2221 itself can support speeds from about 47 kbps (limited
    /// by the internal divider size) up to a maximum of 400kbps.
    ///
    /// Please [open an issue] if you need a bus speed other than 100k or 400k.
    ///
    /// [open an issue]: https://github.com/robjwells/mcp2221-hal/issues
    ///
    /// # Errors
    ///
    /// An [`Error::I2cTransferPreventedSpeedChange`] may be returned if an ongoing
    /// I2C transfer prevented the device from setting the bus speed.
    ///
    /// # Datasheet
    ///
    /// See section 3.11 of the datasheet for the underlying Status/Set Parameters
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
            0x21 => Err(Error::I2cTransferPreventedSpeedChange),
            _ => unreachable!("Invalid response from MCP2221 for I2C speed set command."),
        }
    }

    /// Read data from an I2C target.
    ///
    /// The address must be the 7-bit address, not an 8-bit read or write address.
    /// 10-bit addresses are not supported by the MCP2221.
    ///
    /// Zero-length transfers are not accepted, as they can cause the target to lock
    /// up the I2C bus if it holds SDA low for the first bit.
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
    /// perform a write-read (ST, address-w, data-out, SR, address-r, data-in, SP).
    /// It is exposed to users for completeness with no guarantees or suggestions about
    /// its usage.
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

    /// Perform an I2C read of the type specified.
    ///
    /// The I2C HID commands only differ in their command bytes (and their semantics),
    /// so this is the underlying implementation for the two i2c_read_\* functions.
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

    /// Cancel the I2C transfer if the target device did not acknowledge its address.
    fn i2c_bail_for_nack(&self) -> Result<(), Error> {
        match self.status()?.i2c.ack_received {
            true => Ok(()),
            false => {
                self.i2c_cancel_transfer()?;
                Err(Error::I2cAddressNack)
            }
        }
    }

    /// Read I2C target data back from the MCP2221.
    ///
    /// This is called after requesting a read to get the actual data.
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
    /// 10-bit addresses are not supported by the MCP2221.
    ///
    /// The given `data` buffer cannot be more than 65,535 bytes long, as this is the
    /// maximum transfer size supported by the MCP2221.
    ///
    /// Zero-length writes are not accepted, you want [`MCP2221::i2c_check_address`]
    /// instead.
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
    /// so this is the underlying implementation for the two i2c_read_\* functions.
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
    /// First the provided data buffer is written to the target, without a final STOP
    /// condition. Then a repeated-START is issued and `read_length` bytes are read
    /// from the target and returned.
    ///
    /// # Datasheet
    ///
    /// See sections 3.1.7 and 3.1.9 for the underlying HID commands.
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
                    let address_ack = self.status()?.i2c.ack_received;
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
}
