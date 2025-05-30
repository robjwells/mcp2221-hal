use mcp2221_hal::MCP2221;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct UsbInfo<'a> {
    pub(crate) manufacturer: Option<&'a str>,
    pub(crate) product: Option<&'a str>,
    pub(crate) serial_number: Option<&'a str>,
    pub(crate) vendor_id: String,
    pub(crate) product_id: String,
    pub(crate) path: String,
}

impl<'a> From<&'a hidapi::DeviceInfo> for UsbInfo<'a> {
    fn from(info: &'a hidapi::DeviceInfo) -> Self {
        Self {
            manufacturer: info.manufacturer_string(),
            product: info.product_string().to_owned(),
            serial_number: info.serial_number(),
            vendor_id: format!("{:#X}", info.vendor_id()),
            product_id: format!("{:#X}", info.product_id()),
            path: info.path().to_string_lossy().to_string(),
        }
    }
}

pub(crate) fn print_info(device: &MCP2221) -> Result<(), mcp2221_hal::Error> {
    println!("{:#?}", UsbInfo::from(&device.usb_device_info()?));
    Ok(())
}
