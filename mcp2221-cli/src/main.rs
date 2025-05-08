use analog::DacCommand;
use cli::Commands;
use settings::{SettingsCommand, SettingsType, SettingsWriteCommand};
use usb::UsbInfo;

use clap::Parser;

mod analog;
mod cli;
mod settings;
mod usb;
mod util;

type McpResult<T> = Result<T, mcp2221_hal::error::Error>;

fn main() -> McpResult<()> {
    let cli = cli::Cli::parse();
    let mut device = mcp2221_hal::MCP2221::open_with_vid_and_pid(cli.vid, cli.pid)?;
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
            DacCommand::Write { value } => device.set_dac_output_value(value)?,

            DacCommand::Configure {
                reference,
                vrm_level,
            } => device.configure_dac_source(reference.into_mcp_vref(vrm_level))?,
        },
        Commands::Reset => device.reset_chip()?,
    }

    Ok(())
}
