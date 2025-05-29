# mcp2221-hal

A driver for the [Microchip MCP2221][microchip] and MCP2221A USB to I2C, UART and GPIO
converter. It supports the `embedded-hal` I2C and GPIO traits.

This crate uses the name "MCP2221", however please note that the MCP2221**A** is the
typical part and differs only from the original chip in fixing [a bug with UART
registers][errata] and allowing faster UART baud rates.

[errata]: https://www.microchip.com/en-us/product/mcp2221#Documentation

Our aim is that this crate is well enough documented that you can use it successfully
without having to look things up in the MCP2221 datasheet. We do frequently refer to the
appropriate sections, so if you wish to know the underlying details, [you can find the
latest revision (rev E) of the MCP2221 datasheet here][datasheet]. If you find a method
or field that you were only able to understand by referring to the datasheet, please
[open an issue].

[datasheet]: https://ww1.microchip.com/downloads/aemDocuments/documents/APID/ProductDocuments/DataSheets/MCP2221A-Data-Sheet-20005565E.pdf

# Quick start

Create the driver struct with default values by calling [`MCP2221::connect`].

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut device = mcp2221_hal::MCP2221::connect()?;
# Ok(())
# }
```

It implements the [blocking][i2c-b] and [async][i2c-a] (with the `async` feature) I2C
traits from `embedded-hal`. It has no mutable state, so you can pass a shared reference
to drivers expecting `impl I2c`.

[i2c-b]: embedded_hal::i2c::I2c
[i2c-a]: embedded_hal_async::i2c::I2c

```rust,no_run
use embedded_hal::i2c::I2c;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
#    let mut device = mcp2221_hal::MCP2221::connect()?;
let mut buf = [0u8; 2];
device.write_read(0x26, &[40, 2], &mut buf)?;
#    Ok(())
# }
```

For GPIO digital input and output, use the [`MCP2221::gpio_take_pins`] method, and
convert the [`GpPin`] objects into [`Input`] or [`Output`] types, which implement
the appropriate traits from [`embedded_hal::digital`].

[`GpPin`]: crate::gpio::GpPin
[`Input`]: crate::gpio::Input
[`Output`]: crate::gpio::Output


```rust,no_run
use embedded_hal::digital::{InputPin, OutputPin};
# use mcp2221_hal::gpio::{Pins, Input, Output};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
#    let mut device = mcp2221_hal::MCP2221::connect()?;
let Pins { gp0, gp1, .. } = device.gpio_take_pins().expect("take once");
let mut gp0: Input = gp0.try_into()?;
let mut gp1: Output = gp1.try_into()?;
if gp0.is_high()? {
    gp1.set_low()?;
}
#    Ok(())
# }
```

See the [`MCP2221`] struct documentation for more usage information.

[microchip]: https://www.microchip.com/en-us/product/mcp2221a

# Features

This driver supports all hardware functionality of the MCP2221, with the two exceptions
(discussed below) of the chip protection setting and UART (in a sense).

- Read and write settings (both SRAM and flash memory)
- GP pin functionality (GPIO, analog, and special modes)
    - [`embedded_hal::digital`] GPIO traits
- I2C
    - `embedded-hal` [blocking][i2c-b] and [async][i2c-a] traits

## Feature flags

- `async`: enables the implementation of [`embedded_hal_async::i2c::I2c`]. This is not
  on by default.

## Not-yet-supported

These features are not currently supported but could be with your help!

[open an issue]: https://github.com/robjwells/mcp2221-hal/issues

### Chip protection settings

The datasheet is not particularly clear how the password-protection mechanism
works, and other developers [have locked their MCP2221] by accident. If you have a
spare MCP2221 and would like this feature, please [open an issue].

[have locked their MCP2221]: https://forum.microchip.com/s/topic/a5C3l000000Mb3HEAS/t372487

### I2C: 10-bit addresses

The MCP2221 does not natively support 10-bit addresses, so this must be done in
software. However, I don't have an I2C device with a 10-bit address to test this.
Please [open an issue] if you do.

## Out of scope

These features are better solved outside this library.

### UART serial

The MCP2221 exposes a USB serial port (a CDC device), so you can use it as a
USB-to-serial converter and communicate with a connected device via UART. This is
entirely separate to the HID device used to perform all the other functionality, and
there is not a straightforward way for this library to identify which serial port
belongs to the MCP2221.

Instead, manually identify which serial port is that of the MCP2221, and connect to it
with your preferred serial port library. [`embedded-io`] has blanket implementations
for types that implement the [`std::io`] traits, so you can treat your serial port
library interface as if it was a device UART.

If you [enable USB CDC serial number enumeration][cdc-sn] (and optionally [customise the
serial number][sn-set]), you will be able to connect to the MCP2221 serial port at a stable
location.

[cdc-sn]: crate::settings::ChipSettings::cdc_serial_number_enumeration_enabled
[sn-set]: crate::MCP2221::usb_change_serial_number

### SMBus extra features

This driver supports SMBus as well as the MCP2221 does itself (via its I2C support), but
has no SMBus-specific software support for, eg, special transfer formats or packet error
checking. You should use an SMBus library on top of this driver's I2C support.

# `embedded-hal` notes

## `I2c::transaction` not fully fully supported
 
The MCP2221 cannot support the full generality of the `I2c::transaction` method of the
[`embedded_hal::i2c::I2c`] trait, because there is no command to perform a read without
issuing a Stop condition at the end. This means you cannot issue a transaction where a
read precedes a write, and will receive an error if you try to do so.

## Async `I2c` blocks

The driver supports the [async `I2c`][i2c-a] trait, but there is no async access to the
USB device, so the async trait methods just call the blocking trait methods.

However, this still allows you to, for example, develop an async `embedded-hal` driver
using the MCP2221.

# Strange behaviour

We have identified strange behaviours of the MCP2221 that may be firmware bugs.

- Settings read from SRAM may not reflect the current device behaviour. (See the
  [`settings`] module documentation.)
- Trying to cancel an I2C transfer when the I2C engine is idle makes the I2C engine busy
  (this library tries to work around this.)
- DAC reference set to Vrm with a level of Vdd/"off" outputs 0V. (See the [`analog`] module
  documentation.)
- Setting ADC or DAC reference to Vrm (any level) in flash memory starts the device with
  Vrm set to Vdd/"off" (See the [`analog`] module documentation.)
- Changing GP pin settings in SRAM sets the ADC and DAC references to Vrm with a level
  of Vdd/"off". (See the note in section 1.8.1.1 of the datasheet. This library attempts
  to work around this bug.)

# Thread safety

The driver cannot be used across threads (it is `!Sync`). The driver uses the [`hidapi`]
crate, which in turn uses the [hidapi C library]. Creating multiple drivers in different
threads _should_ fail with an error (`Result::Err`), but it might not and you may end up
with a crash later on. You can see this for yourself by running the project's tests with
`cargo test` (we use `cargo nextest` in sequential mode to avoid this).

[hidapi C library]: https://github.com/signal11/hidapi/

# Related libraries

- [mcp2221-rs][]: supports the `embedded-hal` 0.2 I2C trait, and has some GPIO features.

  Thank you to David Lattimore for writing this crate, it's what got me up and running
  with the MCP2221. David is now writing [wild][], a very fast linker.

- [ftdi-embedded-hal][]: a similar library for FTDI USB converters.

[mcp2221-rs]: https://github.com/google/mcp2221-rs
[wild]: https://github.com/davidlattimore/wild
[ftdi-embedded-hal]: https://github.com/ftdi-rs/ftdi-embedded-hal
