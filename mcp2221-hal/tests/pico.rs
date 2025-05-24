//! Test against the Pico target device.
//!
//! Tests have to be run serially, because attempting to acquire the USB device in
//! two threads at once will fail, and the underlying HidApi struct cannot be shared
//! between threads.
use mcp2221_hal::{
    Error, MCP2221,
    gpio::{Input, Output, Pins},
};

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c::{I2c, Operation};

const ADDRESS: u8 = 0x26;

/// Reads 10 sequential bytes from the Pico.
#[test]
fn pico_eh_i2c_read() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    let mut buf = [0u8; 10];
    device.read(ADDRESS, &mut buf)?;
    assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    Ok(())
}

/// Reads 10 sequential bytes from the Pico into 2 buffers.
#[test]
fn pico_eh_i2c_read_transaction() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    let mut buf_1 = [0u8; 5];
    let mut buf_2 = [0u8; 5];
    device.transaction(
        ADDRESS,
        &mut [Operation::Read(&mut buf_1), Operation::Read(&mut buf_2)],
    )?;
    assert_eq!(buf_1, [0, 1, 2, 3, 4]);
    assert_eq!(buf_2, [5, 6, 7, 8, 9]);
    Ok(())
}

/// Writes [0x20, 0x0A] to the Pico, reads 10 sequential bytes starting at 0x20.
#[test]
fn pico_eh_i2c_writeread() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    let mut buf = [0u8; 10];
    device.write_read(ADDRESS, &[0x20, 10], &mut buf)?;
    assert_eq!(
        buf,
        [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29]
    );
    Ok(())
}

/// Writes [0x30, 0x0A] to the Pico from two buffers, reads 10 sequential bytes
/// starting at 0x20 into two buffers.
#[test]
fn pico_eh_i2c_writeread_transaction() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    let mut buf_1 = [0u8; 5];
    let mut buf_2 = [0u8; 5];
    device.transaction(
        ADDRESS,
        &mut [
            Operation::Write(&[0x30]),
            Operation::Write(&[10]),
            Operation::Read(&mut buf_1),
            Operation::Read(&mut buf_2),
        ],
    )?;
    assert_eq!(buf_1, [0x30, 0x31, 0x32, 0x33, 0x34]);
    assert_eq!(buf_2, [0x35, 0x36, 0x37, 0x38, 0x39]);
    Ok(())
}

/// Writes 5 bytes to the Pico.
///
/// Here we're just checking that no errors occur. The written bytes have to be
/// checked in the Pico RTT output.
#[test]
fn pico_eh_i2c_write() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    device.write(ADDRESS, &[0, 1, 2, 3, 4])?;
    Ok(())
}

/// Writes 6 bytes to the Pico from three buffers.
///
/// Here we're just checking that no errors occur. The written bytes have to be
/// checked in the Pico RTT output.
#[test]
fn pico_eh_i2c_write_transaction() -> Result<(), Error> {
    let mut device = MCP2221::open()?;
    device.transaction(
        ADDRESS,
        &mut [
            Operation::Write(&[0x40, 0x41]),
            Operation::Write(&[0x50, 0x51]),
            Operation::Write(&[0x60, 0x61]),
        ],
    )?;
    Ok(())
}

/// Check the Pico responds to its address.
#[test]
fn pico_eh_i2c_check_address() -> Result<(), Error> {
    let device = MCP2221::open()?;
    assert!(device.i2c_check_address(ADDRESS)?);
    Ok(())
}

/// Check that the two tied GP pins can see the other's output.
#[test]
fn pico_tied_gpio_pins() -> Result<(), Error> {
    let device = MCP2221::open()?;
    let Pins { gp1, gp2, .. } = device.take_pins().unwrap();

    let mut gp1_in: Input = gp1.try_into()?;
    let mut gp2_out: Output = gp2.try_into()?;
    gp2_out.set_high()?;
    assert!(gp1_in.is_high()?, "{:?}", device.gpio_read());
    gp2_out.set_low()?;
    assert!(gp1_in.is_low()?);

    // Swap gp1 and gp2 directions.
    let mut gp2_in = gp2_out.try_into_input()?;
    let mut gp1_out = gp1_in.try_into_output()?;
    gp1_out.set_high()?;
    assert!(gp2_in.is_high()?);
    gp1_out.set_low()?;
    assert!(gp2_in.is_low()?);

    Ok(())
}
