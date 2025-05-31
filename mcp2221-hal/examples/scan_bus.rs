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
