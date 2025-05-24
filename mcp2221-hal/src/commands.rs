use crate::Error;

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
    /// Read the fixed chip factory serial number (always "01234567").
    ///
    /// See section 3.1.2 of the datasheet for the underlying Read Flash Data command.
    /// This is not include as a `FlashDataSubCode` case because it cannot be written.
    ReadChipFactorySerialNumber,
    /// Write various important data structures and strings into the flash memory
    /// of the MCP2221.
    ///
    /// See section 3.1.3 of the datasheet.
    WriteFlashData(FlashDataSubCode),
    /// Configure the run-time chip and GP pin settings.
    ///
    /// See section 3.1.13 of the datasheet for the Set SRAM Settings HID command.
    SetSRAMSettings,
    /// Retrieve the run-time chip and GP pin settings.
    ///
    /// See section 1.4 of the datasheet for information about the configuration
    /// process, particularly regarding the flash settings being copied into SRAM.
    GetSRAMSettings,
    /// Retrieve the GPIO direction and pin value for those pins set to GPIO operation.
    ///
    /// See section 3.1.11 of the datasheet.
    SetGpioOutputValues,
    /// Change GPIO pin output direction and logic level.
    ///
    /// See section 3.1.12 of the datasheet.
    GetGpioValues,
    /// Force a reset of the device.
    ///
    /// See section 3.1.15 of the datasheet.
    ResetChip,
    /// Request a read from an I2C target.
    ///
    /// The read data is not returned in response to this command, but to the
    /// Get Data command.
    I2cReadData,
    /// Request a read from an I2C target with a repeated START condition.
    ///
    /// The read data is not returned in response to this command, but to the
    /// Get Data command.
    I2cReadDataRepeatedStart,
    /// Read requested I2C data back from the MCP2221.
    ///
    /// See section 3.1.10 of the datasheet.
    I2cGetData,
    /// Write data to an I2C target.
    ///
    /// See section 3.1.5 of the datasheet.
    I2cWriteData,
    /// Write data to an I2C target with a repeated START condition.
    ///
    /// See section 3.1.6 of the datasheet.
    I2cWriteDataRepeatedStart,
    /// Write data to an I2C target without a STOP condition.
    ///
    /// See section 3.1.7 of the datasheet.
    I2cWriteDataNoStop,
}

impl McpCommand {
    /// Command prefix to be applied to the buffer sent to the MCP2221.
    ///
    /// In most cases this just involves writing the command code to byte 0 of the
    /// outgoing buffer. Some commands have subcommand bytes (byte 1), but Reset Chip
    /// four bytes in total.
    fn buffer_prefix(&self) -> &[u8] {
        match self {
            McpCommand::StatusSetParameters => &[0x10],
            McpCommand::ReadFlashData(sub_code) => match sub_code {
                FlashDataSubCode::ChipSettings => &[0xB0, 0x00],
                FlashDataSubCode::GPSettings => &[0xB0, 0x01],
                FlashDataSubCode::UsbManufacturerDescriptor => &[0xB0, 0x02],
                FlashDataSubCode::UsbProductDescriptor => &[0xB0, 0x03],
                FlashDataSubCode::UsbSerialNumberDescriptor => &[0xB0, 0x04],
            },
            McpCommand::ReadChipFactorySerialNumber => &[0xB0, 0x05],
            McpCommand::WriteFlashData(sub_code) => match sub_code {
                FlashDataSubCode::ChipSettings => &[0xB1, 0x00],
                FlashDataSubCode::GPSettings => &[0xB1, 0x01],
                FlashDataSubCode::UsbManufacturerDescriptor => &[0xB1, 0x02],
                FlashDataSubCode::UsbProductDescriptor => &[0xB1, 0x03],
                FlashDataSubCode::UsbSerialNumberDescriptor => &[0xB1, 0x04],
            },
            McpCommand::SetSRAMSettings => &[0x60],
            McpCommand::GetSRAMSettings => &[0x61],
            McpCommand::SetGpioOutputValues => &[0x50],
            McpCommand::GetGpioValues => &[0x51],
            McpCommand::ResetChip => &[0x70, 0xAB, 0xCD, 0xEF],
            McpCommand::I2cReadData => &[0x91],
            McpCommand::I2cReadDataRepeatedStart => &[0x93],
            McpCommand::I2cGetData => &[0x40],
            McpCommand::I2cWriteData => &[0x90],
            McpCommand::I2cWriteDataRepeatedStart => &[0x92],
            McpCommand::I2cWriteDataNoStop => &[0x94],
        }
    }
}

impl McpCommand {
    /// Returns true if the command has no response buffer to read.
    fn has_no_response(&self) -> bool {
        matches!(self, &Self::ResetChip)
    }

    /// Check error code for command-specific errors.
    ///
    /// A handful of commands have their own error codes that are not shared.
    ///
    /// The I2cEngineBusy error codes are particular important as they signal
    /// that we should attempt a command again.
    fn check_error_code(&self, code: u8) -> Result<(), Error> {
        match (code, self) {
            (0x01, Self::ReadFlashData(_)) => Err(Error::CommandNotSupported),
            (0x02, Self::WriteFlashData(_)) => Err(Error::CommandNotSupported),
            (0x03, Self::WriteFlashData(_)) => Err(Error::CommandNotAllowed),
            (0x01, Self::I2cWriteData) => Err(Error::I2cEngineBusy),
            (0x01, Self::I2cWriteDataRepeatedStart) => Err(Error::I2cEngineBusy),
            (0x01, Self::I2cWriteDataNoStop) => Err(Error::I2cEngineBusy),
            (0x01, Self::I2cReadData) => Err(Error::I2cEngineBusy),
            (0x01, Self::I2cReadDataRepeatedStart) => Err(Error::I2cEngineBusy),
            (0x41, Self::I2cGetData) => Err(Error::I2cEngineReadError),
            (_, _) => Ok(()),
        }
    }
}

/// Read various settings stored in the flash memory.
pub(crate) enum FlashDataSubCode {
    /// Chip configuration power-up settings.
    ChipSettings,
    /// GP pin power-up settings.
    GPSettings,
    /// USB manufacturer string descriptor used during USB enumeration.
    UsbManufacturerDescriptor,
    /// USB product string descriptor used during USB enumeration.
    UsbProductDescriptor,
    /// USB serial number string descriptor used during USB enumeration.
    UsbSerialNumberDescriptor,
}

pub(crate) struct UsbReport {
    /// Underlying HID command.
    command: McpCommand,
    /// Outgoing buffer sized to match those in the datasheet.
    ///
    /// The actual outgoing buffer will be 65 bytes, as the HidApi crate requires
    /// the USB HID report number to be prepended to the data bytes.
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
    pub(crate) fn new(command: McpCommand) -> Self {
        let mut buf = [0u8; 64];
        let prefix = command.buffer_prefix();
        buf[0..prefix.len()].copy_from_slice(prefix);
        Self {
            command,
            write_buffer: buf,
        }
    }

    /// Returns true if the command has no response buffer to read.
    pub(crate) fn has_no_response(&self) -> bool {
        self.command.has_no_response()
    }

    /// Check for a command-specific error.
    pub(crate) fn check_error_code(&self, code: u8) -> Result<(), Error> {
        self.command.check_error_code(code)
    }

    /// Write a single data byte in the outgoing USB report.
    ///
    /// `byte_index` must be in the range `2..=63`.
    ///
    /// Command at index 0 cannot be overwritten with this method.
    pub(crate) fn set_data_byte(&mut self, byte_index: usize, value: u8) {
        assert!(byte_index < 64, "Byte index {byte_index} too large.");
        assert!(byte_index != 0, "Cannot write to command byte index.");
        self.write_buffer[byte_index] = value;
    }
}
