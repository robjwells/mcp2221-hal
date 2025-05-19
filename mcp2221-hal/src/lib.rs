//! MCP2221 driver crate.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod analog;
mod chip_settings;
mod commands;
pub mod common;
mod driver;
mod error;
mod flash_data;
pub mod gpio;
pub mod i2c;
mod security;
mod sram;
mod status;

pub use chip_settings::ChipSettings;
pub use driver::MCP2221;
pub use error::Error;
pub use flash_data::FlashData;
pub use security::ChipConfigurationSecurity;
pub use sram::{ChangeSramSettings, SramSettings};
pub use status::Status;
