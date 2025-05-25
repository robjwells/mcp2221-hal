use super::MCP2221;
use crate::commands::{FlashDataSubCode, McpCommand, UsbReport};
use crate::common::DeviceString;
use crate::gpio::GpSettings;
use crate::{ChipSettings, Error};

impl MCP2221 {
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
        self.transfer(&command)?;
        Ok(())
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
    pub fn read_usb_manufacturer(&self) -> Result<DeviceString, Error> {
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
    pub fn change_usb_manufacturer(&self, s: &DeviceString) -> Result<(), Error> {
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
    pub fn read_usb_product(&self) -> Result<DeviceString, Error> {
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
    pub fn change_usb_product(&self, s: &DeviceString) -> Result<(), Error> {
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
    pub fn read_usb_serial_number(&self) -> Result<DeviceString, Error> {
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
    pub fn change_usb_serial_number_descriptor(&self, s: &DeviceString) -> Result<(), Error> {
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
    pub fn read_factory_serial_number(&self) -> Result<String, Error> {
        let command = McpCommand::ReadChipFactorySerialNumber;
        let buf = self
            .transfer(&UsbReport::new(command))?
            .expect("Always has response buffer.");
        let length = buf[2] as usize;
        let serial_number_portion = &buf[4..(4 + length)];
        Ok(String::from_utf8_lossy(serial_number_portion).into())
    }
}
