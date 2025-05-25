//! Configuration of and interaction with the GP pins.

mod common;
mod pins;
mod values;

// Only used inside the GPIO module.
use pins::PinNumber;

pub use common::{GpioDirection, LogicLevel};
pub use pins::{GpPin, Input, Output, Pins};
pub use values::{ChangeGpioValues, GpioValues, PinValue};
