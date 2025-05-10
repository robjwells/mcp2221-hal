#[derive(Debug, clap::Parser)]
#[command(flatten_help = true)]
pub(crate) enum I2cCommand {
    /// Set the I2C bus clock speed
    Speed {
        speed: I2cSpeed
    },
    /// Cancel the current I2C transfer and attempt to free the bus.
    Cancel,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum I2cSpeed {
    /// 400kbps "fast" mode
    Fast,
    /// 100kbps "standard" mode
    Standard,
}

impl From<I2cSpeed> for mcp2221_hal::i2c::I2cSpeed {
    fn from(value: I2cSpeed) -> mcp2221_hal::i2c::I2cSpeed {
        match value {
            I2cSpeed::Fast => mcp2221_hal::i2c::I2cSpeed::Fast_400kbps,
            I2cSpeed::Standard => mcp2221_hal::i2c::I2cSpeed::Standard_100kbps,
        }
    }
}
