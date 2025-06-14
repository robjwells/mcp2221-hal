# MCP2221 driver and command-line interface

The [MCP2221][microchip] is a USB to I2C and GPIO adapter, which allows you to
interact with embedded devices from your computer. For example, you could
connect an I2C temperature sensor to the MCP2221 and take readings from it
without needing to program a separate microcontroller.

The MCP2221 is especially good for [`embedded-hal`][] driver development,
particularly for I2C drivers when using a breakout board with a Stemma QT (aka
Qwiic) connector that lets you connect to your target device with a cable,
without having to do any soldering. I like the [Adafruit breakout
board][adafruit], product number 4471.

[microchip]: https://www.microchip.com/en-us/product/mcp2221a
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[adafruit]: https://www.adafruit.com/product/4471

## Crates in this repo

- **`mcp2221-hal`** is the HAL (or driver) crate that exposes the functionality
  of the MCP2221 to your Rust programs.
- **`mcp2221-cli`** is a command-line program that uses the `mcp2221-hal` crate to
  allow you to interact with a connected MCP2221 from your terminal or a shell script.
- `pico-target` is firmware for a Raspberry Pi Pico (at the moment, specifically one
  plugged into a [Pimoroni Pico Explorer Base][]) that provides a target with which
  to communicate via the MCP2221 in the `mcp2221-hal` tests.

[Pimoroni Pico Explorer Base]: https://shop.pimoroni.com/products/pico-explorer-base

## Part name

The original [MCP2221][original] is not recommended for new designs, and the
most common part now seems to be the [MCP2221**A**][microchip], which fixes some
[UART bugs][] in the original MCP2221 and allows for higher UART speeds. This
library will work with either.

[original]: https://www.microchip.com/en-us/product/mcp2221
[UART bugs]: https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/Errata/80000742A.pdf

## License

The `mcp2221-hal` and `mcp2221-cli` crates are licensed under MIT or Apache 2.0
(at your option).

All code in the reference directory is copyright Microchip Technology Inc and
subject to the license described in the individual source files. It is provided
in this repository purely as a reference and aid in developing the Rust driver,
and is not redistributed in either published crate. Thank you to Microchip for
providing these drivers as open source software.
