# pico-testbed

This is firmware for a Raspberry Pi Pico plugged into a Pimoroni Pico
Explorer Base.

It provides a target for developing and testing mcp2221-hal.

## Wiring

| MCP2221 | Pico | Purpose          |
| ------- | ---- | ---------------- |
| UART Rx | GP0  | MCP to Pico UART |
| UART Tx | GP1  | Pico to MCP UART |
| GP0     | GP2  | MCP digital out  |
| GP3     | GP26 | MCP analog out   |
| SDA     | GP20 | I2C data         |
| SCL     | GP21 | I2C clock        |

## Interface summary

### I2C

See `src/tasks/i2c.rs`. Read, Write, Write-Read and General Call are all supported.

### UART

mcp2221-hal does not (really cannot) provide UART functionality, but it is
connected to Pico UART to provide a sanity check by echoing received characters.

### Digital IO

The MCP2221 GP0 is configured as a digital output, and is monitored by the Pico
on GP2. The logic level of the MCP2221 output is reflected in the state of the
Pico's built-in LED.

### Analog IO

The MCP2221 GP3 is configured as an analog output, and is read by the Pico's GP26,
configured as an analog input. The Pico does not have a DAC so the reverse cannot
be checked.
