use mcp2221_hal::{
    MCP2221,
    i2c::{I2cCancelTransferResponse, I2cSpeed},
};

#[derive(Debug, clap::Parser)]
pub(crate) enum I2cCommand {
    /// Read a given number of bytes from an I2C address.
    Read {
        /// I2C target 7-bit address.
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        /// Number of bytes to read from the target.
        length: u16,
    },
    /// Write bytes (given in hex) to an I2C address.
    Write {
        /// I2C target 7-bit address.
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        /// Bytes (in hex) to write to the target.
        #[arg(last = true, num_args = 1..=65_535, value_parser = crate::util::u8_from_hex)]
        data: Vec<u8>,
    },
    /// Write bytes (given in hex) to an I2C address, and read back the specified number
    /// of bytes.
    ///
    /// No Stop condition occurs between the write and the read.
    WriteRead {
        /// I2C target 7-bit address.
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        /// Number of bytes to read from the target.
        read_length: u16,
        /// Bytes (in hex) to write to the target.
        #[arg(last = true, num_args = 1..=65_535, value_parser = crate::util::u8_from_hex)]
        write_data: Vec<u8>,
    },
    /// Check if a device acknowledges an address.
    CheckAddress {
        /// I2C target 7-bit address.
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
    },
    /// Set the I2C bus clock speed in kbit/s.
    ///
    /// Standard I2C bus speeds are 400 kbit/s and 100 kbit/s. The MCP2221 can use bus
    /// speeds from 47 kbit/s to 400 kbit/s.
    Speed {
        /// Desired I2C bus speed in kbit/s.
        kbps: u32,
    },
    /// Cancel the current I2C transfer and attempt to free the bus.
    Cancel,
}

pub(crate) fn action(device: &MCP2221, command: I2cCommand) -> Result<(), mcp2221_hal::Error> {
    match command {
        I2cCommand::Cancel => match device.i2c_cancel_transfer()? {
            I2cCancelTransferResponse::MarkedForCancellation => {
                println!("Transfer marked for cancellation.")
            }
            I2cCancelTransferResponse::NoTransfer => {
                println!("There was no ongoing I2C transfer to cancel.")
            }
            I2cCancelTransferResponse::Done => {
                println!("Transfer cancelled.");
            }
        },
        I2cCommand::Speed { kbps } => device.i2c_set_bus_speed(I2cSpeed::new(kbps * 1000))?,
        I2cCommand::Read { address, length } => {
            // Length as u16 ensures it's within the MCP2221's limits, even if
            // we immediately convert it to a usize.
            let mut data = vec![0; length as usize];
            device.i2c_read(address, data.as_mut_slice())?;
            print_bytes(&data);
        }
        I2cCommand::Write { address, data } => {
            device.i2c_write(address, data.as_slice())?;
        }
        I2cCommand::CheckAddress { address } => match device.i2c_check_address(address) {
            Ok(true) => println!("Device found at {address:#04X}"),
            Ok(false) => println!("No device found at {address:#04X}"),
            Err(e) => {
                eprintln!("{e}")
            }
        },
        I2cCommand::WriteRead {
            address,
            read_length,
            write_data,
        } => {
            // read_length as u16 ensures it's within the MCP2221's limits, even if
            // we immediately convert it to a usize.
            let mut read_data = vec![0; read_length as usize];
            device.i2c_write_read(address, &write_data, &mut read_data)?;
            print_bytes(&read_data);
        }
    }
    Ok(())
}

fn print_bytes(data: &[u8]) {
    eprintln!("{} bytes read", data.len());
    for chunk in data.chunks(8) {
        for byte in chunk {
            print!("{byte:02X} ");
        }
        println!();
    }
}
