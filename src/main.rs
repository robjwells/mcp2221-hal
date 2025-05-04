#![allow(unused, dead_code)]
use mcp2221_hal::types::{DeviceString, I2cSpeed};

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

    // let flash_data = mcp.read_flash_data().expect("Failed to read flash data");
    // println!("{flash_data:#?}");

    // println!("{:?}", mcp.cancel_i2c_transfer());

    // println!("{:?}", mcp.set_i2c_bus_speed(I2cSpeed::Fast_400kbps));

    let orig_mfr = DeviceString::try_from("Microchip Technology Inc.".to_owned())
        .expect("Couldn't construct device string with Microchip original string.");
    let new_mfr = DeviceString::try_from("robjwells".to_owned())
        .expect("Failed to make robjwells device string.");
    let orig_prod = DeviceString::try_from("MCP2221 USB-I2C/UART Combo".to_owned())
        .expect("Couldn't construct device string with Microchip original string.");
    let new_prod = DeviceString::try_from("rob's project".to_owned())
        .expect("Failed to make robjwells device string.");
    let orig_serial = DeviceString::try_from("0003181506".to_owned())
        .expect("Failed to make device string with original serial number");
    let new_serial = DeviceString::try_from("hello".to_owned())
        .expect("Failed to make device string with new serial number");

    if let Err(e) = mcp.write_usb_manufacturer_descriptor(&new_mfr) {
        println!("{e:?}");
    }
    if let Err(e) = mcp.write_usb_product_descriptor(&new_prod) {
        println!("{e:?}");
    }
    if let Err(e) = mcp.write_usb_serial_number_descriptor(&new_serial) {
        println!("{e:?}");
    }
    let flash_data = mcp.read_flash_data().expect("Failed to read flash data");
    println!("{flash_data:#?}");

    if let Err(e) = mcp.write_usb_manufacturer_descriptor(&orig_mfr) {
        println!("{e:?}");
    }
    if let Err(e) = mcp.write_usb_product_descriptor(&orig_prod) {
        println!("{e:?}");
    }
    if let Err(e) = mcp.write_usb_serial_number_descriptor(&orig_serial) {
        println!("{e:?}");
    }
    let flash_data = mcp.read_flash_data().expect("Failed to read flash data");
    println!("{flash_data:#?}");
}
