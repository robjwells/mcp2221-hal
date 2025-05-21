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
