use analog::{AdcCommand, DacCommand};
use cli::Commands;
use i2c::I2cCommand;
use mcp2221_hal::ChangeSramSettings;
use pins::GpModes;
use settings::{SettingsCommand, SettingsType, SettingsWriteCommand};
use usb::UsbInfo;

use mcp2221_hal::MCP2221;
use mcp2221_hal::i2c::{CancelI2cTransferResponse, I2cSpeed};

use clap::Parser;

mod analog;
mod cli;
mod i2c;
mod pins;
mod settings;
mod usb;
mod util;

type McpResult<T> = Result<T, mcp2221_hal::Error>;

fn main() -> McpResult<()> {
    let cli = cli::Cli::parse();
    let device = MCP2221::open_with_vid_and_pid(cli.vid, cli.pid)?;
    match cli.command {
        Commands::Status => println!("{:#?}", device.status()?),
        Commands::Settings(settings_command) => match settings_command {
            SettingsCommand::Read { which } => match which {
                SettingsType::Flash => print_all_flash_data(&device)?,
                SettingsType::Sram => println!("{:#?}", device.sram_read_settings()?),
            },
            SettingsCommand::Write(write_command) => match write_command {
                SettingsWriteCommand::Manufacturer { string } => {
                    device.change_usb_manufacturer(&string)?
                }
                SettingsWriteCommand::Product { string } => device.change_usb_product(&string)?,
            },
        },
        Commands::Usb => {
            println!("{:#?}", UsbInfo::from(&device.usb_device_info()?));
        }
        Commands::Dac(dac_command) => match dac_command {
            DacCommand::Write { flash: true, value } => {
                let mut cs = device.flash_read_chip_settings()?;
                cs.dac_power_up_value = value;
                device.flash_write_chip_settings(cs)?;
            }
            DacCommand::Write {
                flash: false,
                value,
            } => {
                // do sram write
                device.analog_write(value)?;
            }

            DacCommand::Configure {
                flash: true,
                reference,
                vrm_level,
            } => {
                let mut cs = device.flash_read_chip_settings()?;
                cs.dac_reference = reference.into_mcp_vref(vrm_level);
                device.flash_write_chip_settings(cs)?;
            }
            DacCommand::Configure {
                flash: false,
                reference,
                vrm_level,
            } => {
                device.dac_set_reference(reference.into_mcp_vref(vrm_level))?;
            }
        },
        Commands::Adc(adc_command) => match adc_command {
            AdcCommand::Read => println!("{:#?}", device.analog_read()?),
            AdcCommand::Configure {
                flash: false,
                reference,
                vrm_level,
            } => device.adc_set_reference(reference.into_mcp_vref(vrm_level))?,
            AdcCommand::Configure {
                flash: true,
                reference,
                vrm_level,
            } => {
                let mut cs = device.flash_read_chip_settings()?;
                cs.adc_reference = reference.into_mcp_vref(vrm_level);
                device.flash_write_chip_settings(cs)?;
            }
        },
        Commands::Reset => device.reset()?,
        Commands::I2c(i2c_command) => match i2c_command {
            I2cCommand::Cancel => match device.i2c_cancel_transfer()? {
                CancelI2cTransferResponse::MarkedForCancellation => {
                    println!("Transfer marked for cancellation.")
                }
                CancelI2cTransferResponse::NoTransfer => {
                    println!("There was no ongoing I2C transfer to cancel.")
                }
            },
            I2cCommand::Speed { kbps } => device.i2c_set_bus_speed(I2cSpeed::new(kbps * 1000))?,
        },
        Commands::Pins(pins_command) => match pins_command {
            pins::PinsCommand::Read => {
                println!("{:#?}", device.gpio_read()?);
            }
            pins::PinsCommand::SetMode(GpModes {
                flash: true,
                pin_configs,
            }) => {
                let mut gp_settings = device.flash_read_gp_settings()?;
                pin_configs.merge_into_existing(&mut gp_settings);
                device.flash_write_gp_settings(gp_settings)?;
            }
            pins::PinsCommand::SetMode(GpModes {
                flash: false,
                pin_configs,
            }) => {
                let mut sram_settings = device.sram_read_settings()?;
                pin_configs.merge_into_existing(&mut sram_settings.gp_settings);
                device.sram_write_settings(ChangeSramSettings::new().with_gp_modes(
                    sram_settings.gp_settings,
                    Some(sram_settings.chip_settings.dac_reference),
                    Some(sram_settings.chip_settings.adc_reference),
                ))?;
            }
            pins::PinsCommand::Write(pin_values) => {
                device.gpio_write(&pin_values.into())?;
            }
        },
    }

    Ok(())
}

fn print_all_flash_data(device: &mcp2221_hal::MCP2221) -> McpResult<()> {
    println!("{:#?}", device.flash_read_chip_settings()?);
    println!("{:#?}", device.flash_read_gp_settings()?);
    println!(
        r#"USB Manufacturer:  "{}""#,
        device.read_usb_manufacturer()?
    );
    println!(r#"USB Product:       "{}""#, device.read_usb_product()?);
    println!(
        r#"USB Serial Number: "{}""#,
        device.read_usb_serial_number()?
    );
    println!(
        r#"Factory serial:    "{}""#,
        device.read_factory_serial_number()?
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;

    use clap::CommandFactory;

    #[test]
    fn check_cli_debug_asserts() {
        Cli::command().debug_assert();
    }
}
