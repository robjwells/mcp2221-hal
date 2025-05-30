use clap::{Parser, ValueEnum, value_parser};
use mcp2221_hal::{
    MCP2221,
    analog::{VoltageReference, VrmVoltage::*},
};

/// Configure analog output.
#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum DacCommand {
    /// Change the DAC output value.
    Write {
        #[arg(long, default_value = "false")]
        /// Set the DAC output value in flash memory rather than SRAM.
        ///
        /// This will not change the current DAC output value in SRAM,
        /// and will only be observed after resetting the MCP2221.
        flash: bool,
        #[arg(value_parser = value_parser!(u8).range(0..=31))]
        /// New output value, in the range 0..=31.
        value: u8,
    },
    /// Change the DAC voltage reference.
    Configure {
        #[arg(long, default_value = "false")]
        /// Set the DAC configuration in flash memory rather than SRAM.
        ///
        /// This will not change the current DAC configuration in SRAM,
        /// and will only be observed after resetting the MCP2221.
        flash: bool,
        /// Set Vdd or Vrm as the DAC voltage reference.
        reference: VrefSource,
        /// Vrm voltage level.
        ///
        /// Ignored if the reference is Vdd.
        #[arg(default_value = "off")]
        vrm_level: VrmLevel,
    },
}

/// Voltage source to use as the analog reference.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum VrefSource {
    /// Use the supply voltage Vdd.
    Vdd,
    /// Use the internal Vrm voltage reference.
    Vrm,
}

impl VrefSource {
    pub(crate) fn into_mcp_vref(
        self,
        vrm_level: VrmLevel,
    ) -> mcp2221_hal::analog::VoltageReference {
        match (self, vrm_level) {
            (VrefSource::Vdd, _) => VoltageReference::Vdd,
            (VrefSource::Vrm, VrmLevel::_1V) => VoltageReference::Vrm(V1_024),
            (VrefSource::Vrm, VrmLevel::_2V) => VoltageReference::Vrm(V2_048),
            (VrefSource::Vrm, VrmLevel::_4V) => VoltageReference::Vrm(V4_096),
            (VrefSource::Vrm, VrmLevel::Off) => VoltageReference::Vrm(Off),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum VrmLevel {
    /// 1.024V
    _1V,
    /// 2.048V
    _2V,
    /// 4.096V
    _4V,
    /// Disable the Vrm reference.
    ///
    /// In practice this produces a voltage just above the 0 value voltage of the
    /// other references.
    Off,
}

/// Read or configure analog input.
#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum AdcCommand {
    /// Read the three channels of the 10-bit ADC.
    ///
    /// The reading for a channel will be `None` if the corresponding pin is not
    /// configured as an analog input.
    Read,
    /// Change the ADC voltage reference.
    Configure {
        #[arg(long, default_value = "false")]
        /// Set the ADC configuration in flash memory rather than SRAM.
        ///
        /// This will not change the current ADC configuration in SRAM,
        /// and will only be observed after resetting the MCP2221.
        flash: bool,
        /// Set Vdd or Vrm as the ADC voltage reference.
        reference: VrefSource,
        /// Vrm voltage level.
        ///
        /// Ignored if the reference is Vdd.
        #[arg(default_value = "off")]
        vrm_level: VrmLevel,
    },
}

pub(crate) fn dac_action(device: &MCP2221, command: DacCommand) -> Result<(), mcp2221_hal::Error> {
    match command {
        DacCommand::Write { flash: true, value } => {
            let mut cs = device.flash_read_chip_settings()?;
            cs.dac_value = value;
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
            device.analog_set_output_reference(reference.into_mcp_vref(vrm_level))?;
        }
    }
    Ok(())
}

pub(crate) fn adc_action(device: &MCP2221, command: AdcCommand) -> Result<(), mcp2221_hal::Error> {
    match command {
        AdcCommand::Read => println!("{:#?}", device.analog_read()?),
        AdcCommand::Configure {
            flash: false,
            reference,
            vrm_level,
        } => device.analog_set_input_reference(reference.into_mcp_vref(vrm_level))?,
        AdcCommand::Configure {
            flash: true,
            reference,
            vrm_level,
        } => {
            let mut cs = device.flash_read_chip_settings()?;
            cs.adc_reference = reference.into_mcp_vref(vrm_level);
            device.flash_write_chip_settings(cs)?;
        }
    }
    Ok(())
}
