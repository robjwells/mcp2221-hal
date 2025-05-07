use mcp2221_hal::types::{VoltageReference, VrmVoltage};

fn main() {
    let mut mcp = mcp2221_hal::MCP2221::open_with_vid_pid(1240, 221).unwrap();

    let status = mcp.status().expect("Failed to get status.");
    println!("{status:#?}");

    let flash_data = mcp.read_flash_data().expect("Failed to read flash data");
    println!("{flash_data:#?}");

    let sram_settings = mcp
        .get_sram_settings()
        .expect("Failed to read Sram settings");
    println!("{sram_settings:#?}");

    mcp.configure_dac_source(VoltageReference::Vrm, VrmVoltage::V1_024)
        .unwrap();
    mcp.set_dac_output_value(23).unwrap();
}
