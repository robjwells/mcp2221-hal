#![allow(dead_code)]
#![allow(unused_variables)]

pub mod error;
pub mod flash_data;
pub mod status;

use error::Error;
use flash_data::FlashData;
use status::Status;

const MICROCHIP_VENDOR_ID: u16 = 1240;
const MCP2221A_PRODUCT_ID: u16 = 221;

struct UsbCommand {
    pub(crate) write_buffer: [u8; 64],
}

impl UsbCommand {
    fn report_bytes(&self) -> [u8; 65] {
        let mut out = [0u8; 65];
        out[1..65].copy_from_slice(&self.write_buffer);
        out
    }
}

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
        let buf = self.transfer(self.new_command(McpCommand::StatusSetParameters))?;
        Ok(Status::from_buffer(&buf))
    }

    /// Cancel current I2C transfer.
    ///
    /// The device will cancel the current I2C transfer and will attempt to free the I2C
    /// bus. See table 3-1 in section 3.1.1 of the datasheet.
    pub fn cancel_i2c_transfer(&mut self) -> Result<CancelI2cTransferResponse, Error> {
        let mut uc = self.new_command(McpCommand::StatusSetParameters);
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
        let mut uc = self.new_command(McpCommand::StatusSetParameters);
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
        use McpCommand::ReadFlashData;
        use ReadFlashDataSubCode::*;

        let chip_settings = self.transfer(self.new_command(ReadFlashData(ChipSettings)))?;

        let gp_settings = self.transfer(self.new_command(ReadFlashData(GPSettings)))?;

        let usb_mfr = self.transfer(self.new_command(ReadFlashData(UsbManufacturerDescriptor)))?;

        let usb_product = self.transfer(self.new_command(ReadFlashData(UsbProductDescriptor)))?;

        let usb_serial =
            self.transfer(self.new_command(ReadFlashData(UsbSerialNumberDescriptor)))?;

        let chip_factory_serial =
            self.transfer(self.new_command(ReadFlashData(ChipFactorySerialNumber)))?;

        Ok(FlashData::from_buffers(
            &chip_settings,
            &gp_settings,
            &usb_mfr,
            &usb_product,
            &usb_serial,
            &chip_factory_serial,
        ))
    }

    /// Write the appropriate command byte to write_buffer[1].
    ///
    /// write_buffer starts with the dummy/default report number, so the
    /// actual MCP command is at write_buffer[1..=65].
    fn new_command(&self, c: McpCommand) -> UsbCommand {
        let mut buf = [0u8; 64];
        use McpCommand::*;
        use ReadFlashDataSubCode::*;
        let (command_byte, sub_command_byte): (u8, Option<u8>) = match c {
            StatusSetParameters => (0x10, None),
            ReadFlashData(ChipSettings) => (0xB0, Some(0x00)),
            ReadFlashData(GPSettings) => (0xB0, Some(0x01)),
            ReadFlashData(UsbManufacturerDescriptor) => (0xB0, Some(0x02)),
            ReadFlashData(UsbProductDescriptor) => (0xB0, Some(0x03)),
            ReadFlashData(UsbSerialNumberDescriptor) => (0xB0, Some(0x04)),
            ReadFlashData(ChipFactorySerialNumber) => (0xB0, Some(0x05)),
        };
        buf[0] = command_byte;
        if let Some(sub_command_byte) = sub_command_byte {
            buf[1] = sub_command_byte;
        }
        UsbCommand { write_buffer: buf }
    }

    /// Write the given command to the MCP and read the 64-byte response.
    fn transfer(&self, command: UsbCommand) -> Result<[u8; 64], Error> {
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
        match read_buffer[1] {
            0x00 => Ok(read_buffer),
            code => Err(Error::CommandFailed(code)),
        }
    }
}

#[derive(Debug)]
pub enum CancelI2cTransferResponse {
    MarkedForCancellation,
    NoTransfer,
}

enum McpCommand {
    /// Poll for the status of the device, cancel an I2C transfer,
    /// or set the I2C bus speed.
    ///
    /// See section 3.1.1.
    StatusSetParameters,
    /// Read various important data structures and strings stored in the flash
    /// memory on the MCP2221A.
    ///
    /// See section 3.1.2 of the datasheet.
    ///
    /// Many of these settings determine start-up values that can be changed
    /// at runtime (the MCP2221A copies them into SRAM). See section 1.4.3.
    ReadFlashData(ReadFlashDataSubCode),
}

/// Read various settings stored in the flash memory.
enum ReadFlashDataSubCode {
    ChipSettings,
    // GP pin power-up settings.
    GPSettings,
    /// USB manufacturer string descriptor used during USB enumeration.
    UsbManufacturerDescriptor,
    /// USB product string descriptor used during USB enumeration.
    UsbProductDescriptor,
    /// USB serial number string descriptor used during USB enumeration.
    UsbSerialNumberDescriptor,
    /// Factory-set serial number. Always "01234567".
    ChipFactorySerialNumber,
}

#[allow(non_camel_case_types)]
pub enum I2cSpeed {
    /// I2c bus speed of 400kbps ("Fast-mode")
    Fast_400kbps,
    /// I2c bus speed of 100kbps ("Standard-mode")
    Standard_100kbps,
}

impl I2cSpeed {
    /// Convert the speed mode into a clock divider suitable for the
    /// STATUS/SET PARAMETERS command.
    fn to_clock_divider(&self) -> u8 {
        // 12 MHz internal clock.
        const MCP_CLOCK: u32 = 12_000_000;

        // I don't know why the division is followed by -3.
        // But it appears in the Microchip Linux C driver as well as the Adafruit Blinka
        // Python driver. The mcp2221-rs library uses -2. None of them have a comment
        // explaining why. The mcp2221a Go library has a comment also expressing
        // surprise at the `-3` part.
        const STANDARD_DIVIDER: u8 = (MCP_CLOCK / 100_000 - 3) as u8;
        const FAST_DIVIDER: u8 = (MCP_CLOCK / 400_000 - 3) as u8;

        match self {
            I2cSpeed::Fast_400kbps => FAST_DIVIDER,
            I2cSpeed::Standard_100kbps => STANDARD_DIVIDER,
        }
    }
}

#[derive(Debug)]
/// GPIO pin level setting.
pub enum LogicLevel {
    High,
    Low,
}

impl From<bool> for LogicLevel {
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

#[derive(Debug)]
/// GPIO pin direction.
pub enum GpioDirection {
    Input,
    Output,
}

impl From<bool> for GpioDirection {
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}
