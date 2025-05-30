use clap::{Parser, ValueEnum};
use mcp2221_hal::MCP2221;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum SettingsType {
    Sram,
    Flash,
}

/// Read device settings.
///
/// Specify whether you want to read the settings from SRAM, which affect the current
/// behaviour of the device, or the settings in flash memory that affect the initial
/// behaviour of the device on power-up.
///
/// Note that the GP pin settings read from SRAM may not reflect the current status
/// of GPIO pins. This appears to be a bug in the MCP2221 firmware.
#[derive(Debug, Parser)]
pub(crate) struct SettingsCommand {
    /// Current SRAM settings or initial settings in flash memory.
    which: SettingsType,
}

pub(crate) fn action(device: &MCP2221, command: SettingsCommand) -> Result<(), mcp2221_hal::Error> {
    match command.which {
        SettingsType::Flash => print_all_flash_data(device)?,
        SettingsType::Sram => print_sram_settings(device)?,
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

fn print_sram_settings(device: &MCP2221) -> Result<(), mcp2221_hal::Error> {
    let (cs, gp) = device.sram_read_settings()?;
    println!("{cs:#?}\n{gp:#?}");
    Ok(())
}
