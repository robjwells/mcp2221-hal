use clap::{Parser, ValueEnum};
use mcp2221_hal::gpio::{GpioChanges, GpioDirection, LogicLevel};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GpioSetting {
    /// GPIO output, set high.
    #[value(aliases = ["high"])]
    OutputHigh,
    /// GPIO output, set low.
    #[value(aliases = ["low"])]
    OutputLow,
    /// GPIO input.
    #[value(aliases = ["in"])]
    Input,
}

impl GpioSetting {
    fn level(&self) -> Option<LogicLevel> {
        match self {
            GpioSetting::OutputHigh => Some(LogicLevel::High),
            GpioSetting::OutputLow => Some(LogicLevel::Low),
            GpioSetting::Input => None,
        }
    }
}

impl From<GpioSetting> for GpioDirection {
    fn from(value: GpioSetting) -> Self {
        match value {
            GpioSetting::OutputHigh => GpioDirection::Output,
            GpioSetting::OutputLow => GpioDirection::Output,
            GpioSetting::Input => GpioDirection::Input,
        }
    }
}

#[derive(Debug, Parser)]
pub(crate) struct PinValues {
    #[arg(long, short = '0')]
    gp0: Option<GpioSetting>,
    #[arg(long, short = '1')]
    gp1: Option<GpioSetting>,
    #[arg(long, short = '2')]
    gp2: Option<GpioSetting>,
    #[arg(long, short = '3')]
    gp3: Option<GpioSetting>,
}

impl From<PinValues> for GpioChanges {
    fn from(value: PinValues) -> Self {
        let mut s = Self::new();
        if let Some(gp) = value.gp0 {
            s.with_gp0_direction(gp.into());
            if let Some(level) = gp.level() {
                s.with_gp0_level(level);
            }
        }
        if let Some(gp) = value.gp1 {
            s.with_gp1_direction(gp.into());
            if let Some(level) = gp.level() {
                s.with_gp1_level(level);
            }
        }
        if let Some(gp) = value.gp2 {
            s.with_gp2_direction(gp.into());
            if let Some(level) = gp.level() {
                s.with_gp2_level(level);
            }
        }
        if let Some(gp) = value.gp3 {
            s.with_gp3_direction(gp.into());
            if let Some(level) = gp.level() {
                s.with_gp3_level(level);
            }
        }
        s
    }
}
