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
    /// The default VID is 1240 (0x4D8) and PID 221 (0xDD).
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_and_pid(MICROCHIP_VENDOR_ID, MCP2221A_PRODUCT_ID)
    }

    /// Open the first USB device found with the given venor and product ID.
    ///
    /// Use this function if you have changed the USB VID or PID of your MCP2221.
    pub fn open_with_vid_and_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self { inner: device })
    }

    /// Get the USB HID device information from the host's USB interface.
    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
}

/// # HID Commands
impl MCP2221 {
    /// Read the status of the MCP2221.
    ///
    /// This is read via the Status/Set Parameters command. See section 3.11 of the
    /// datasheet for details.
    pub fn status(&self) -> Result<Status, Error> {
        let buf = self.transfer(UsbReport::new(McpCommand::StatusSetParameters))?;
        Ok(Status::from_buffer(&buf))
    }

    /// Cancel current I2C transfer.
    ///
    /// The device will cancel the current I2C transfer and will attempt to free the I2C
    /// bus. See table 3-1 in section 3.1.1 of the datasheet.
    ///
    /// <div class="warning">
    ///
    /// Note that issuing the cancellation command to the MCP2221 while the I2C
    /// engine is already idle appears to put the engine into a busy state. The driver
    /// avoids this by checking the engine status and _not issuing the cancellation_
    /// if the engine is already idle.
    ///
    /// </div>
    pub fn cancel_i2c_transfer(&self) -> Result<CancelI2cTransferResponse, Error> {
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

    /// Set the baud rate of the I2C bus.
    ///
    /// Returns `Ok(())` when the speed was set successfully and
    /// `Err(`[`Error::I2cTransferInProgress`]`)` if the speed could not be
    /// set due to an ongoing I2C transfer.
    pub fn set_i2c_bus_speed(&self, speed: I2cSpeed) -> Result<(), Error> {
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
    /// These settings take effect on power-up. See section 1.4 for information on the
    /// configuration process. See section 3.1.2 for the Read Flash Data command.
    pub fn read_flash_data(&self) -> Result<FlashData, Error> {
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

    /// Update settings stored in flash memory.
    ///
    /// These settings take effect on power-up. See section 1.4 for information on the
    /// configuration process. See section 3.1.3 for the Write Flash Data command.
    ///
    /// <div class="warning">
    /// The chip security setting is not written to the device. This is to avoid
    /// permanently locking the device. Currently, this will always attempt to set the
    /// device to "unlocked" mode. If you have previously password-locked the MCP2221
    /// via other means, you will likely encounter an error.
    /// </div>
    pub fn write_chip_settings_to_flash(&self, cs: ChipSettings) -> Result<(), Error> {
        let mut command =
            UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::ChipSettings));
        cs.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the GP pin settings stored in flash memory.
    ///
    /// This changes the power-up setting and will not affect the active SRAM settings.
    /// To have this change take effect, reset the device or apply the same changes to
    /// the SRAM settings.
    pub fn write_gp_settings_to_flash(&self, gp: GpSettings) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::GPSettings));
        gp.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB manufacturer string descriptor used during USB enumeration.
    pub fn write_usb_manufacturer_descriptor(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbManufacturerDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB product string descriptor used during USB enumeration.
    pub fn write_usb_product_descriptor(&self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbProductDescriptor,
        ));
        s.apply_to_flash_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB serial number descriptor string used during USB enumeration.
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
    /// <div class="warning">
    ///
    /// Do not rely on the returned [`SramSettings`] accurately reflecting
    /// the current state of the MCP2221. Some commands will (in practice) change these
    /// settings without those changes being shown when subsequently reading the SRAM.
    ///
    /// - GPIO pin direction and level after using the Set GPIO Output Values command
    ///   (implemented by [`MCP2221::set_gpio_values`]).
    /// - Vrm reference level set to off after setting GP pin settings via `Set SRAM Settings`
    ///   (implemented by [`MCP2221::set_sram_settings`]) _without_ also explicitly setting
    ///   the Vrm level. See the note in section 1.8.1.1 of the datasheet, as well
    ///   as the documentation for [`ChangeSramSettings::with_gp_modes`].
    ///
    /// </div>
    ///
    /// See section 3.1.13 of the datasheet for details about the underlying `Get SRAM
    /// Settings` command, and section 1.4 for information about the configuration
    /// process at power-up.
    pub fn get_sram_settings(&self) -> Result<SramSettings, Error> {
        let command = UsbReport::new(McpCommand::GetSRAMSettings);
        let buf = self.transfer(command)?;
        Ok(SramSettings::from_buffer(&buf))
    }

    /// Change run-time chip and GP pin settings.
    ///
    /// If you only need to change GPIO pin direction or output level, use the
    /// [`MCP2221::set_gpio_values()`] method.
    ///
    /// <div class="warning">
    /// Changing the GP pin settings without also setting the ADC and DAC voltage
    /// references will result in them being set to Vrm in "off" mode. See the note
    /// in section 1.8.1.1 of the datasheet.
    /// </div>
    ///
    /// Changes made in this way (to SRAM) do not persist across device reset.
    ///
    /// See section 3.1.13 of the datasheet for details about the underlying command.
    pub fn set_sram_settings(&self, settings: &ChangeSramSettings) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::SetSRAMSettings);
        settings.apply_to_sram_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Configure the DAC voltage reference in SRAM.
    ///
    /// This setting is not persisted across reset.
    pub fn configure_dac_source(&self, source: VoltageReference) -> Result<(), Error> {
        self.set_sram_settings(ChangeSramSettings::new().with_dac_reference(source))?;
        Ok(())
    }

    /// Set the DAC output value in SRAM.
    ///
    /// This setting is not persisted across reset.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DacValueOutOfRange`] if the value is too large (maximum 31
    /// for the 5-bit DAC) or if an error occurred communicating with the MCP2221.
    pub fn set_dac_output_value(&self, value: u8) -> Result<(), Error> {
        // TODO: Should this be an error or should the value be clamped?
        if value > 31 {
            return Err(Error::DacValueOutOfRange);
        }

        self.set_sram_settings(ChangeSramSettings::new().with_dac_value(value))?;
        Ok(())
    }

    /// Read the current values of the three-channel ADC.
    pub fn read_adc(&self) -> Result<AdcReading, Error> {
        let (ch1, ch2, ch3) = self.status()?.adc_values;
        let SramSettings {
            adc_reference: vref,
            gp_settings: gp,
            ..
        } = self.get_sram_settings()?;
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
    /// This setting is not persisted across reset.
    pub fn configure_adc_source(&self, source: VoltageReference) -> Result<(), Error> {
        self.set_sram_settings(ChangeSramSettings::new().with_adc_reference(source))?;
        Ok(())
    }

    /// Get GPIO pin direction and current logic levels.
    ///
    /// Only pins that are configured for GPIO operation are present in the returned
    /// [`GpioValues`] struct. The logic level listed for output pins is the currently
    /// set output, and for input pins it is the voltage level read on that pin.
    pub fn get_gpio_values(&self) -> Result<GpioValues, Error> {
        let buf = self.transfer(UsbReport::new(McpCommand::GetGpioValues))?;
        Ok(GpioValues::from_buffer(&buf))
    }

    /// Change GPIO pins' output direction and output logic level.
    ///
    /// <div class="warning">
    ///
    /// This method will not change the mode of GP pins that are not configured for
    /// GPIO operation. That must be done by changing the mode settings in SRAM via
    /// [`MCP2221::set_sram_settings`], or in flash via
    /// [`MCP2221::write_gp_settings_to_flash`] and resetting the device.
    ///
    /// </div>
    ///
    /// See section 3.1.11 of the datasheet for the underlying Set GPIO Output Values
    /// command.
    pub fn set_gpio_values(&self, changes: &ChangeGpioValues) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::SetGpioOutputValues);
        changes.apply_to_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Reset the MCP2221.
    ///
    /// Resetting the chip causes the device to re-enumerate, so you will need
    /// to create a new driver struct afterwards.
    pub fn reset_chip(self) -> Result<(), Error> {
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
