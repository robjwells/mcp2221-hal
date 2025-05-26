//! TODO module level docs for settings.

mod chip;
mod chip_settings;
mod common;
mod gp;
mod gp_settings;
mod security;
mod sram;

pub use chip_settings::ChipSettings;
pub use common::{ClockFrequency, ClockSetting, DeviceString, DutyCycle};
pub use gp::{Gp0Mode, Gp1Mode, Gp2Mode, Gp3Mode, GpSettings};
pub use security::ChipConfigurationSecurity;
pub use sram::{ChangeSramSettings, InterruptSettings, SramSettings};
