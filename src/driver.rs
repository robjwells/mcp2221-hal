use crate::commands::{FlashDataSubCode, McpCommand, UsbReport};
use crate::error::Error;
use crate::flash_data::{ChipSettings, FlashData, GpSettings};
use crate::status::Status;
use crate::types::{CancelI2cTransferResponse, DeviceString, I2cSpeed};

const MICROCHIP_VENDOR_ID: u16 = 1240;
const MCP2221A_PRODUCT_ID: u16 = 221;

pub struct MCP2221 {
    inner: hidapi::HidDevice,
}

/// USB device functionality
impl MCP2221 {
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_pid(MICROCHIP_VENDOR_ID, MCP2221A_PRODUCT_ID)
    }

    pub fn open_with_vid_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = hidapi::HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self { inner: device })
    }

    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
}

/// HID Commands
impl MCP2221 {
    pub fn status(&mut self) -> Result<Status, Error> {
        let buf = self.transfer(UsbReport::new(McpCommand::StatusSetParameters))?;
        Ok(Status::from_buffer(&buf))
    }

    /// Cancel current I2C transfer.
    ///
    /// The device will cancel the current I2C transfer and will attempt to free the I2C
    /// bus. See table 3-1 in section 3.1.1 of the datasheet.
    pub fn cancel_i2c_transfer(&mut self) -> Result<CancelI2cTransferResponse, Error> {
        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        uc.write_buffer[2] = 0x10;
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
    /// Err([Error::I2cTransferInProgress]) if the speed could not be set due to an
    /// ongoing I2C transfer.
    pub fn set_i2c_bus_speed(&mut self, speed: I2cSpeed) -> Result<(), Error> {
        let mut uc = UsbReport::new(McpCommand::StatusSetParameters);
        // When this value is put in this field, the device will take the next command
        // field and interpret it as the system clock divider that will give the
        // I2C/SMBus communication clock.
        uc.write_buffer[3] = 0x20;
        uc.write_buffer[4] = speed.to_clock_divider();
        let read_buffer = self.transfer(uc)?;
        match read_buffer[3] {
            0x20 => Ok(()),
            0x21 => Err(Error::I2cTransferInProgress),
            _ => unreachable!("Invalid response from MCP2221 for I2C speed set command."),
        }
    }

    /// Read all the settings stored in flash memory.
    pub fn read_flash_data(&mut self) -> Result<FlashData, Error> {
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

    /// Update the chip settings stored in flash memory.
    ///
    /// **NOTE** that the chip security setting is not written to the device. This is to
    /// avoid permanently locking the device. Currently, this will always attempt to set
    /// the device to "unlocked" mode. If you have previously password-locked the
    /// MCP2221A via other means, you will likely encounter an error.
    pub fn write_chip_settings_to_flash(&mut self, cs: ChipSettings) -> Result<(), Error> {
        let mut command =
            UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::ChipSettings));
        cs.apply_to_write_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the GP pin settings stored in flash memory.
    pub fn write_gp_settings_to_flash(&mut self, gps: GpSettings) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(FlashDataSubCode::GPSettings));
        gps.apply_to_write_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB manufacturer string descriptor used during USB enumeration.
    pub fn write_usb_manufacturer_descriptor(&mut self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbManufacturerDescriptor,
        ));
        s.apply_to_write_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB product string descriptor used during USB enumeration.
    pub fn write_usb_product_descriptor(&mut self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbProductDescriptor,
        ));
        s.apply_to_write_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Update the USB serial number descriptor string used during USB enumeration.
    pub fn write_usb_serial_number_descriptor(&mut self, s: &DeviceString) -> Result<(), Error> {
        let mut command = UsbReport::new(McpCommand::WriteFlashData(
            FlashDataSubCode::UsbSerialNumberDescriptor,
        ));
        s.apply_to_write_buffer(&mut command.write_buffer);
        self.transfer(command)?;
        Ok(())
    }

    /// Write the given command to the MCP and read the 64-byte response.
    fn transfer(&self, command: UsbReport) -> Result<[u8; 64], Error> {
        let out_command_byte = command.write_buffer[0];
        let written = self.inner.write(&command.report_bytes())?;

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
            // Write Flash Data extra error codes
            (0xB1, 0x02) => Err(Error::CommandNotSupported),
            (0xB1, 0x03) => Err(Error::CommandNotAllowed),
            (_, code) => Err(Error::CommandFailed(code)),
        }
    }
}
