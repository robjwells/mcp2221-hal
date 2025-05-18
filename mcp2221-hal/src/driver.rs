use hidapi::{HidApi, HidDevice};

use crate::analog::{AdcReading, VoltageReference};
use crate::commands::{FlashDataSubCode, McpCommand, UsbReport};
use crate::common::DeviceString;
use crate::error::Error;
use crate::flash_data::{ChipSettings, FlashData};
use crate::gpio::{ChangeGpioValues, GpSettings, GpioValues};
use crate::i2c::{CancelI2cTransferResponse, I2cSpeed};
use crate::sram::{ChangeSramSettings, SramSettings};
use crate::status::Status;

const MICROCHIP_VENDOR_ID: u16 = 1240;
const MCP2221A_PRODUCT_ID: u16 = 221;

/// Driver for the MCP2221.
pub struct MCP2221 {
    inner: HidDevice,
}

/// # USB device functionality
impl MCP2221 {
    /// Open the first USB device found with the default vendor and product ID.
    ///
    /// The default VID is 1240 (0x4D8) and PID 221 (0xDD) for both the original
    /// MCP2221 and the (more common) MCP2221A.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_and_pid(MICROCHIP_VENDOR_ID, MCP2221A_PRODUCT_ID)
    }

    /// Open the first USB device found with the given venor and product ID.
    ///
    /// Use this function if you have changed the USB VID or PID of your MCP2221.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn open_with_vid_and_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self { inner: device })
    }

    /// Get the USB HID device information from the host's USB interface.
    ///
    /// This is a thin wrapper around [`HidDevice::get_device_info`].
    ///
    /// # Errors
    ///
    /// An error will be returned if the device information cannot be returned
    /// from the underlying USB interface.
    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
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
        let buf = self.transfer(UsbReport::new(McpCommand::StatusSetParameters))?;
        Ok(Status::from_buffer(&buf))
    }

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
    /// # Datasheet
    ///
    /// See section 3.11 of the datasheet for the underlying Status/Set Parameters
    /// HID command.
    pub fn i2c_cancel_transfer(&self) -> Result<CancelI2cTransferResponse, Error> {
        // Only issue the cancellation command if the I2C engine is busy to avoid it
        // _becoming_ busy by issuing the cancellation..
        if self.status()?.i2c_engine_is_idle {
            return Ok(CancelI2cTransferResponse::NoTransfer);
        }

        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        uc.set_data_byte(2, 0x10);
        let read_buffer = self.transfer(uc)?;

        match read_buffer[2] {
            0x10 => Ok(CancelI2cTransferResponse::MarkedForCancellation),
            0x11 => Ok(CancelI2cTransferResponse::NoTransfer),
            _ => unreachable!("Invalid value from MCP2221 for transfer cancellation."),
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
    /// An [`Error::I2cTransferInProgress`] may be returned if an ongoing I2C transfer
    /// prevented the device from setting the bus speed.
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
        let read_buffer = self.transfer(uc)?;
        match read_buffer[3] {
            0x20 => Ok(()),
            0x21 => Err(Error::I2cTransferInProgress),
            _ => unreachable!("Invalid response from MCP2221 for I2C speed set command."),
        }
    }

    /// Read settings stored in flash memory.
    ///
    /// Settings stored in the flash memory of the MCP2221 take effect when the device
    /// is powered-up.
    ///
    /// # Datasheet
    ///
    /// See section 1.4 for information on the configuration process. See section
    /// 3.1.2 for the underlying Read Flash Data HID command. For convenience, this
    /// method executes all subcommands to read all settings stored in flash.
    pub fn flash_read_settings(&self) -> Result<FlashData, Error> {
        use FlashDataSubCode::*;
        use McpCommand::ReadFlashData;

        let chip_settings = self.transfer(UsbReport::new(ReadFlashData(ChipSettings)))?;
        let gp_settings = self.transfer(UsbReport::new(ReadFlashData(GPSettings)))?;
        let usb_mfr = self.transfer(UsbReport::new(ReadFlashData(UsbManufacturerDescriptor)))?;
        let usb_product = self.transfer(UsbReport::new(ReadFlashData(UsbProductDescriptor)))?;
        let usb_serial = self.transfer(UsbReport::new(ReadFlashData(UsbSerialNumberDescriptor)))?;
        let chip_factory_serial =
            self.transfer(UsbReport::new(ReadFlashData(ChipFactorySerialNumber)))?;

        Ok(FlashData::from_buffers(
            &chip_settings,
            &gp_settings,
            &usb_mfr,
            &usb_product,
            &usb_serial,
            &chip_factory_serial,
        ))
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
    /// <div class="warning">
    ///
    /// The chip security setting is not written to the device, to avoid inadvertently
    /// locking the device. This method will attempt to set the device to unprotected
    /// mode. If you have previously restricted the MCP2221 via other means, you will
    /// likely encounter an error.
    ///
    /// </div>
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
        self.transfer(command)?;
        Ok(())
    }

    /// Write GP pin settings to flash memory.
    ///
    /// This can be used to set appropriate defaults for the pin functions for your
    /// use case, and further (temporary) changes can be made at run time via the
    /// methods [`MCP2221::set_sram_settings`] (for changing pin functions) and
    /// [`MCP2221::set_gpio_values`] (for changing digital output direction and level).
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
        self.transfer(command)?;
        Ok(())
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
    pub fn change_usb_manufacturer(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbManufacturerDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
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
    pub fn change_usb_product(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbProductDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
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
    pub fn write_usb_serial_number_descriptor(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbSerialNumberDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Retrieve the chip and GP pin settings stored in SRAM.
    ///
    /// The settings read from SRAM match the structure of the [`ChipSettings`] stored
    /// in flash, with the addition of the [`GpSettings`].
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
        let buf = self.transfer(command)?;
        Ok(SramSettings::from_buffer(&buf))
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
        self.transfer(command)?;
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
    /// Note that the DAC output is not linear from 0V to the reference and (at least
    /// with 3.3V supply) does not reach the reference voltage.
    ///
    /// This setting is not persisted across reset. See [`MCP2221::flash_write_chip_settings`]
    /// to set the DAC to output a particular value at power-on.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DacValueOutOfRange`] if the value is too large (maximum 31
    /// for the 5-bit DAC) or if an error occurred communicating with the MCP2221.
    ///
    /// # Datasheet
    ///
    /// See section 1.8.3 for information about the 5-bit DAC, and section 3.1.13 for
    /// the underlying Set SRAM Settings HID command.
    pub fn analog_write(&self, value: u8) -> Result<(), Error> {
        // TODO: Values above 31 should just be clamped and a warning emitted.
        if value > 31 {
            return Err(Error::DacValueOutOfRange);
        }

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
    /// # Datasheet
    ///
    /// See section 1.8.2 for information about the 10-bit ADC and section 3.1.1 for
    /// the underlying Status/Set Parameters HID command.
    pub fn analog_read(&self) -> Result<AdcReading, Error> {
        let (ch1, ch2, ch3) = self.status()?.adc_values;
        let SramSettings {
            adc_reference: vref,
            gp_settings: gp,
            ..
        } = self.sram_read_settings()?;
        let reading = AdcReading {
            vref,
            gp1: gp.gp1.is_adc().then_some(ch1),
            gp2: gp.gp2.is_adc().then_some(ch2),
            gp3: gp.gp3.is_adc().then_some(ch3),
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
        let buf = self.transfer(UsbReport::new(McpCommand::GetGpioValues))?;
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
    pub fn gpio_write(&self, changes: &ChangeGpioValues) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::SetGpioOutputValues);
        changes.apply_to_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
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

        self.transfer(command)?;
        Ok(())
    }

    /// Write the given command to the MCP and read the 64-byte response.
    fn transfer(&self, command: UsbReport) -> Result<[u8; 64], Error> {
        let out_command_byte = command.write_buffer[0];
        let written = self.inner.write(&command.report_bytes())?;
        if out_command_byte == 0x70 {
            // TODO: Fix this. Manual checking for reset command. This is horrible.
            // Also faking the response buffer (no response from reset) is gross.
            return Ok([0u8; 64]);
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

        // Check success code.
        match (out_command_byte, read_buffer[1]) {
            (_, 0x00) => Ok(read_buffer),
            // Read Flash Data extra error code
            (0xB0, 0x01) => Err(Error::CommandNotSupported),
            // Write Flash Data extra error codes
            (0xB1, 0x02) => Err(Error::CommandNotSupported),
            (0xB1, 0x03) => Err(Error::CommandNotAllowed),
            (_, code) => Err(Error::CommandFailed(code)),
        }
    }
}
