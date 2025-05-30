use mcp2221_hal::{
    MCP2221,
    i2c::{I2cCancelTransferResponse, I2cSpeed},
};

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
    Write {
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        data: Vec<u8>,
    },
    WriteRead {
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
        #[arg(short, long)]
        read_length: u16,
        write_data: Vec<u8>,
    },
    CheckAddress {
        #[arg(value_parser = crate::util::seven_bit_address)]
        address: u8,
    },
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
