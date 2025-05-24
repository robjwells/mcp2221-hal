use embedded_hal::i2c::I2c as BlockingI2c;
use embedded_hal_async::i2c::I2c as AsyncI2c;

impl AsyncI2c for crate::MCP2221 {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        BlockingI2c::transaction(self, address, operations)
    }

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        BlockingI2c::read(self, address, read)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        BlockingI2c::write(self, address, write)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        BlockingI2c::write_read(self, address, write, read)
    }
}
