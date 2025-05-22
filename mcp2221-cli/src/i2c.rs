#[derive(Debug, clap::Parser)]
pub(crate) enum I2cCommand {
    /// Set the I2C bus clock speed in kbps.
    ///
    /// Standard I2C bus speeds are 400 kbps and 100 kbps. The MCP2221 can use bus
    /// speeds from 47 kbps to 400 kbps.
    Speed {
        /// Desired I2C bus speed in kbps.
        kbps: u32,
    },
    /// Cancel the current I2C transfer and attempt to free the bus.
    Cancel,
    Read {
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        length: u16,
    },
}
