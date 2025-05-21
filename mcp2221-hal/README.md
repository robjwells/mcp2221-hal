# mcp2221-hal

A driver for the [Microchip MCP2221][microchip] USB to I2C, UART and GPIO converter.
This driver typically refers to the "MCP2221", however, please note that the
MCP2221**A** is the typical part and differs only from the original MCP2221
in fixing [a bug with UART registers][errata] and allowing faster UART baud rates.

[microchip]: https://www.microchip.com/en-us/product/mcp2221a
[errata]: https://www.microchip.com/en-us/product/mcp2221#Documentation

## Supported features

(My todo list!)

- [x] Read and write settings
    + [x] Power-up settings in flash memory
    + [x] Run-time settings in SRAM
- [x] GP pin modes
    + [x] Digital input and output (GPIO)
    + [x] Analog input and output
    + [x] Clock output, LED indicator output, interrupt detection
- [ ] I2C
    + [ ] Write
        - [ ] 3.1.5 I2C Write Data
        - [ ] 3.1.6 I2C Write Data Repeated Start
        - [ ] 3.1.7 I2C Write Data No Stop
    + [ ] Read
        - [ ] 3.1.8 I2C Read Data
        - [ ] 3.1.9 I2C Read Data Repeated Start
    + [ ] WriteRead
    + [x] Standard I2C bus speeds (100k/400k)
    + [ ] Custom I2C bus speeds (47k–400k)
- [ ] [embedded-hal] traits
- [ ] UART serial (for now, interact with the MCP2221 CDC device directly)

[embedded-hal]: https://github.com/rust-embedded/embedded-hal

## Unsupported features

Currently there is no plan to support the following, though please [open an issue]
if you are in need of them.

- SMBus\*
- Password protected settings
- Permanently locked settings

\* Note that the MCP2221 has no special support for SMBus other than its I2C support.
Any particular support for SMBus must be done in software.

## `embedded-hal` support

Note that the MCP2221 cannot support the full generality of the `I2c::transaction`
method of the `embedded-hal::i2c::I2c` trait, because there is no command to issue
a read without issuing a Stop condition at the end.

## Deviations from the datasheet

### Vrm level selection

The HID commands allow for setting the Vrm level when using Vdd as a voltage reference.
This driver always sets the Vrm level to "off" when using Vdd as the reference.

### Vrm and Vdd selection

The datasheet is inconsistent as to whether, when selecting a voltage reference source,
1 means Vrm or Vdd (and 0 the opposite). Generally, the datasheet says that 1 means Vrm
and 0 means Vdd, except when writing the DAC reference to flash (table 3-12) or when
reading the ADC reference from flash (table 3-5).

In practice, 1 is Vrm and 0 is Vdd, and it appears the cases where this is reversed are
typos. (There are also odd typos where the ADC is described as the DAC, and one in table
3-36 where both Vrm and Vdd appear in the description of a bit setting.)

## Unexpected behaviour

Certain actions may cause the MCP2221 to behave strangely.
(This list is likely incomplete!)

### DAC and Vrm

Setting the DAC to use Vrm with a level of "off" (also referred to in the datasheet
as setting Vrm to reference Vdd) will cause the DAC to output a very low voltage level
regardless of the set output value.

Setting the DAC's power-up reference (ie, setting the DAC reference in flash memory)
to Vrm at _any_ level starts the DAC in the above described low-voltage Vrm-off mode.

Neither of these are documented in the datasheet.

### ADC and Vrm

Setting the ADC's power-up reference (ie, setting the ADC reference in flash memory)
to Vrm at _any_ level starts the ADC in Vrm-off mode.

However, the ADC works as the datasheet describes in this mode, that is it appears to
be equivalent to using Vdd as the voltage reference.

The ADC power-up behaviour is not documented in the datasheet.

### Changing GP pin settings in SRAM changes Vrm level

Changing the GP pin settings in SRAM without also explicitly setting the Vrm level
for the ADC and DAC causes the Vrm level for both to be set to "off". This affects
the Set SRAM Settings HID command.

This is documented in a note in section 1.8.1.1 since datasheet revision D. This driver
tries to mitigate this by taking optional voltage references when altering the GP pin
settings in SRAM.

### Stale SRAM settings

The SRAM settings read from the device may not reflect the current behaviour of the
device. Two causes have been discovered so far:

- Changing GP pin settings as mentioned above. The SRAM settings for the ADC and DAC
  references do not change to reflect the Vdd-off behaviour.
- Changing GPIO pin direction or output value via the Set GPIO Output Values HID
  command. The SRAM settings will not reflect the changed values. (However, this
  command does not trigger the voltage reference reset.)

If you wish to rely on the read SRAM settings to accurately reflect the current
behaviour of the MCP2221, change GP pin settings via the Set SRAM Settings command
and always explicitly set the Vrm levels.

The stale settings behaviour is not documented in the datasheet.

### I2C transfer cancellation causes an idle engine to become busy

The Status/Set Parameters has a subcommand to cancel an ongoing I2C transfer. However,
issuing this command to the MCP2221 when the I2C engine is _not_ performing a transfer
puts the engine into a busy state. It appears that device must be reset to return the
I2C engine to idle.

This behaviour is not documented in the datasheet. This driver works around it by only
issuing a cancellation if the engine is not idle (as read from the Status command).

<!-- TODO: Update this if performing I2C transfers clears the busy state. -->

## DAC output curve

The DAC output voltage is not linear and does not reach up to its voltage reference.
The following readings were taken using an [Adafruit MCP2221A breakout board][adafruit],
a supply voltage of 3.3V, and read with the 10-bit ADC of a Raspberry Pi Pico. Please
note these are very rough measurements meant only to indicate the expected DAC output
voltages. Strictly speaking, the Vrm references are documented in the datasheet as
being 1.024V and 2.048V.

(Also I am very much a novice at electronics, so if you can think of a reason why I
may have been "holding it wrong" or whatever, please do [open an issue].)

[adafruit]: https://www.adafruit.com/product/4471
[open an issue]: https://github.com/robjwells/mcp2221-hal/issues

<details>
    <summary><h4>Chart and table of DAC output voltage readings</h4></summary>

![A chart showing the output of the MCP2221 DAC while using as reference Vdd, Vrm at 1V, and Vrm at 2V.][dac-voltages]

| Value | Vdd @ 3.3V | Vrm @ 2V | Vrm @ 1V |
| ----: | ---------: | -------: | -------: |
|     0 |       0.03 |     0.03 |     0.03 |
|     1 |       0.11 |     0.08 |     0.06 |
|     2 |       0.16 |     0.11 |     0.08 |
|     3 |       0.20 |     0.14 |     0.09 |
|     4 |       0.24 |     0.16 |     0.10 |
|     5 |       0.27 |     0.18 |     0.11 |
|     6 |       0.30 |     0.20 |     0.12 |
|     7 |       0.32 |     0.22 |     0.13 |
|     8 |       0.35 |     0.23 |     0.14 |
|     9 |       0.38 |     0.25 |     0.15 |
|    10 |       0.40 |     0.26 |     0.15 |
|    11 |       0.42 |     0.28 |     0.16 |
|    12 |       0.44 |     0.29 |     0.17 |
|    13 |       0.47 |     0.31 |     0.18 |
|    14 |       0.50 |     0.33 |     0.19 |
|    15 |       0.54 |     0.35 |     0.19 |
|    16 |       0.57 |     0.37 |     0.20 |
|    17 |       0.60 |     0.39 |     0.21 |
|    18 |       0.65 |     0.41 |     0.23 |
|    19 |       0.69 |     0.43 |     0.24 |
|    20 |       0.75 |     0.46 |     0.25 |
|    21 |       0.80 |     0.49 |     0.27 |
|    22 |       0.87 |     0.53 |     0.28 |
|    23 |       0.94 |     0.57 |     0.30 |
|    24 |       1.03 |     0.62 |     0.33 |
|    25 |       1.14 |     0.68 |     0.35 |
|    26 |       1.27 |     0.75 |     0.39 |
|    27 |       1.42 |     0.84 |     0.42 |
|    28 |       1.60 |     0.94 |     0.46 |
|    29 |       1.83 |     1.07 |     0.52 |
|    30 |       2.12 |     1.23 |     0.60 |
|    31 |       2.47 |     1.43 |     0.70 |

</details>


<!-- Image data beyond. -->

<!-- This is the base64-encoded voltage chart png in the assets directory. -->
[dac-voltages]: data:image/png;base64,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

