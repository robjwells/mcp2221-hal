use crate::analog::{AdcCommand, DacCommand};
use crate::i2c::I2cCommand;
use crate::pins::PinsCommand;
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
    /// Alter DAC settings.
    #[command(subcommand)]
    Dac(DacCommand),
    /// Read or configure analog input.
    #[command(subcommand)]
    Adc(AdcCommand),
    /// Read or configure the GP pins.
    #[command(subcommand)]
    Pins(PinsCommand),
    /// Perform I2C functions
    #[command(subcommand)]
    I2c(I2cCommand),
    /// Reset the MCP2221.
    Reset,
    /// Read the current device status.
    Status,
    /// Read or write device settings.
    #[command(subcommand)]
    Settings(SettingsCommand),
    /// Print the USB HID device info.
    Usb,
}
