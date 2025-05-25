#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod analog;
mod chip_settings;
mod commands;
pub mod common;
mod driver;
mod error;
pub mod gpio;
pub mod i2c;
mod security;
mod sram;
pub mod status;

pub use chip_settings::ChipSettings;
pub use driver::MCP2221;
pub use error::Error;
pub use security::ChipConfigurationSecurity;
pub use sram::{ChangeSramSettings, SramSettings};

/// Notable constant values.
pub mod constants {
    pub(crate) const MAX_I2C_TRANSFER: usize = u16::MAX as usize;
    pub(crate) const MAX_I2C_TRANSFER_PLUS_1: usize = MAX_I2C_TRANSFER + 1;
    pub(crate) const COMMAND_SUCCESS: u8 = 0x00;

    /// Microchip USB vendor ID.
    pub const MICROCHIP_VID: u16 = 1240;
    /// MCP2221 USB product ID.
    pub const MCP2221_PID: u16 = 221;
}
