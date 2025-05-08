use clap::{Parser, ValueEnum, value_parser};
use mcp2221_hal::analog::{VoltageReference, VrmVoltage::*};

#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum DacCommand {
    /// Change the DAC output value.
    Write {
        #[arg(value_parser = value_parser!(u8).range(0..=31))]
        /// New output value, in the range 0..=31.
        value: u8,
    },
    Configure {
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
