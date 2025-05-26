#![allow(dead_code)]

use super::{ChangeGpioValues, GpioDirection, LogicLevel, PinValue};
use crate::{
    Error, MCP2221,
    settings::{Gp0Mode, Gp1Mode, Gp2Mode, Gp3Mode, GpSettings},
};

/// A GP pin that be configured for GPIO input or output.
#[derive(Debug)]
pub struct GpPin<'a> {
    driver: &'a MCP2221,
    pin_number: PinNumber,
}

impl<'a> GpPin<'a> {
    /// Set up the GP pin as a GPIO digital input.
    ///
    /// You can retrieve the pin (for reconfiguration as an output) by calling
    /// [`Input::destroy`].
    pub fn configure_as_digital_input(self) -> Result<Input<'a>, Error> {
        let sram_settings = self.driver.sram_read_settings()?;
        let mut gp_settings = sram_settings.gp_settings;
        self.pin_number
            .configure_as_gpio(&mut gp_settings, GpioDirection::Input);
        self.driver.sram_write_gp_settings(gp_settings)?;
        Ok(Input(self))
    }

    /// Set up the GP pin as a GPIO digital output.
    ///
    /// You can retrieve the pin (for reconfiguration as an input) by calling
    /// [`Output::destroy`].
    pub fn configure_as_digital_output(self) -> Result<Output<'a>, Error> {
        let mut gp_settings = self.driver.sram_read_settings()?.gp_settings;
        self.pin_number
            .configure_as_gpio(&mut gp_settings, GpioDirection::Output);
        self.driver.sram_write_gp_settings(gp_settings)?;
        Ok(Output(self))
    }
}

impl<'a> TryFrom<GpPin<'a>> for Input<'a> {
    type Error = Error;
    fn try_from(pin: GpPin<'a>) -> Result<Self, Self::Error> {
        pin.configure_as_digital_input()
    }
}

impl<'a> TryFrom<GpPin<'a>> for Output<'a> {
    type Error = Error;
    fn try_from(pin: GpPin<'a>) -> Result<Self, Self::Error> {
        pin.configure_as_digital_output()
    }
}

impl<'a> From<Input<'a>> for GpPin<'a> {
    fn from(value: Input<'a>) -> Self {
        value.destroy()
    }
}

impl<'a> From<Output<'a>> for GpPin<'a> {
    fn from(value: Output<'a>) -> Self {
        value.destroy()
    }
}

/// A GP pin in GPIO input mode.
pub struct Input<'a>(GpPin<'a>);

impl<'a> Input<'a> {
    /// Get the input level of this pin.
    pub fn get_level(&self) -> Result<LogicLevel, Error> {
        self.0
            .driver
            .gpio_read()?
            .for_pin_number(self.0.pin_number)
            .ok_or(Error::PinModeChanged)
            .map(|v| v.level)
    }

    /// Return the underlying Pin object, so that it can be reconfigured.
    ///
    /// This method does not change any MCP2221 settings.
    pub fn destroy(self) -> GpPin<'a> {
        self.0
    }

    /// Switch the pin mode from input to output.
    pub fn try_into_output(self) -> Result<Output<'a>, Error> {
        // Ensure still set for GPIO operation.
        self.get_level()?;
        // Change the direction.
        let mut changes = ChangeGpioValues::new();
        changes.with_direction_for_pin_number(self.0.pin_number, GpioDirection::Output);
        self.0.driver.gpio_write(&changes)?;
        // Return the new wrapper type now the direction change is done.
        Ok(Output(self.0))
    }
}

impl embedded_hal::digital::Error for Error {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        // Currently the only variant.
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for Input<'_> {
    type Error = Error;
}

impl embedded_hal::digital::InputPin for Input<'_> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.get_level().map(LogicLevel::is_high)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.get_level().map(LogicLevel::is_low)
    }
}

/// A GP pin in GPIO output mode.
pub struct Output<'a>(GpPin<'a>);

impl<'a> Output<'a> {
    fn pin_settings(&self) -> Result<PinValue, Error> {
        self.0
            .driver
            .gpio_read()?
            .for_pin_number(self.0.pin_number)
            .filter(|v| v.direction.is_output())
            .ok_or(Error::PinModeChanged)
    }

    /// Set the output level of this pin.
    pub fn set_level(&self, level: LogicLevel) -> Result<(), Error> {
        // Ensure the pin is still set as an output.
        self.pin_settings()?;
        let mut changes = ChangeGpioValues::new();
        changes.with_level_for_pin_number(self.0.pin_number, level);
        self.0.driver.gpio_write(&changes)
    }

    /// Get the currently set output level of this pin.
    pub fn get_output_level(&self) -> Result<LogicLevel, Error> {
        self.pin_settings().map(|v| v.level)
    }

    /// Return the underlying Pin object, so that it can be reconfigured.
    pub fn destroy(self) -> GpPin<'a> {
        self.0
    }

    /// Switch the pin mode from output to input.
    pub fn try_into_input(self) -> Result<Input<'a>, Error> {
        // Ensure still set for GPIO operation.
        self.pin_settings()?;
        // Change the direction.
        let mut changes = ChangeGpioValues::new();
        changes.with_direction_for_pin_number(self.0.pin_number, GpioDirection::Input);
        self.0.driver.gpio_write(&changes)?;
        // Return the new wrapper type now the direction change is done.
        Ok(Input(self.0))
    }
}

impl embedded_hal::digital::ErrorType for Output<'_> {
    type Error = Error;
}

impl embedded_hal::digital::OutputPin for Output<'_> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_level(LogicLevel::Low)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_level(LogicLevel::High)
    }
}

impl embedded_hal::digital::StatefulOutputPin for Output<'_> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.get_output_level().map(LogicLevel::is_high)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.get_output_level().map(LogicLevel::is_low)
    }
}

impl<'a> TryFrom<Output<'a>> for Input<'a> {
    type Error = Error;

    fn try_from(value: Output<'a>) -> Result<Self, Self::Error> {
        value.try_into_input()
    }
}

impl<'a> TryFrom<Input<'a>> for Output<'a> {
    type Error = Error;

    fn try_from(value: Input<'a>) -> Result<Self, Self::Error> {
        value.try_into_output()
    }
}

/// The four MCP2221 GP pins.
#[derive(Debug)]
pub struct Pins<'a> {
    /// Pin GP0
    pub gp0: GpPin<'a>,
    /// Pin GP1
    pub gp1: GpPin<'a>,
    /// Pin GP2
    pub gp2: GpPin<'a>,
    /// Pin GP3
    pub gp3: GpPin<'a>,
}

impl<'a> Pins<'a> {
    pub(crate) fn new(driver: &'a MCP2221) -> Self {
        Self {
            gp0: GpPin {
                pin_number: PinNumber::Gp0,
                driver,
            },
            gp1: GpPin {
                pin_number: PinNumber::Gp1,
                driver,
            },
            gp2: GpPin {
                pin_number: PinNumber::Gp2,
                driver,
            },
            gp3: GpPin {
                pin_number: PinNumber::Gp3,
                driver,
            },
        }
    }
}

/// The specific GP pin number of a given Pin.
#[derive(Debug, Clone, Copy)]
pub(super) enum PinNumber {
    Gp0,
    Gp1,
    Gp2,
    Gp3,
}

impl PinNumber {
    pub(crate) fn configure_as_gpio(&self, gp_settings: &mut GpSettings, direction: GpioDirection) {
        match self {
            PinNumber::Gp0 => {
                gp_settings.gp0_mode = Gp0Mode::Gpio;
                gp_settings.gp0_direction = direction;
            }
            PinNumber::Gp1 => {
                gp_settings.gp1_mode = Gp1Mode::Gpio;
                gp_settings.gp1_direction = direction;
            }
            PinNumber::Gp2 => {
                gp_settings.gp2_mode = Gp2Mode::Gpio;
                gp_settings.gp2_direction = direction;
            }
            PinNumber::Gp3 => {
                gp_settings.gp3_mode = Gp3Mode::GPIO;
                gp_settings.gp3_direction = direction;
            }
        }
    }
}
