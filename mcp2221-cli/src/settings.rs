use clap::{Parser, ValueEnum};
use mcp2221_hal::{MCP2221, settings::DeviceString};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum SettingsType {
    Sram,
    Flash,
}

#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum SettingsCommand {
    /// Read device settings.
    Read {
        /// Whether to read the current SRAM settings or the power-on settings
        /// written to the MCP2221's flash memory.
        which: SettingsType,
    },
    /// Change device settings.
    #[command(subcommand, flatten_help = true)]
    Write(SettingsWriteCommand),
}

#[derive(Debug, Parser)]
pub(crate) enum SettingsWriteCommand {
    /// Set the USB manufacturer descriptor string in flash.
    ///
    /// The device must be reset and re-enumerated for this to be shown.
    Manufacturer {
        /// Must be 60 bytes or fewer when encoded as UTF-16.
        string: DeviceString,
    },
    /// Set the USB product descriptor string in flash.
    ///
    /// The device must be reset and re-enumerated for this to be shown.
    Product {
        /// Must be 60 bytes or fewer when encoded as UTF-16.
        string: DeviceString,
    },
}

pub(crate) fn action(
    device: &MCP2221,
    command: SettingsCommand,
) -> Result<(), mcp2221_hal::Error> {
    match command {
        SettingsCommand::Read { which } => match which {
            SettingsType::Flash => print_all_flash_data(device)?,
            SettingsType::Sram => println!("{:#?}", device.sram_read_settings()?),
        },
        SettingsCommand::Write(write_command) => match write_command {
            SettingsWriteCommand::Manufacturer { string } => {
                device.usb_change_manufacturer(&string)?
            }
            SettingsWriteCommand::Product { string } => device.usb_change_product(&string)?,
        },
    }
    Ok(())
}

fn print_all_flash_data(device: &mcp2221_hal::MCP2221) -> Result<(), mcp2221_hal::Error> {
    println!("{:#?}", device.flash_read_chip_settings()?);
    println!("{:#?}", device.flash_read_gp_settings()?);
    println!(r#"USB Manufacturer:  "{}""#, device.usb_manufacturer()?);
    println!(r#"USB Product:       "{}""#, device.usb_product()?);
    println!(r#"USB Serial Number: "{}""#, device.usb_serial_number()?);
    println!(
        r#"Factory serial:    "{}""#,
        device.factory_serial_number()?
    );
    Ok(())
}
