//! Chip and GP pin configuration.
//!
//! # Overview
//!
//! The MCP2221 has two main groups of settings:
//!
//! - [`ChipSettings`], collecting various configuration options.
//! - [`GpSettings`], for configuring the four general-purpose (GP) pins.
//!
//! These are stored in two places:
//!
//! - Durably in flash memory, from which they are copied into SRAM on power-up and
//!   determine the initial behaviour of the MCP2221. Driver methods that read or
//!   change flash memory start with `flash_`.
//! - Ephemerally in SRAM, where changes affect the behaviour of the running chip but
//!   do not persist across resets unless specifically written to flash memory. Driver
//!   methods that read or change SRAM start with `sram_`.
//!
//! This configuration process is described in section 1.4 of the datasheet.
//!
//! In addition, the USB manufacturer, device and serial number strings are stored in
//! flash memory only and changes to these (like all changes to flash memory) are
//! not reflected until the device is reset. Driver methods that change these start
//! with `change_usb_` and take a [`DeviceString`], a string wrapper that enforces
//! a 60-byte length when the string is encoded as UTF-16.
//!
//! Not everything in [`ChipSettings`] can be changed in SRAM, for example the USB
//! vendor ID (VID) and product ID (PID). Changes to the SRAM settings are done with
//! [`SramSettingsChanges`], which offers a builder-like interface where settings
//! that are not set are left unchanged.
//!
//! # Bugs
//!
//! There are two strange behaviours related to the MCP2221 settings that appear to be
//! firmware bugs.
//!
//! ## Stale SRAM settings
//!
//! The [`ChipSettings`] and [`GpSettings`] read from SRAM may not reflect the actual
//! configuration that is in effect. This is known to happen in two situations:
//!
//! - Changing the [`GpSettings`] without also setting the ADC and DAC Vrm levels
//!   results in the Vrm level being set to "off", but this is not shown in the
//!   settings when they are read subsequently. See the [`analog`] module for
//!   details about the behave of the device in this state.
//!
//!   This bug is noted in the datasheet in a note in section 1.8.1.1.
//!
//!   [`SramSettingsChanges::with_gp_modes`] takes optional DAC and ADC voltage
//!   references, so that this firmware bug can be worked around. It is also named
//!   `with_gp_modes` to emphasise that it is only necessary to change GP pin modes,
//!   and GPIO output direction and logic level can be changed with
//!   [`MCP2221::gpio_write`].
//!
//! - That said, [`MCP2221::gpio_write`] will cause the GPIO output direction and
//!   logic level in the [`GpSettings`] read from SRAM to be incorrect. It appears
//!   that the underlying HID command for that method (Set GPIO Output Values) does
//!   not update the GP pin settings structure stored in the device’s SRAM, though
//!   it does have the expected effect on the pins themselves.
//!
//! [`MCP2221::gpio_write`]: crate::MCP2221::gpio_write
//!
//! In short, you cannot perform arbitrary changes to chip or GP pin settings in
//! SRAM and then expect to be able to know the actual behaviour of the device by
//! reading those SRAM settings back.
//!
//! ## ADC and DAC voltage reference in flash memory
//!
//! If you change the chip settings in flash memory to set the ADC or DAC to use the Vrm
//! voltage reference (at any voltage level), when the chip powers-up the respective
//! peripheral will behave as if it was configured with a Vrm level of "off", and this
//! is almost certainly not what you want (especially for the DAC). Please see the
//! documentation of the [`analog`] module for more information.
//!
//! If you wish to use the ADC or DAC with the Vrm reference, you should configure this
//! in SRAM after the device is powered-up.
//!
//! [`analog`]: crate::analog
//!
//! # Datasheet
//!
//! See section 3.1.2 (the Read Flash Data HID command) for descriptions of all of the
//! chip settings (table 3-5) and GP settings (table 3-6) that are read from the device.
//!
//! See section 3.1.13 (the Set SRAM Settings command) for details about what chip
//! settings can be changed in SRAM (all GP settings can be changed in SRAM).
//!
//! The clearest descriptions of the various settings are often found in the registers
//! section, 1.4.2 for chip settings and 1.4.3 for GP settings. Note that not everything
//! in the registers maps cleanly to something exposed through the settings-related HID
//! commands, though most do and even with the bit patterns unchanged in the HID
//! response buffers.

mod chip_settings;
mod common;
mod gp;
mod sram;

pub use chip_settings::ChipSettings;
pub use common::{ClockDutyCycle, ClockFrequency, ClockOutputSetting, DeviceString};
pub use gp::{Gp0Mode, Gp1Mode, Gp2Mode, Gp3Mode, GpSettings};
pub use sram::{InterruptSettingsChanges, SramSettingsChanges};
