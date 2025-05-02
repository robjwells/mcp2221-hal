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

pub struct MCP2221 {
    inner: hidapi::HidDevice,
    write_buffer: [u8; 65],
    read_buffer: [u8; 64],
}

/// USB device functionality
impl MCP2221 {
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_pid(MICROCHIP_VENDOR_ID, MCP2221A_PRODUCT_ID)
    }

    pub fn open_with_vid_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = hidapi::HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self {
            inner: device,
            write_buffer: [0u8; 65],
            read_buffer: [0u8; 64],
        })
    }

    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
}

/// HID Commands
impl MCP2221 {
    pub fn status(&mut self) -> Result<Status, Error> {
        self.set_command(Command::StatusSetParameters);
        let _ = self._try_transfer()?;
        Ok(Status::from_buffer(&self.read_buffer))
    }

    /// Read all the settings stored in flash memory.
    pub fn read_flash_data(&mut self) -> Result<FlashData, Error> {
        use Command::ReadFlashData;
        use ReadFlashDataSubCode::*;

        self.set_command(ReadFlashData(ChipSettings));
        let _ = self._try_transfer()?;
        let chip_settings = self.read_buffer;

        self.set_command(ReadFlashData(GPSettings));
        let _ = self._try_transfer()?;
        let gp_settings = self.read_buffer;

        self.set_command(ReadFlashData(UsbManufacturerDescriptor));
        let _ = self._try_transfer()?;
        let usb_mfr = self.read_buffer;

        self.set_command(ReadFlashData(UsbProductDescriptor));
        let _ = self._try_transfer()?;
        let usb_product = self.read_buffer;

        self.set_command(ReadFlashData(UsbSerialNumberDescriptor));
        let _ = self._try_transfer()?;
        let usb_serial = self.read_buffer;

        self.set_command(ReadFlashData(ChipFactorySerialNumber));
        let _ = self._try_transfer()?;
        let chip_factory_serial = self.read_buffer;

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
    fn set_command(&mut self, c: Command) {
        use Command::*;
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
        self.write_buffer[1] = command_byte;
        if let Some(sub_command_byte) = sub_command_byte {
            self.write_buffer[2] = sub_command_byte;
        }
    }

    /// Write the current output buffer state to the MCP and read from it.
    fn _try_transfer(&mut self) -> Result<(usize, usize), Error> {
        let sent_command = self.write_buffer[1];
        let written = self.inner.write(&self.write_buffer)?;
        let read = self.inner.read(&mut self.read_buffer)?;

        // Check length written and read.
        assert_eq!(written, 65, "Didn't write entire write buffer.");
        assert_eq!(read, 64, "Didn't read full report.");

        // Zero write buffer to prevent pollution of future commands.
        // TODO: Is this the best way to do it? It doesn't happen if
        // .write() or .read() return an error!
        self.write_buffer = [0; 65];

        // Check command-code echo.
        if self.read_buffer[0] != sent_command {
            return Err(Error::MismatchedCommandCodeEcho {
                sent: self.write_buffer[1],
                received: self.read_buffer[0],
            });
        }

        // Check success code.
        match self.read_buffer[1] {
            0x00 => Ok((written, read)),
            code => Err(Error::CommandFailed(code)),
        }
    }
}

enum Command {
    /// Poll for the status of the device.
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
