use std::cell::Cell;

use hidapi::HidApi;

use super::MCP2221;
use crate::Error;
use crate::constants::{MCP2221_PID, MICROCHIP_VID};

/// # USB device functionality
impl MCP2221 {
    /// Open the first USB device found with the default vendor and product ID.
    ///
    /// The default VID is 1240 (0x4D8) and PID 221 (0xDD) for both the original
    /// MCP2221 and the (more common) MCP2221A.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_and_pid(MICROCHIP_VID, MCP2221_PID)
    }

    /// Open the first USB device found with the given venor and product ID.
    ///
    /// Use this function if you have changed the USB VID or PID of your MCP2221.
    ///
    /// # Errors
    ///
    /// An error will be returned if the USB device cannot be opened.
    pub fn open_with_vid_and_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self {
            inner: device,
            pins_taken: Cell::new(false),
        })
    }

    /// Get the USB HID device information from the host's USB interface.
    ///
    /// This is a thin wrapper around [`HidDevice::get_device_info`].
    ///
    /// [`HidDevice::get_device_info`]: hidapi::HidDevice::get_device_info
    ///
    /// # Errors
    ///
    /// An error will be returned if the device information cannot be returned
    /// from the underlying USB interface.
    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
}
