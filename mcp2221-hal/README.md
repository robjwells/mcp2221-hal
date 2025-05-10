# Deviations from the datasheet

- Handling of DAC and ADC source (1 = VRM, 0 = VDD).
    The datasheet has a couple of occurrences where Vdd = 1 but the driver always
    treats Vrm = 1. It appears these are typos in the datasheet.
- Configuring DAC/ADC reference as Vdd always sets the Vrm level to Off.
- I2C transfer cancellation only issued if status shows I2C engine is not idle.
    (The MCP2221 seems to put the I2C engine _into_ a busy state is a cancellation is
    issued while the I2C engine is already idle. I've never seen the 0x11 "already idle"
    response from the MCP2221 so we check this ourselves and skip if appropriate.)
