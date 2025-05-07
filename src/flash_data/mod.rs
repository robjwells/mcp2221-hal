//! Configuration stored in the MCP2221's flash memory.

use crate::types::DeviceString;

pub use chip_settings::ChipSettings;
pub use gp::GpSettings;

mod chip_settings;
mod common;
mod gp;

/// Configuration stored in the MCP2221's flash memory.
///
/// Changes to these settings take effect on power-up.
#[derive(Debug)]
pub struct FlashData {
    /// Chip settings.
    pub chip_settings: ChipSettings,
    /// General-purpose pins power-up settings.
    pub gp_settings: GpSettings,
    /// Manufacturer string descriptor used during USB enumeration.
    pub usb_manufacturer_descriptor: DeviceString,
    /// Product string descriptor used during USB enumeration.
    pub usb_product_descriptor: DeviceString,
    /// Serial number used during USB enumeration.
    pub usb_serial_number_descriptor: DeviceString,
    /// Factory-set serial number.
    ///
    /// Always "01234567" for the MCP2221. This cannot be changed.
    pub chip_factory_serial_number: String,
}

impl FlashData {
    pub(crate) fn from_buffers(
        chip_settings: &[u8; 64],
        gp_settings: &[u8; 64],
        usb_mfr: &[u8; 64],
        usb_product: &[u8; 64],
        usb_serial: &[u8; 64],
        chip_factory_serial: &[u8; 64],
    ) -> Self {
        Self {
            chip_settings: ChipSettings::from_buffer(chip_settings),
            gp_settings: GpSettings::from_buffer(gp_settings),
            usb_manufacturer_descriptor: DeviceString::from_device_report(usb_mfr),
            usb_product_descriptor: DeviceString::from_device_report(usb_product),
            usb_serial_number_descriptor: DeviceString::from_device_report(usb_serial),
            chip_factory_serial_number: FlashData::buffer_to_chip_factory_serial(
                chip_factory_serial,
            ),
        }
    }

    // Chip factory serial is ASCII chars, and always "01234567".
    fn buffer_to_chip_factory_serial(buf: &[u8; 64]) -> String {
        let length = buf[2] as usize;
        String::from_utf8(buf[4..(4 + length)].to_vec())
            .expect("Chip factory serial not ASCII as expected.")
    }
}
