# MCP2221 CLI

A command-line interface for the [MCP2221 USB to I2C and GPIO converter][mcp].

This allows you to interact in a simple manner with a connected MCP2221 without having
to write a standalone program to, for example, send and receive I2C data or change
settings.

The CLI uses [mcp2221-hal][], which you might like to check out if you're writing
embedded-hal device drivers via an MCP2221.

## Features

- I2C: read, write, write-read, address check
- GPIO: read digital input, set digital output
- Analog input (ADC) and output (DAC)
- Configure pins into other modes (activity indication, for example)
- Change USB device information, such as product and manufacturer strings.
- Reset the device

## Installation

For the moment, just use Cargo:

```
cargo install --locked mcp2221-cli
```

## License

This crate is licensed under MIT or Apache 2.0 (at your option).

[mcp]: https://www.microchip.com/en-us/product/mcp2221a
[mcp2221-hal]: https://github.com/robjwells/mcp2221-hal
