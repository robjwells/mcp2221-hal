#![allow(dead_code)]
#![allow(unused_variables)]

pub mod analog;
mod commands;
pub mod common;
mod driver;
pub mod error;
pub mod flash_data;
pub mod gpio;
pub mod sram;
pub mod status;
pub mod types;

pub use driver::MCP2221;
