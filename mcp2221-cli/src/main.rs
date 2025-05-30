use clap::Parser;
use mcp2221_hal::MCP2221;

use cli::Commands;

mod analog;
mod cli;
mod i2c;
mod pins;
mod settings;
mod usb;
mod util;

fn main() -> Result<(), mcp2221_hal::Error> {
    let cli = cli::Cli::parse();
    let device = MCP2221::connect_with_vid_and_pid(cli.vid, cli.pid)?;
    match cli.command {
        Commands::Status => println!("{:#?}", device.status()?),
        Commands::Settings(command) => settings::action(&device, command)?,
        Commands::Usb(command) => usb::action(&device, command)?,
        Commands::Dac(command) => analog::dac_action(&device, command)?,
        Commands::Adc(command) => analog::adc_action(&device, command)?,
        Commands::Reset => device.reset()?,
        Commands::I2c(command) => i2c::action(&device, command)?,
        Commands::Pins(command) => pins::action(&device, command)?,
    }
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
