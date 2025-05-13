use analog::{AdcCommand, DacCommand};
use cli::Commands;
use i2c::I2cCommand;
use settings::{SettingsCommand, SettingsType, SettingsWriteCommand};
use usb::UsbInfo;

use mcp2221_hal::MCP2221;
use mcp2221_hal::i2c::CancelI2cTransferResponse;

use clap::Parser;

mod analog;
mod cli;
mod i2c;
mod settings;
mod usb;
mod util;

type McpResult<T> = Result<T, mcp2221_hal::error::Error>;

fn main() -> McpResult<()> {
    let cli = cli::Cli::parse();
    let mut device = MCP2221::open_with_vid_and_pid(cli.vid, cli.pid)?;
    match cli.command {
        Commands::Status => println!("{:#?}", device.status()?),
        Commands::Settings(settings_command) => match settings_command {
            SettingsCommand::Read { which } => match which {
                SettingsType::Flash => println!("{:#?}", device.read_flash_data()?),
                SettingsType::Sram => println!("{:#?}", device.get_sram_settings()?),
            },
            SettingsCommand::Write(write_command) => match write_command {
                SettingsWriteCommand::Manufacturer { string } => {
                    device.write_usb_manufacturer_descriptor(&string)?
                }
                SettingsWriteCommand::Product { string } => {
                    device.write_usb_product_descriptor(&string)?
                }
            },
        },
        Commands::Usb => {
            println!("{:#?}", UsbInfo::from(&device.usb_device_info()?));
        }
        Commands::Dac(dac_command) => match dac_command {
            DacCommand::Write { flash: true, value } => {
                // TODO: This is querying all the flash data, when we only need the
                // chip settings. Perhaps break up the read_flash_data() method?
                let mut cs = device.read_flash_data()?.chip_settings;
                cs.dac_power_up_value = value;
                device.write_chip_settings_to_flash(cs)?;
            }
            DacCommand::Write {
                flash: false,
                value,
            } => {
                // do sram write
                device.set_dac_output_value(value)?;
            }

            DacCommand::Configure {
                flash: true,
                reference,
                vrm_level,
            } => {
                // TODO: This is querying all the flash data, when we only need the
                // chip settings. Perhaps break up the read_flash_data() method?
                let mut cs = device.read_flash_data()?.chip_settings;
                cs.dac_reference = reference.into_mcp_vref(vrm_level);
                device.write_chip_settings_to_flash(cs)?;
            }
            DacCommand::Configure {
                flash: false,
                reference,
                vrm_level,
            } => {
                device.configure_dac_source(reference.into_mcp_vref(vrm_level))?;
            }
        },
        Commands::Adc(adc_command) => match adc_command {
            AdcCommand::Read => println!("{:#?}", device.read_adc()?),
            AdcCommand::Configure {
                flash: false,
                reference,
                vrm_level,
            } => device.configure_adc_source(reference.into_mcp_vref(vrm_level))?,
            AdcCommand::Configure {
                flash: true,
                reference,
                vrm_level,
            } => {
                let mut cs = device.read_flash_data()?.chip_settings;
                cs.adc_reference = reference.into_mcp_vref(vrm_level);
                device.write_chip_settings_to_flash(cs)?;
            }
        },
        Commands::Reset => device.reset_chip()?,
        Commands::I2c(i2c_command) => match i2c_command {
            I2cCommand::Cancel => match device.cancel_i2c_transfer()? {
                CancelI2cTransferResponse::MarkedForCancellation => {
                    println!("Transfer marked for cancellation.")
                }
                CancelI2cTransferResponse::NoTransfer => {
                    println!("There was no ongoing I2C transfer to cancel.")
                }
            },
            I2cCommand::Speed { speed } => device.set_i2c_bus_speed(speed.into())?,
        },
    }

    Ok(())
}
