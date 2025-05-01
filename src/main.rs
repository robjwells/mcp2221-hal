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
    // match mcp.try_write() {
    //     Ok(report) => print_report(report),
    //     Err(e) => eprintln!("{:?}", e),
    // }
}
//
// fn print_report(report: [u8; 64]) {
//     for (idx, value) in report.into_iter().enumerate() {
//         println!("{idx:02}\t0x{value:02X}");
//     }
// }
