//! TODO module level docs for settings.

mod chip;
mod chip_settings;
mod common;
mod gp;
mod sram;

pub use chip_settings::ChipSettings;
pub use common::{ClockDutyCycle, ClockFrequency, ClockOutputSetting, DeviceString};
pub use gp::{Gp0Mode, Gp1Mode, Gp2Mode, Gp3Mode, GpSettings};
pub use sram::{ChangeInterruptSettings, ChangeSramSettings, SramSettings};
