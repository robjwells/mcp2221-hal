% mcp2221-cli(1) 2025-06-14 0.1.0

NAME
====

mcp2221-cli - interact with an MCP2221 USB to I2C and GPIO converter

SYNOPSIS
========

`mcp2221-cli [OPTIONS] i2c COMMAND`
: I2C communication

`mcp2221-cli [OPTIONS] pins COMMAND`
: Pin configuration and GPIO operation

`mcp2221-cli [OPTIONS] adc COMMAND`
: Analog input

`mcp2221-cli [OPTIONS] dac COMMAND`
: Analog output

`mcp2221-cli [OPTIONS] settings <sram|flash>`
: Read device settings

`mcp2221-cli [OPTIONS] status`
: Read device status

`mcp2221-cli [OPTIONS] usb COMMAND`
: Read USB information from the host and set device USB configuration

`mcp2221-cli [OPTIONS] reset`
: Reset the MCP2221

OPTIONS
=======

`-v, --vid`
:   USB vendor ID (VID) of the MCP2221, if it is not the default (0x4D8)

`-p, --pid`
:   USB product ID (PID) of the MCP2221, if it is not the default (0xDD)

COMMANDS
========

I2C
---

Communicate on or configure the I2C bus. Addresses and data bytes should be
given in hexadecimal. Read and write transfers must be at least 1 byte long
and at most 65,535 bytes.

`i2c read ADDRESS LENGTH`
:   Read LENGTH bytes from a target at ADDRESS

`i2c write ADDRESS -- DATA...`
:   Write DATA bytes to the target at ADDRESS.

    DATA must be at least 1 byte and at most 65,535 bytes, given in hexadecimal
    and separated by a space.

`i2c write-read ADDRESS READ_LENGTH -- WRITE_DATA...`
:   Write WRITE_DATA bytes to the target at ADDRESS, issue a repeated-Start, and
    read READ_LENGTH bytes from the target. No Stop is issued between the write
    and the read.

`i2c check-address ADDRESS`
:   Check if a target acknowledges ADDRESS by performing a zero-length write.

`i2c speed KBPS`
:   Set the I2C bus speed to KBPS kbit/s. KBPS should be a decimal integer
    between 47 and 400.

`i2c cancel`
:   Cancel the current I2C transfer and attempt to free the bus.

PINS
----

General purpose (GP) pin configuration, including digital (GPIO) operation.

`pins read`
:   Read the logic level and direction of the GP pins. Only pins that are set
    to GPIO mode will show a value.

`pins write [--flash] <--gp0 MODE | --gp1 MODE | --gp2 MODE | --gp3 MODE>`
:   Change the direction and logic level of the GP pins. Pins will only be
    affected if they are set to GPIO mode.

    `--flash`
    :   Change GPIO configuration in flash memory, affecting power-up settings.

        The behaviour of the GPIO pins will not be affected until the device is
        reset.

    `MODE`
    :   GPIO mode for the given GP pin.

        `output-high`
        :   Output a *high* logic level

        `output-low`
        :   Output a *low* logic level

        `input`
        :   Set pin as an input

`pins set-mode [--flash] <--gp0 GP0 | --gp1 GP1 | --gp2 GP2 | --gp3 GP3>`
:   Change the mode of the given GP pins, allowing the use of functions that
    are specific to each pin. All pins support the following GPIO modes, which
    are not repeated below:

        gpio-output-high : GPIO mode, set to output a *high* logic level

        gpio-output-low  : GPIO mode, set to output a *low* logic level

        gpio-input       : GPIO mode, set to input

    `--flash`
    :   Change pin configuration in flash memory, affecting power-up settings.

        The behaviour of the pins will not be affected until the device is reset.

    `GP0`
    :   Mode of pin GP0

        `uart-receive-led`
        :   UART traffic received indicator (LED_URX), will pulse low for a few
            milliseconds to provide a visual indication of the UART receive traffic

        `usb-suspend-state`
        :   USB Suspend state (SSPND), low when the USB host has issued the Suspend
            state, high after the Resume state is achieved

    `GP1`
    :   Mode of pin GP1

        `clock-output`
        :   Digital clock output (CLK_OUT or CLKR)

        `analog-input`
        :   ADC channel 1

        `uart-transmit-led`
        :   UART traffic transmitted indicator (LED_UTX), will pulse low for a few
            milliseconds to provide a visual indication of the UART transmit traffic

        `interrupt`
        :   Detects rising or falling edges and sets a flag in the chip status

    `GP2`
    :   Mode of pin GP2

        `usb-device-configured`
        :   USB device-configured status (USBCFG), starts low after reset and goes
            high once the device successfully configures to the USB

        `analog-input`
        :   ADC channel 2

        `analog-output`
        :   DAC output (single channel shared with GP3)

    `GP3`
    :   Mode of pin GP3

        `i2c-led`
        :   Indicates I2C activity (LED_I2C), this pin will pulse low for a few
            milliseconds to provide a visual indication of the I2C traffic

        `analog-input`
        :   ADC channel 3

        `analog-output`
        :   DAC output (single channel shared with GP2)

ADC
---

Analog input commands.

`adc read`
:   Get the current readings from the analog-to-digital converter.
    Only pins that are configured as analog inputs will show a value.

`adc configure [--flash] REFERENCE [VRM_LEVEL]`
:   Set the voltage reference for the analog-to-digital converter.

    The MCP2221's analog reference behaviour can be surprising (buggy). See QUIRKS.

    `--flash`
    :   Change ADC configuration in flash memory, affecting power-up settings.

        The behaviour of the ADC will not be affected until the device is reset.

    `REFERENCE`
    :   Voltage reference source for the ADC.

        `vdd`
        :   Use the supply voltage as the voltage reference

        `vrm`
        :   Use the internal voltage reference module

    `[VRM_LEVEL]`
    :   Voltage of the Vrm voltage reference, optional if REFERENCE is Vdd.

        `1v`
        : 1.024V Vrm reference

        `2v`
        : 2.048V Vrm reference

        `4v`
        : 4.048V Vrm reference (only if Vdd is above 4V)

        `off`
        :   Equivalent to setting REFERENCE to Vdd

DAC
---

Analog output commands.

`dac write VALUE`
:   Change the DAC output to VALUE, a 5-bit integer (0 through 31).

`dac configure [--flash] REFERENCE [VRM_LEVEL]`
:   Set the voltage reference for the digital-to-analog converter.

    The MCP2221's analog reference behaviour can be surprising (buggy). See QUIRKS.

    `--flash`
    :   Change DAC configuration in flash memory, affecting power-up settings.

        The behaviour of the DAC will not be affected until the device is reset.

    `REFERENCE`
    :   Voltage reference source for the DAC.

        `vdd`
        :   Use the supply voltage as the voltage reference

        `vrm`
        :   Use the internal voltage reference module

    `[VRM_LEVEL]`
    :   Voltage of the Vrm voltage reference, optional if REFERENCE is Vdd.

        `1v`
        : 1.024V Vrm reference

        `2v`
        : 2.048V Vrm reference

        `4v`G
        : 4.048V Vrm reference (only if Vdd is above 4V)

        `off`
        :   Equivalent to setting REFERENCE to Vdd

SETTINGS
--------

`settings <sram|flash>`
:   Read the GP pin and chip settings from either SRAM or flash.

    The SRAM settings determine the current behaviour of the MCP2221, while the
    flash settings determine the initial behaviour of the device after reset.

    Note that the GP settings read from SRAM may not reflect the current status of
    the GPIO pins. See QUIRKS.

STATUS
------

`status`
:   > Reads the current status of the MCP2221.

    The structure is mostly I2C settings, along with the interrupt-detected flag,
    raw ADC values, and hardware and firmware revisin numbers.

USB
---

`info`
:   > Read USB information from the host (from `usbhid`)

`set manufacturer STRING`
:   Set the USB manufacturer descriptor to STRING, which must be fewer than 60 bytes
    when encoded as UTF-16.

`set product STRING`
:   Set the USB product descriptor to STRING, which must be fewer than 60 bytes
    when encoded as UTF-16.

`set serial STRING`
:   Set the USB serial number descriptor to STRING, which must be fewer than 60 bytes when encoded as UTF-16.

`set cdc-enumeration <true|false>`
:   Set whether the USB serial number string will be presented during enumeration of
    the CDC device (the USB serial converter). If enabled, this will give the MCP2221
    serial port a stable name.

`set vid`
:   Set the USB vendor ID (VID).

    If changed from the default, you must use the `--vid` option with the CLI.

`set pid`
:   Set the USB product ID (PID).

    If changed from the default, you must use the `--pid` option with the CLI.

RESET
-----

`reset`
:   > Reset the MCP2221.

QUIRKS
======

The MCP2221, at least the MCP2221A rev A.6 with the 1.2 firmware, has some surprising
or buggy behaviour. These behaviours are described in detail in the relevant sections
of the mcp2221-hal documentation, but in summary:

-   Settings read from SRAM may not reflect the current status of pins in GPIO
    mode. It is also possible for the ADC and DAC Vrm levels to be set to "off"
    without this being reflected in the read SRAM settings (though the bug that
    causes this cannot triggered by mcp2221-cli or mcp2221-hal).

-   Setting the DAC reference to Vrm with a level of "off" produces an output
    of about 0V.

-   Setting either DAC or ADC voltage reference in flash to Vrm (at any level)
    will lead to the relevant peripheral acting as if it was set to Vrm with a
    level of "off".

VERSION
=======

0.1.0

HOMEPAGE
========

https://github.com/robjwells/mcp2221-hal
:   Source repository and bug tracker

https://docs.rs/mcp2221-hal
:   Documentation of the underlying mcp2221-hal crate

Please report bugs and feature requests to the bug tracker. If the behaviour
of the CLI is in any way unclear or confusing, please also open an issue.

AUTHORS
=======

Rob Wells <rob@robjwells.com>
