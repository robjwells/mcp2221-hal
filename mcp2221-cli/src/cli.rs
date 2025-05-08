use crate::analog::DacCommand;
use crate::settings::SettingsCommand;
use crate::util;

use clap::Parser;

/// MCP2221 CLI
#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    /// Device vendor ID in hexadecimal
    #[arg(short, long = "vid-hex", default_value = "4D8", value_parser = util::from_hex)]
    pub(crate) vid: u16,
    /// Device product ID in hexadecimal
    #[arg(short, long = "pid-hex", default_value = "DD", value_parser = util::from_hex)]
    pub(crate) pid: u16,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Parser)]
pub(crate) enum Commands {
    /// Alter DAC settings in SRAM.
    #[command(subcommand)]
    Dac(DacCommand),
    /// Read the current device status.
    Status,
    /// Read or write device settings.
    #[command(subcommand)]
    Settings(SettingsCommand),
    /// Print the USB HID device info.
    Usb,
    /// Reset the MCP2221.
    Reset,
}
