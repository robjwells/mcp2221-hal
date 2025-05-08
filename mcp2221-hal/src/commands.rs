pub(crate) enum McpCommand {
    /// Poll for the status of the device, cancel an I2C transfer,
    /// or set the I2C bus speed.
    ///
    /// See section 3.1.1.
    StatusSetParameters,
    /// Read various important data structures and strings stored in the flash
    /// memory on the MCP2221.
    ///
    /// See section 3.1.2 of the datasheet.
    ///
    /// Many of these settings determine start-up values that can be changed
    /// at runtime (the MCP2221 copies them into SRAM). See section 1.4.3.
    ReadFlashData(FlashDataSubCode),
    /// Write various important data structures and strings into the flash memory
    /// of the MCP2221.
    ///
    /// See section 3.1.3 of the datasheet.
    WriteFlashData(FlashDataSubCode),
    /// Retrieve the run-time chip and GP pin settings.
    ///
    /// See section 1.4 of the datasheet for information about the configuration
    /// process, particularly regarding the flash settings being copied into SRAM.
    GetSRAMSettings,
    SetSRAMSettings,
    SetGpioOutputValues,
    ResetChip,
}

/// Read various settings stored in the flash memory.
pub(crate) enum FlashDataSubCode {
    ChipSettings,
    // GP pin power-up settings.
    GPSettings,
    /// USB manufacturer string descriptor used during USB enumeration.
    UsbManufacturerDescriptor,
    /// USB product string descriptor used during USB enumeration.
    UsbProductDescriptor,
    /// USB serial number string descriptor used during USB enumeration.
    UsbSerialNumberDescriptor,
    /// Factory-set serial number. Always "01234567".
    ChipFactorySerialNumber,
}

pub(crate) struct UsbReport {
    pub(crate) write_buffer: [u8; 64],
}

impl UsbReport {
    pub(crate) fn report_bytes(&self) -> [u8; 65] {
        let mut out = [0u8; 65];
        out[1..65].copy_from_slice(&self.write_buffer);
        out
    }

    /// Write the appropriate command byte to write_buffer[1].
    ///
    /// write_buffer starts with the dummy/default report number, so the
    /// actual MCP command is at write_buffer[1..=65].
    pub(crate) fn new(c: McpCommand) -> Self {
        let mut buf = [0u8; 64];
        use FlashDataSubCode::*;
        use McpCommand::*;
        let (command_byte, sub_command_byte): (u8, Option<u8>) = match c {
            StatusSetParameters => (0x10, None),
            ReadFlashData(ChipSettings) => (0xB0, Some(0x00)),
            ReadFlashData(GPSettings) => (0xB0, Some(0x01)),
            ReadFlashData(UsbManufacturerDescriptor) => (0xB0, Some(0x02)),
            ReadFlashData(UsbProductDescriptor) => (0xB0, Some(0x03)),
            ReadFlashData(UsbSerialNumberDescriptor) => (0xB0, Some(0x04)),
            ReadFlashData(ChipFactorySerialNumber) => (0xB0, Some(0x05)),
            WriteFlashData(ChipSettings) => (0xB1, Some(0x00)),
            WriteFlashData(GPSettings) => (0xB1, Some(0x01)),
            WriteFlashData(UsbManufacturerDescriptor) => (0xB1, Some(0x02)),
            WriteFlashData(UsbProductDescriptor) => (0xB1, Some(0x03)),
            WriteFlashData(UsbSerialNumberDescriptor) => (0xB1, Some(0x04)),
            WriteFlashData(ChipFactorySerialNumber) => {
                todo!("Chip factory serial number cannot be changed. Error I guess?")
            }
            GetSRAMSettings => (0x61, None),
            SetSRAMSettings => (0x60, None),
            SetGpioOutputValues => (0x50, None),
            ResetChip => (0x70, Some(0xAB)),
        };
        buf[0] = command_byte;
        if let Some(sub_command_byte) = sub_command_byte {
            buf[1] = sub_command_byte;
        }
        Self { write_buffer: buf }
    }

    /// Write a single data byte in the outgoing USB report.
    ///
    /// `byte_index` must be in the range `2..=63`.
    ///
    /// Command and subcommand bytes (indices 0 and 1) cannot be set
    /// with this command.
    pub(crate) fn set_data_byte(&mut self, byte_index: usize, value: u8) {
        assert!(byte_index < 64, "Byte index {byte_index} too large.");
        assert!(byte_index != 0, "Cannot write to command byte index.");
        assert!(byte_index != 1, "Cannot write to subcommand byte index.");
        self.write_buffer[byte_index] = value;
    }
}
