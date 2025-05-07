# From Howard Long

- Write Chip / Configure Device in the MCP2221 Utility, with Vref = VDD.
- GP1 = ADC1; ADC Vref = Vdd.
- GP3 = DAC2; DAC Vref = Vdd.

Data written to the device:

```
00 0xB1     -- Write Flash Data
01 0x00     -- Write Chip Settings
02 0x3C     -- CDC / chip security
    0b 0011 1100
03 0x12     -- Clock output divider
04 0x1F     -- DAC settings
    0b 0001 1111
        Bit 5 = 0
        Datasheet T3-12 says Vrm.
05 0x60
    0b 0110 0000
        Bit 2 = 0
        Datasheet T3-12 says Vdd.
06 0xD8
07 0x04
08 0xDD
09 0x00
10 0x80
11 0x32
```
