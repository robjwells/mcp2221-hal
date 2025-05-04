#![allow(dead_code)]
#![allow(unused_variables)]

mod commands;
mod driver;
pub mod error;
pub mod flash_data;
pub mod status;
pub mod types;

pub use driver::MCP2221;
