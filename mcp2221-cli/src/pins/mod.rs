use clap::Parser;
use mcp2221_hal::MCP2221;

use modes::GpModes;
use values::PinValues;

mod modes;
mod values;

#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum PinsCommand {
    Read,
    #[command(flatten_help = true)]
    Write(PinValues),
    #[command(flatten_help = true)]
    SetMode(GpModes),
}

pub(crate) fn action(device: &MCP2221, command: PinsCommand) -> Result<(), mcp2221_hal::Error> {
    match command {
        PinsCommand::Read => {
            println!("{:#?}", device.gpio_read()?);
        }
        PinsCommand::SetMode(GpModes {
            flash: true,
            pin_configs,
        }) => {
            let mut gp_settings = device.flash_read_gp_settings()?;
            pin_configs.merge_into_existing(&mut gp_settings);
            device.flash_write_gp_settings(gp_settings)?;
        }
        PinsCommand::SetMode(GpModes {
            flash: false,
            pin_configs,
        }) => {
            let (_, mut gp_settings) = device.sram_read_settings()?;
            pin_configs.merge_into_existing(&mut gp_settings);
            device.sram_write_gp_settings(gp_settings)?;
        }
        PinsCommand::Write(pin_values) => {
            device.gpio_write(&pin_values.into())?;
        }
    }
    Ok(())
}
