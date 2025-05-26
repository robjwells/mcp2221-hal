//! Digital input and output.
//!
//! # Input and output pin types
//!
//! [`Input`] and [`Output`] are the most important types of this module. They represent
//! a GP pin configured for GPIO operation, and implement the appropriate traits from
//! [`embedded_hal::digital`].
//!
//! To use them, call [`MCP2221::take_pins`], and convert the returned [`GpPin`] objects
//! as needed into the appropriate type of GPIO pin. Pins can be taken only once from
//! the driver, subsequent calls will return `None`.
//!
//! ```no_run
//! # use mcp2221_hal::{MCP2221, gpio::{Input, Output, Pins}};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use embedded_hal::digital::{InputPin, OutputPin};
//! let device = MCP2221::open()?;
//! let Pins { gp0, gp1, .. } = device.take_pins().expect("Can only take once.");
//! let mut gp0: Input = gp0.try_into()?;
//! let mut gp1: Output = gp1.try_into()?;
//! if gp0.is_high()? {
//!     gp1.set_high()?;
//! }
//! # Ok(())
//! # }
//! ````
//!
//! [`Input`]s and [`Output`]s can be converted between each other. You can also destroy
//! them and extract the contained [`GpPin`], a type which just represents a pin with
//! an unknown mode.
//!
//! # Driver GPIO supporting types
//!
//! The [`MCP2221::gpio_read`] and [`MCP2221::gpio_write`] methods use the
//! [`GpioValues`] and [`GpioChanges`] structs, respectively. These methods can be used
//! to adjust the GPIO settings of several pins at once. However, these methods do not
//! put pins into GPIO mode, which must be done with [`MCP2221::sram_write_settings`].
//! If a GP pin is not in GPIO mode, then its corresponding field in [`GpioValues`] will
//! be `None`.
//!
//! # Changing pin direction or mode
//!
//! You can convert between [`Input`] and [`Output`] pins to change their direction:
//! ```no_run
//! # use mcp2221_hal::{MCP2221, gpio::{Input, Output}};
//! # use embedded_hal::digital::{InputPin, OutputPin};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let device = MCP2221::open()?;
//! let mut gp2: Output = device.take_pins().expect("take once").gp2.try_into()?;
//! gp2.set_high()?;
//! let mut gp2: Input = gp2.try_into()?;
//! println!("Reading low? {}", gp2.is_low()?);
//! let mut gp2: Output = gp2.try_into()?;
//! # Ok(())
//! # }
//! ```
//!
//! If you are using the [`Input`] and [`Output`] types, you should not use the
//! [`MCP2221::gpio_write`] method to change the direction of those pins, as you will
//! receive an error when trying to use their methods. Likewise, don't use
//! [`MCP2221::sram_write_settings`] to change the mode or direction of in-use pins for
//! the same reason.
//!
//! ```no_run
//! # use mcp2221_hal::{MCP2221, gpio::{GpioChanges, GpioDirection, LogicLevel, Output}};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let device = MCP2221::open()?;
//! let gp3: Output = device.take_pins().expect("take once").gp3.try_into()?;
//! device.gpio_write(
//!     GpioChanges::new()
//!         .with_gp3_direction(GpioDirection::Input)
//! )?;
//! assert!(
//!     gp3.set_level(LogicLevel::Low).is_err(),
//!     "Pin direction changed."
//! );
//! # Ok(())
//! # }
//! ```
//!
//! [`MCP2221::take_pins`]: crate::MCP2221::take_pins
//! [`MCP2221::gpio_write`]: crate::MCP2221::gpio_write
//! [`MCP2221::gpio_read`]: crate::MCP2221::gpio_read
//! [`MCP2221::sram_write_settings`]: crate::MCP2221::sram_write_settings

mod common;
mod pins;
mod values;

pub use common::{GpioDirection, LogicLevel};
pub use pins::{GpPin, Input, Output, Pins};
pub use values::{GpioChanges, GpioValues};
