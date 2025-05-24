//! Configuration of and interaction with the GP pins.

mod common;
mod pins;
mod settings;
mod values;

// Only used inside the GPIO module.
use common::PinNumber;

pub use common::{GpioDirection, LogicLevel};
/// Individual GP pins for use with embedded_hal::digital traits.
pub use pins::{GpPin, Input, Output, Pins};
/// GP pin configuration, include modes other than GPIO.
pub use settings::{
    Gp0Designation, Gp0Settings, Gp1Designation, Gp1Settings, Gp2Designation, Gp2Settings,
    Gp3Designation, Gp3Settings, GpSettings,
};
pub use values::{ChangeGpioValues, GpioValues, PinValue};
