//! TODO module level docs for settings.

mod chip_settings;
mod common;
pub mod gpio;
mod security;
mod sram;

pub use chip_settings::ChipSettings;
pub use common::{ClockFrequency, ClockSetting, DeviceString, DutyCycle};
pub use gpio::GpSettings;
pub use security::ChipConfigurationSecurity;
pub use sram::{ChangeSramSettings, InterruptSettings, SramSettings};
