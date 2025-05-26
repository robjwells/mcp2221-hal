# mcp2221-hal

A driver for the [Microchip MCP2221][microchip] USB to I2C, UART and GPIO converter.
This driver typically refers to the "MCP2221", however, please note that the
MCP2221**A** is the typical part and differs only from the original MCP2221
in fixing [a bug with UART registers][errata] and allowing faster UART baud rates.

[microchip]: https://www.microchip.com/en-us/product/mcp2221a
[errata]: https://www.microchip.com/en-us/product/mcp2221#Documentation

## Supported features

- [x] Read and write settings
    + [x] Power-up settings in flash memory
    + [x] Run-time settings in SRAM
- [x] GP pin modes
    + [x] Digital input and output (GPIO)
    + [x] Analog input and output
    + [x] Clock output, LED indicator output, interrupt detection
- [x] I2C
    + [x] Write
        - [x] 3.1.5 I2C Write Data
        - [x] 3.1.6 I2C Write Data Repeated Start
        - [x] 3.1.7 I2C Write Data No Stop
    + [x] Read
        - [x] 3.1.8 I2C Read Data
        - [x] 3.1.9 I2C Read Data Repeated Start
    + [x] WriteRead
    + [x] Standard I2C bus speeds (100k/400k)
    + [x] Custom I2C bus speeds (47k–400k)
- [x] [embedded-hal] traits
    - [x] `embedded_hal::i2c::I2c`
    - [x] `embedded_hal_async::i2c::I2c`
    - [x] `embedded_hal::digital::*`

[embedded-hal]: https://github.com/rust-embedded/embedded-hal

## Unsupported features

Currently there is no plan to support the following features.

- UART serial
    + This library interacts with the MCP2221 via USB HID commands, opening the device
      via vendor ID and product ID, but there does not appear to be a straightforward
      way to find a serial port by its USB properties.

      If you need programmatic access to the MCP2221 UART, for eg driver development, 
      I'd recommend you use [`embedded-io`] with your preferred serial library since
      it has blanket implementations for types that implement the `std::io` traits.
      If you enable USB CDC serial number enumeration in the MCP2221 settings, you
      should be able to find the MCP2221 serial port at a stable path.
- `embedded_hal_async::digital::Wait`
    + While the async I2C trait is faked (by just calling the blocking API), the async
      GPIO trait would require busy-waiting, which is an unacceptable tradeoff.
- SMBus (System Management Bus)
    + In practice SMBus is supported as well as the MCP2221 hardware can, via its I2C
      support. But no software support is present in this library for SMBUS-specific
      transfer formats or packet error checking.
- Password protected or permanently locked settings
    + I don't need this feature, and other developers [have locked their MCP2221] by
      accident, so it doesn't seem worthwhile. If you do need this feature, and have
      a device you're willing to risk, [open an issue] and we can work on it.

[have locked their MCP2221]: https://forum.microchip.com/s/topic/a5C3l000000Mb3HEAS/t372487

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
