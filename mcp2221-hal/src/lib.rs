//! MCP2221 driver crate.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod analog;
mod commands;
pub mod common;
mod driver;
mod error;
pub mod flash_data;
pub mod gpio;
pub mod i2c;
pub mod security;
pub mod sram;
pub mod status;

pub use driver::MCP2221;
pub use error::Error;
