use clap::{ArgAction, Parser};
use mcp2221_hal::{
    MCP2221,
    settings::{ChipSettings, DeviceString},
};

/// Read or configure USB device information.
#[derive(Debug, Clone, Parser)]
pub(crate) enum UsbCommand {
    /// Read USB information from the host.
    Info,
    /// Write USB-related settings to the MCP2221.
    #[command(subcommand)]
    Set(UsbSetCommand),
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct UsbInfo<'a> {
    pub(crate) manufacturer: Option<&'a str>,
    pub(crate) product: Option<&'a str>,
    pub(crate) serial_number: Option<&'a str>,
    pub(crate) vendor_id: String,
    pub(crate) product_id: String,
    pub(crate) path: String,
}

impl<'a> From<&'a hidapi::DeviceInfo> for UsbInfo<'a> {
    fn from(info: &'a hidapi::DeviceInfo) -> Self {
        Self {
            manufacturer: info.manufacturer_string(),
            product: info.product_string().to_owned(),
            serial_number: info.serial_number(),
            vendor_id: format!("{:#X}", info.vendor_id()),
            product_id: format!("{:#X}", info.product_id()),
            path: info.path().to_string_lossy().to_string(),
        }
    }
}

/// Change device USB settings.
///
/// These are related to USB enumeration and are stored in flash,
/// so the device will need to be reset for them to take effect.
///
/// See the i2c command to change the I2C bus speed, the pins command
/// to change GP pin settings, and the adc and dac commands to change
/// voltage references or analog output.
#[derive(Debug, Clone, Parser)]
pub(crate) enum UsbSetCommand {
    /// Set the USB manufacturer descriptor string.
    Manufacturer {
        /// Must be 60 bytes or fewer when encoded as UTF-16.
        string: DeviceString,
    },
    /// Set the USB product descriptor string.
    Product {
        /// Must be 60 bytes or fewer when encoded as UTF-16.
        string: DeviceString,
    },
    /// Set the USB serial number descriptor string.
    ///
    /// This can be combined with enabling CDC serial number enumeration
    /// to give the serial port a convenient name.
    Serial {
        /// Must be 60 bytes or fewer when encoded as UTF-16.
        string: DeviceString,
    },
    /// Set CDC USB serial number enumeration.
    ///
    /// This will present the USB serial number during enumeration of the CDC (USB
    /// serial converter) device, which will give the serial port a stable name.
    CdcEnumeration {
        #[arg(action = ArgAction::Set)]
        on: bool,
    },
    /// Set the USB vendor ID (VID).
    ///
    /// If you are changing the VID from the default, you will have to use the
    /// --vid argument when using the CLI.
    ///
    /// The default VID is 0x4D8.
    Vid {
        #[arg(value_parser = crate::util::u16_from_hex)]
        vid: u16,
    },
    /// Set the USB product ID (PID) to the given hex value.
    ///
    /// If you are changing the PID from the default, you will have to use the
    /// --pid argument when using the CLI.
    ///
    /// The default PID is 0xDD.
    Pid {
        #[arg(value_parser = crate::util::u16_from_hex)]
        pid: u16,
    },
}

pub(crate) fn action(device: &MCP2221, command: UsbCommand) -> Result<(), mcp2221_hal::Error> {
    match command {
        UsbCommand::Set(write_command) => match write_command {
            UsbSetCommand::Manufacturer { string } => device.usb_change_manufacturer(&string)?,
            UsbSetCommand::Product { string } => device.usb_change_product(&string)?,
            UsbSetCommand::Serial { string } => device.usb_change_serial_number(&string)?,
            UsbSetCommand::Vid { vid } => modify_flash_chip_settings(device, |cs| {
                cs.usb_vendor_id = vid;
            })?,

            UsbSetCommand::Pid { pid } => modify_flash_chip_settings(device, |cs| {
                cs.usb_product_id = pid;
            })?,

            UsbSetCommand::CdcEnumeration { on } => modify_flash_chip_settings(device, |cs| {
                cs.cdc_serial_number_enumeration_enabled = on;
            })?,
        },

        UsbCommand::Info => print_info(device)?,
    }
    Ok(())
}

fn print_info(device: &MCP2221) -> Result<(), mcp2221_hal::Error> {
    println!("{:#?}", UsbInfo::from(&device.usb_device_info()?));
    Ok(())
}

fn modify_flash_chip_settings(
    device: &MCP2221,
    f: impl Fn(&mut ChipSettings),
) -> Result<(), mcp2221_hal::Error> {
    let mut chip_settings = device.flash_read_chip_settings()?;
    f(&mut chip_settings);
    device.flash_write_chip_settings(chip_settings)?;
    Ok(())
}
