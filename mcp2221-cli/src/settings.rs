use clap::{Parser, ValueEnum};
use mcp2221_hal::settings::DeviceString;

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
