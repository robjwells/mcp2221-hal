use clap::Parser;
use mcp2221_hal::MCP2221;

use modes::GpModes;
use values::PinValues;

mod modes;
mod values;

#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum PinsCommand {
    /// Read current GPIO pin values.
    ///
    /// For GP pins set as GPIO inputs, the logic level read is the level on that pin.
    /// For outputs, it is the set output level.
    ///
    /// GP pins that are not in GPIO mode are listed as None.
    Read,
    /// Set GPIO pin direction and output levels.
    ///
    /// Note this does not put pins into GPIO mode, only configure pins that are
    /// already set to GPIO mode. Use the set-mode subcommand to put the pins into
    /// GPIO mode if needed.
    #[command(flatten_help = true)]
    Write(PinValues),
    /// Set the mode for each of the GP pins.
    ///
    /// Each pin supports GPIO (digital) input and output, as well as pin-specific
    /// alternate modes. If the pin is set as a GPIO output, its output value is also
    /// set.
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
