fn main() {
    let mut mcp = mcp2221_hal::MCP2221::open().unwrap();
    let info = mcp.usb_device_info().unwrap();
    println!("VID PID:\t{} {}", info.vendor_id(), info.product_id());
    println!("Vendor: \t{:?}", info.manufacturer_string());
    println!("Product:\t{:?}", info.product_string());
    println!("Serial: \t{:?}", info.serial_number());
    println!("Release:\t{:?}", info.release_number());
    println!("Path:   \t{:?}", info.path());

    let status = mcp.status().expect("Failed to get status.");
    println!("{status:#?}");

    let flash_data = mcp.read_flash_data().expect("Failed to read flash data");
    println!("{flash_data:#?}");
}
