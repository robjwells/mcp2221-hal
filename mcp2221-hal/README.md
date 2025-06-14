# mcp2221-hal

A driver for the [Microchip MCP2221][microchip] and MCP2221A USB to I2C, UART
and GPIO converter. It supports the [`embedded-hal`][] I2C and GPIO traits,
including [`embedded-hal-async`][].

A **CLI** to interact with the MCP2221, using this crate, is available as
[`mcp2221-cli`][].

## Feature flags

- `async` enables `embedded-hal-async` support.

## Features

All features of the MCP2221 are supported, except for "chip protection".

- I2C (with 7-bit addresses)
- GPIO
- ADC and DAC (analog input and output)
- Pin alternate modes (eg edge detection, clock output, etc)
- USB description configuration (eg PID, manufacturer name, etc)
- Initial configuration (in flash) and running configuration (in SRAM)

## Quick start

Here's a simple example showing I2C and GPIO usage:

```rust
// Creation with default USB VID & PID
let mut device = mcp2221_hal::MCP2221::connect()?;

// I2C
use embedded_hal::i2c::I2c;
let mut buf = [0u8; 2];
device.write_read(0x26, &[40, 2], &mut buf)?;
println!("{buf:?}");

// GPIO
use embedded_hal::digital::{InputPin, OutputPin};
use mcp2221_hal::gpio::{Input, Output, Pins};
let Pins { gp0, gp1, .. } = device.gpio_take_pins().expect("take once");
let mut gp0: Input = gp0.try_into()?;
let mut gp1: Output = gp1.try_into()?;
if gp0.is_high()? {
    gp1.set_low()?;
}
```

## UART (USB CDC serial)

This HAL does not contain UART-related functionality, because the MCP2221 exposes its
UART feature as a separate USB CDC serial port. You are better served using a separate
serial port library. See the crate documentation for more detailed advice.

## Quirks and unsupported features

This crate is **not thread safe** due to the underlying `hidapi` C library. The
`MCP2221` struct itself is `!Sync`, and `hidapi` _should_ prevent you from
connecting to a device that is already in use. However, sometimes you can
connect to the same device from another thread, only to crash later on. (Run
`cargo test` in the repo to see this in action!)

The **chip-protection** features (requiring a password for configuration
changes, or permanently locking the configuration) are not supported. If you
have an already-protected MCP2221 you will likely encounter an error.

**10-bit I2C addresses** are not natively supported by the MCP2221 and are not
_currently_ supported by the HAL (get in touch if you have a 10-bit target!).

The **I2C transaction** interface of `embedded-hal` does not support transactions where
a read precedes a write, with a repeated-Start in between. This is a limitation of the
MCP2221 USB HID interface (there's no way to perform a read without a Stop condition).

The **async** I2C interface blocks (it calls the `embedded-hal` trait methods).
There isn't async access to the USB HID device, so this is provided as a
convenience for developing `embedded-hal-async` drivers.

The extra functionality required for proper **SMBus** support is not provided by this
crate. The MCP2221 itself provides only standard I2C support, and you are better
served using a separate SMBus library on top of this crate’s I2C features.

A number of **strange (buggy?) behaviours** of the MCP2221 have been identified
during development (particularly with the ADC and DAC). See the crate
documentation for details.

## Related libraries

- [mcp2221-rs][]: supports the `embedded-hal` 0.2 I2C trait, and has some GPIO features.

  Thank you to David Lattimore for writing this crate, it's what got me up and running
  with the MCP2221. David is now writing [wild][], a very fast linker.

- [ftdi-embedded-hal][]: a similar library for FTDI USB converters.

[microchip]: https://www.microchip.com/en-us/product/mcp2221a
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
[`mcp2221-cli`]: https://crates.io/crates/mcp2221-cli
[mcp2221-rs]: https://github.com/google/mcp2221-rs
[wild]: https://github.com/davidlattimore/wild
[ftdi-embedded-hal]: https://github.com/ftdi-rs/ftdi-embedded-hal
