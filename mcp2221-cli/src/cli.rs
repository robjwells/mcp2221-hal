use crate::analog::{AdcCommand, DacCommand};
use crate::i2c::I2cCommand;
use crate::pins::PinsCommand;
use crate::settings::SettingsCommand;
use crate::usb::UsbCommand;
use crate::util;

use clap::Parser;

/// CLI for the MCP2221 USB to I2C and GPIO converter
///
/// This exposes a subset of the functionality of the Microchip MCP2221 via the
/// command line.
///
/// You can put the GP pins into any of their possible modes, as well as read
/// from and write to them as GPIO (digital) inputs or outputs. Analog input and
/// output is handled via the adc and dac commands, respectively.
///
/// I2C transfers can be made up to the maximum length of 65,535 bytes. Only
/// 7-bit I2C addresses are currently supported. The CLI supports writes, reads,
/// and write-reads (where no Stop condition occurs between a write and a read).
///
/// All settings can be read from the device, but the CLI can currently change
/// only a subset relating to the GP pins (pins), analog IO (adc/dac configure),
/// I2C bus speed (i2c speed), and USB device information (usb set).
#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    /// Device vendor ID in hexadecimal
    #[arg(short, long = "vid", default_value = "0x4D8", value_parser = util::u16_from_hex)]
    pub(crate) vid: u16,
    /// Device product ID in hexadecimal
    #[arg(short, long = "pid", default_value = "0xDD", value_parser = util::u16_from_hex)]
    pub(crate) pid: u16,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Parser)]
pub(crate) enum Commands {
    /// Perform I2C transfers
    #[command(subcommand)]
    I2c(I2cCommand),
    /// Read or configure the GP pins.
    #[command(subcommand)]
    Pins(PinsCommand),
    #[command(subcommand)]
    Adc(AdcCommand),
    #[command(subcommand)]
    Dac(DacCommand),
    Settings(SettingsCommand),
    /// Read the current device status.
    Status,
    #[command(subcommand)]
    Usb(UsbCommand),
    /// Reset the MCP2221.
    Reset,
}
