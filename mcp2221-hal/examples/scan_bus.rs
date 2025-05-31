//! # I2C bus-scanning example
//!
//! This attempts to find out which devices are connected to the I2C bus
//! by sending each possible address as if it were the start of a write,
//! and then ending the write without having written anything.
//!
//! (I2C addresses are the upper 7 bits of a byte, and the lowest bit of
//! the byte is the Read/_Write (not-write) bit, 1 for reads and 0 for
//! writes.)
//!
//! If a device acknowledges this address, the address is printed (in
//! hexadecimal). But if a connected device does not respond to writes,
//! then it won't appear in the output.
//!
//! (The scan can't be done with reads, because it has the potential to
//! lock up the bus if the device puts a 0 on the bus by pulling the
//! line low, with no-one to read it by clocking the data out.)
//!
//! 00 is a special address, the "general call" address, used to address
//! all devices on the bus. But not every device responds to general
//! calls. If it appears in the output, at least one device on the
//! bus acknowledged the general call address.
use mcp2221_hal::MCP2221;

fn main() -> Result<(), mcp2221_hal::Error> {
    let device = MCP2221::connect()?;

    println!("Scanning the I2C bus...\n");
    for address in 0..128u8 {
        if start_line(address) {
            print!("{address:02X}:  ");
        }
        if device.i2c_check_address(address)? {
            print!("{address:02X} ");
        } else {
            print!("-- ")
        }
        if end_line(address) {
            println!();
        }
    }
    println!("{}", TRAILER);

    Ok(())
}

fn start_line(n: u8) -> bool {
    n % 16 == 0
}

fn end_line(n: u8) -> bool {
    n % 16 == 15
}

const TRAILER: &str = r#"
A two-digit number is an address (in hex) that was acknowledged.

00 is the general call address. If it appears, at least one
device on the bus responds to general calls."#;
