//! Use of the MCP2221 with embedded-hal to read from an SHT4x sensor over I2C.
//!
//! (If you've used I2C via embedded-hal before, skip this as it's very basic.)
//!
//! The [SHT40, SHT41, SHT43 and SHT45][sht] are temperature and humidity sensors made
//! by Sensirion that share an I2C interface. Here we demonstrate basic communication
//! with the sensor over I2C.
//!
//! You may wish to read this example alongside section 4 of the [SHT4x datasheet][sht].
//!
//! [sht]: https://sensirion.com/products/catalog/SHT40
//! [datasheet]: https://sensirion.com/media/documents/33FD6951/67EB9032/HT_DS_Datasheet_SHT4x_5.pdf
use std::time::Duration;

use mcp2221_hal::MCP2221;

/// Most SHT4x parts have the same 0x44 address.
///
/// Consult the "device overview" on p1 of the datasheet or section 10 for details,
/// as your particular part may have the address 0x45 or 0x46.
const ADDRESS: u8 = 0x44;

/// SHT4x command code for a high precision (high repeatability) reading.
const HIGH_PRECISION_READING: u8 = 0xFD;

/// SHT4x command code for reading the sensor serial number.
const SERIAL_NUMBER: u8 = 0x89;

fn main() -> Result<(), mcp2221_hal::Error> {
    // Connect to the MCP2221 with the default VID and PID.
    let mut device = MCP2221::connect()?;

    // We'll use the embedded-hal trait methods, which are the recommended way of
    // using the MCP2221 for I2C communication. These allow you to use your code with
    // any other HAL, for example with the rp-rs hal for the RP2040.
    //
    // You should think of the `i2c_read`, `i2c_write` and `i2c_write_read` methods on
    // the MCP2221 struct as implementation details that allow you to use the
    // cross-platform embedded-hal methods. Note that the driver also implements
    // embedded_hal_async::i2c::I2c (delegating to the blocking methods).
    use embedded_hal::i2c::I2c;

    // We'll instruct the SHT4x to start a high-precision reading.
    device.write(ADDRESS, &[HIGH_PRECISION_READING])?;

    // Now we delay, because the SHT4x becomes unresponsive during readings, which can
    // cause your hal to return an error (address NACK).
    std::thread::sleep(Duration::from_millis(10));

    // SHT4x sensors return six bytes: a u16 temperature reading, then a u8 CRC of the
    // temperature, then a u16 humidity reading, then a u8 CRC of the humidity. We
    // create an array to hold that response. The read buffer's length determines how
    // many bytes the controller (the MCP2221) will attempt to read from the target
    // (the SHT4x sensor).
    let mut read_buffer = [0u8; 6];
    device.read(ADDRESS, &mut read_buffer)?;

    // Let's pick out the data bytes and ignore the error-checking codes, just for the
    // purposes of this example.
    let [t0, t1, _, h0, h1, _] = read_buffer;
    // The SHT4x transmits u16 values most-significant byte (MSB) first.
    let temp_reading = u16::from_be_bytes([t0, t1]);
    let humidity_reading = u16::from_be_bytes([h0, h1]);

    // Quite often with I2C targets, you'll perform a write-read to fetch some value
    // already available to the target (rather than manually delaying as we did
    // above). A write-read involves performing a write without a final Stop
    // condition, and then immediately performing a read (you'll see this referred
    // to as a read with a Repeated-Start).
    //
    // To demonstrate, we can fetch the SHT4x sensor's serial number. It's 6 bytes
    // again, with the same [d, d, CRC, d, d, CRC] layout.
    let mut serial_buf = [0u8; 6];
    device.write_read(ADDRESS, &[SERIAL_NUMBER], &mut serial_buf)?;
    let [s0, s1, _, s2, s3, _] = serial_buf;
    let serial = u32::from_be_bytes([s0, s1, s2, s3]);

    println!(
        "Sensor {serial}:    {:.2}°C    {:.2}% humidity",
        celsius_from_reading(temp_reading),
        humidity_from_reading(humidity_reading)
    );

    Ok(())
}

/// Convert an SHT4x temperature reading to Celsius.
///
/// This is taken from section 4.6 of the SHT4x datasheet. It doesn't matter for the
/// purposes of understanding the use of the MCP2221!
fn celsius_from_reading(reading: u16) -> f32 {
    // Easiest to work with floats on your development machine. But note that not all
    // microcontrollers have floating-point hardware.
    let reading = f32::from(reading);
    -45.0 + 175.0 * (reading / 65_535.0)
}

/// Convert an SHT4x humidity reading to % relative humidity (%RH).
fn humidity_from_reading(reading: u16) -> f32 {
    let reading = f32::from(reading);
    -6.0 + 125.0 * (reading / 65_535.0)
}
