use super::{GpioChanges, GpioDirection, GpioValues, LogicLevel};
use crate::settings::{Gp0Mode, Gp1Mode, Gp2Mode, Gp3Mode, GpSettings};
use crate::{Error, MCP2221};

/// A GP pin that be configured for GPIO input or output.
///
/// This struct is intended for use with the [`embedded_hal`] GPIO traits. If you
/// wish to configure a pin for a mode other than GPIO, use the driver method
/// [`MCP2221::sram_write_gp_settings`].
#[derive(Debug)]
pub struct GpPin<'a> {
    /// Driver can be a shared reference because its methods take &self.
    driver: &'a MCP2221,
    /// The underlying pin.
    pin_number: PinNumber,
}

impl<'a> GpPin<'a> {
    /// Set the pin as a GPIO digital input.
    pub fn configure_as_digital_input(self) -> Result<Input<'a>, Error> {
        let (_, mut gp_settings) = self.driver.sram_read_settings()?;
        self.pin_number
            .configure_as_gpio(&mut gp_settings, GpioDirection::Input);
        self.driver.sram_write_gp_settings(gp_settings)?;
        Ok(Input(self))
    }

    /// Set the pin as a GPIO digital output.
    pub fn configure_as_digital_output(self) -> Result<Output<'a>, Error> {
        let (_, mut gp_settings) = self.driver.sram_read_settings()?;
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
///
/// This struct is intended for use with the [`embedded_hal::digital::InputPin`] trait.
pub struct Input<'a>(GpPin<'a>);

impl<'a> Input<'a> {
    /// Get the current settings for the GPIO pin.
    ///
    /// Returns an error if the pin is no longer a GPIO input or not in GPIO mode.
    /// (That indicates a driver method has been called to reconfigure the pins
    /// while this `Input` was held.)
    fn pin_settings(&self) -> Result<(GpioDirection, LogicLevel), Error> {
        let gpio_values = self.0.driver.gpio_read()?;
        self.0
            .pin_number
            .get_value(&gpio_values)
            .filter(|(dir, _)| dir.is_input())
            .ok_or(Error::PinModeChanged)
    }

    /// Get the input level of this pin.
    pub fn get_level(&self) -> Result<LogicLevel, Error> {
        self.pin_settings().map(|(_, level)| level)
    }

    /// Extract the underlying [`GpPin`].
    ///
    /// This method does not change any MCP2221 settings.
    ///
    /// If you only wish to change the GPIO direction of the pin, use
    /// [`Input::try_into_output`].
    pub fn destroy(self) -> GpPin<'a> {
        self.0
    }

    /// Switch the pin mode from input to output.
    pub fn try_into_output(self) -> Result<Output<'a>, Error> {
        // Ensure still set for GPIO operation.
        self.pin_settings()?;
        // Change the direction.
        let mut changes = GpioChanges::new();
        self.0
            .pin_number
            .change_direction(&mut changes, GpioDirection::Output);
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
///
/// This struct is intended for use with the [`embedded_hal::digital::OutputPin`] and
/// [`embedded_hal::digital::StatefulOutputPin`] traits.
pub struct Output<'a>(GpPin<'a>);

impl<'a> Output<'a> {
    /// Get the current settings for the GPIO pin.
    ///
    /// Returns an error if the pin is no longer a GPIO output or not in GPIO mode.
    /// (That indicates a driver method has been called to reconfigure the pins
    /// while this `Output` was held.)
    fn pin_settings(&self) -> Result<(GpioDirection, LogicLevel), Error> {
        let gpio_values = self.0.driver.gpio_read()?;
        self.0
            .pin_number
            .get_value(&gpio_values)
            .filter(|(dir, _)| dir.is_output())
            .ok_or(Error::PinModeChanged)
    }

    /// Set the output level of this pin.
    pub fn set_level(&self, level: LogicLevel) -> Result<(), Error> {
        // Ensure the pin is still set as an output.
        self.pin_settings()?;
        let mut changes = GpioChanges::new();
        self.0.pin_number.change_level(&mut changes, level);
        self.0.driver.gpio_write(&changes)
    }

    /// Get the current output level of this pin.
    pub fn get_output_level(&self) -> Result<LogicLevel, Error> {
        self.pin_settings().map(|(_, level)| level)
    }

    /// Extract the underlying [`GpPin`].
    ///
    /// This method does not change any MCP2221 settings.
    ///
    /// If you only wish to change the GPIO direction of the pin, use
    /// [`Output::try_into_input`].
    pub fn destroy(self) -> GpPin<'a> {
        self.0
    }

    /// Switch the pin mode from output to input.
    pub fn try_into_input(self) -> Result<Input<'a>, Error> {
        // Ensure still set for GPIO operation.
        self.pin_settings()?;
        // Change the direction.
        let mut changes = GpioChanges::new();
        self.0
            .pin_number
            .change_direction(&mut changes, GpioDirection::Input);
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

/// Wrapper for the four GP pins.
///
/// This is returned from [`MCP2221::gpio_take_pins`], so each pin can be used as an
/// [`Input`] or [`Output`], for use with the [`embedded_hal::digital`] traits.
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

/// The specific GP pin number of a given GpPin.
///
/// This enum is used only in this module, to allow for a [`GpPin`] struct that
/// can be used without, say, pin-specific generics.
#[derive(Debug, Clone, Copy)]
enum PinNumber {
    Gp0,
    Gp1,
    Gp2,
    Gp3,
}

impl PinNumber {
    /// Change `GpSettings` to put a particular GP pin into GPIO mode with the given direction.
    fn configure_as_gpio(&self, gp_settings: &mut GpSettings, direction: GpioDirection) {
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
                gp_settings.gp3_mode = Gp3Mode::Gpio;
                gp_settings.gp3_direction = direction;
            }
        }
    }

    /// Extract the values for the particular pin from the Get GPIO Values result.
    ///
    /// This is defined on `PinNumber` so that no other module needs to concern itself
    /// with the definition of `PinNumber`. It makes it very slightly more awkward to
    /// fetch the level from the `GpioValues` struct, but that only happens in two
    /// places in this module since this is not exposed anywhere else.
    fn get_value(&self, gpio_values: &GpioValues) -> Option<(GpioDirection, LogicLevel)> {
        match self {
            PinNumber::Gp0 => gpio_values.gp0,
            PinNumber::Gp1 => gpio_values.gp1,
            PinNumber::Gp2 => gpio_values.gp2,
            PinNumber::Gp3 => gpio_values.gp3,
        }
    }

    /// Update `GpioChanges` to set the GPIO output level for a particular GP pin.
    fn change_level(&self, changes: &mut GpioChanges, level: LogicLevel) {
        match self {
            PinNumber::Gp0 => changes.with_gp0_level(level),
            PinNumber::Gp1 => changes.with_gp1_level(level),
            PinNumber::Gp2 => changes.with_gp2_level(level),
            PinNumber::Gp3 => changes.with_gp3_level(level),
        };
    }

    /// Update `GpioChanges` to set the GPIO direction for a particular GP pin.
    fn change_direction(&self, changes: &mut GpioChanges, direction: GpioDirection) {
        match self {
            PinNumber::Gp0 => changes.with_gp0_direction(direction),
            PinNumber::Gp1 => changes.with_gp1_direction(direction),
            PinNumber::Gp2 => changes.with_gp2_direction(direction),
            PinNumber::Gp3 => changes.with_gp3_direction(direction),
        };
    }
}
