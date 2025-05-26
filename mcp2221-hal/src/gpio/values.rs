use super::{GpioDirection, LogicLevel};

/// Status of the GPIO pins.
///
/// A field is `None` if the corresponding pin is not configured for GPIO operation.
///
/// If a pin is set as an input, the logic level is the value read on that pin. If
/// it is set as an output, it is the current output value of that pin.
///
/// Pins can be configured for GPIO operation in SRAM via [`MCP2221::sram_write_settings`].
///
/// [`MCP2221::sram_write_settings`]: crate::MCP2221::sram_write_settings
///
/// ## Datasheet
///
/// See section 3.1.12 for the underlying Get GPIO Values HID command.
#[derive(Debug)]
pub struct GpioValues {
    /// GP0 GPIO values.
    pub gp0: Option<(GpioDirection, LogicLevel)>,
    /// GP1 GPIO values.
    pub gp1: Option<(GpioDirection, LogicLevel)>,
    /// GP2 GPIO values.
    pub gp2: Option<(GpioDirection, LogicLevel)>,
    /// GP3 GPIO values.
    pub gp3: Option<(GpioDirection, LogicLevel)>,
}

impl GpioValues {
    /// Parse the response from the Get GPIO Values command.
    ///
    /// ## Datasheet
    ///
    /// See table 3-35 for the Get GPIO Values response layout.
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        Self {
            gp0: parse_bytes(buf[2], buf[3]),
            gp1: parse_bytes(buf[4], buf[5]),
            gp2: parse_bytes(buf[6], buf[7]),
            gp3: parse_bytes(buf[8], buf[9]),
        }
    }
}

/// Parse received bytes into a direction and level pair.
///
/// In the Get GPIO Values response, the bytes come in the same order as this function's
/// arguments: level first, then direction. We reverse this order in the returned tuple
/// since the level is only meaningful once you know the direction, so direction first
/// seems more fitting with how it will be used.
///
/// Returned value is optional because a sentinel value is received for each when the
/// pin in question is not configured for GPIO operation.
///
/// ## Datasheet
///
/// See table 3-35 for the Get GPIO Values response layout and byte definitions.
fn parse_bytes(level_byte: u8, direction_byte: u8) -> Option<(GpioDirection, LogicLevel)> {
    let level = logic_level_from_byte(level_byte);
    let direction = direction_from_byte(direction_byte);
    if let (Some(direction), Some(level)) = (direction, level) {
        Some((direction, level))
    } else {
        None
    }
}

/// Parse a byte from Get GPIO Values into a logic level.
///
/// The returned value is optional if the pin in question is not configured for GPIO
/// operation.
fn logic_level_from_byte(byte: u8) -> Option<LogicLevel> {
    /// Byte returned for a GP pin's logic level ("value") if the pin is not in GPIO mode.
    /// Note that this is different to the sentinel value for pin direction.
    const NOT_GPIO: u8 = 0xEE;

    match byte {
        NOT_GPIO => None,
        0x00 => Some(LogicLevel::Low),
        0x01 => Some(LogicLevel::High),
        _ => unreachable!("Invalid byte '{byte:X}' for GPIO logic level"),
    }
}

/// Parse a byte from Get GPIO Values into a GPIO pin direction.
///
/// The returned value is optional if the pin in question is not configured for GPIO
/// operation.
fn direction_from_byte(byte: u8) -> Option<GpioDirection> {
    /// Byte returned for a GP pin's direction if the pin is not in GPIO mode.
    /// Note that this is different to the sentinel value for pin logic level.
    const NOT_GPIO: u8 = 0xEF;

    match byte {
        NOT_GPIO => None,
        0x00 => Some(GpioDirection::Output),
        0x01 => Some(GpioDirection::Input),
        _ => unreachable!("Invalid byte '{byte:X}' for GPIO pin direction"),
    }
}

/// Changes to make to GPIO pin settings.
///
/// This offers a builder-like interface where values that are not set are left
/// unchanged in the device settings.
///
/// You can "set" the logic level for an input pin. This reflects the MCP2221 interface
/// but such a change naturally does not take effect unless and until the pin is set to
/// be an output. There is no advantage to doing this over setting the value at the same
/// time you set the pin as an output.
///
/// Note that these changes will not put a pin set to another mode into GPIO mode.
/// The mode must be changed separately via [`MCP2221::sram_write_settings`].
///
/// [`MCP2221::sram_write_settings`]: crate::MCP2221::sram_write_settings
///
/// ## Datasheet
///
/// See section 3.1.11 for the underlying Set GPIO Output Values HID command.
#[derive(Default, Debug)]
pub struct GpioChanges {
    gp0_direction: Option<GpioDirection>,
    gp0_level: Option<LogicLevel>,
    gp1_direction: Option<GpioDirection>,
    gp1_level: Option<LogicLevel>,
    gp2_direction: Option<GpioDirection>,
    gp2_level: Option<LogicLevel>,
    gp3_direction: Option<GpioDirection>,
    gp3_level: Option<LogicLevel>,
}

impl GpioChanges {
    /// Create a struct with no pending changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the direction of GP0.
    pub fn with_gp0_direction(&mut self, direction: GpioDirection) -> &mut Self {
        self.gp0_direction = Some(direction);
        self
    }

    /// Set the logic level of GP0.
    pub fn with_gp0_level(&mut self, level: LogicLevel) -> &mut Self {
        self.gp0_level = Some(level);
        self
    }

    /// Set the direction of GP1.
    pub fn with_gp1_direction(&mut self, direction: GpioDirection) -> &mut Self {
        self.gp1_direction = Some(direction);
        self
    }

    /// Set the logic level of GP1.
    pub fn with_gp1_level(&mut self, level: LogicLevel) -> &mut Self {
        self.gp1_level = Some(level);
        self
    }

    /// Set the direction of GP2.
    pub fn with_gp2_direction(&mut self, direction: GpioDirection) -> &mut Self {
        self.gp2_direction = Some(direction);
        self
    }

    /// Set the logic level of GP2.
    pub fn with_gp2_level(&mut self, level: LogicLevel) -> &mut Self {
        self.gp2_level = Some(level);
        self
    }

    /// Set the direction of GP3.
    pub fn with_gp3_direction(&mut self, direction: GpioDirection) -> &mut Self {
        self.gp3_direction = Some(direction);
        self
    }

    /// Set the logic level of GP3.
    pub fn with_gp3_level(&mut self, level: LogicLevel) -> &mut Self {
        self.gp3_level = Some(level);
        self
    }

    /// Write changes into a buffer for Set GPIO Output Values.
    ///
    /// ## Datasheet
    ///
    /// See section 3.1.11 of the datasheet for the layout of the command buffer.
    pub(crate) fn apply_to_buffer(&self, buf: &mut [u8; 64]) {
        // Each logic level setting and direction setting has a preceding byte
        // that determines if a new value is to be loaded. If that "enable" byte
        // is non-zero, the following byte is the setting loaded.
        //
        // The command write buffer is initially zeroed, which means that we only
        // need to change the write buffer where we intend to change a setting.
        const ENABLE_SETTING: u8 = 0x01;

        // GP0
        if let Some(level) = self.gp0_level {
            buf[2] = ENABLE_SETTING;
            buf[3] = level.into();
        }
        if let Some(direction) = self.gp0_direction {
            buf[4] = ENABLE_SETTING;
            buf[5] = direction.into();
        }

        // GP1
        if let Some(level) = self.gp1_level {
            buf[6] = ENABLE_SETTING;
            buf[7] = level.into();
        }
        if let Some(direction) = self.gp1_direction {
            buf[8] = ENABLE_SETTING;
            buf[9] = direction.into();
        }

        // GP2
        if let Some(level) = self.gp2_level {
            buf[10] = ENABLE_SETTING;
            buf[11] = level.into();
        }
        if let Some(direction) = self.gp2_direction {
            buf[12] = ENABLE_SETTING;
            buf[13] = direction.into();
        }

        // GP3
        if let Some(level) = self.gp3_level {
            buf[14] = ENABLE_SETTING;
            buf[15] = level.into();
        }
        if let Some(direction) = self.gp3_direction {
            buf[16] = ENABLE_SETTING;
            buf[17] = direction.into();
        }
    }
}
