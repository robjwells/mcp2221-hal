use super::{GpioDirection, LogicLevel, PinNumber};

/// Byte returned for a GP pin's logic level ("value") if the pin is not in GPIO mode.
const NOT_GPIO_LEVEL: u8 = 0xEE;
/// Byte returned for a GP pin's direction if the pin is not in GPIO mode.
const NOT_GPIO_DIRECTION: u8 = 0xEF;

/// GPIO status read from the device.
///
/// Each field is `None` if the corresponding pin is not configured for GPIO operation.
#[derive(Debug)]
pub struct GpioValues {
    /// GPIO settings for GP0.
    pub gp0: Option<PinValue>,
    /// GPIO settings for GP1.
    pub gp1: Option<PinValue>,
    /// GPIO settings for GP2.
    pub gp2: Option<PinValue>,
    /// GPIO settings for GP3.
    pub gp3: Option<PinValue>,
}

impl GpioValues {
    /// Parse the response from the Get GPIO Values command.
    ///
    /// See datasheet section 3.1.12.1.
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        let gp0 = PinValue::from_bytes(buf[2], buf[3]);
        let gp1 = PinValue::from_bytes(buf[4], buf[5]);
        let gp2 = PinValue::from_bytes(buf[6], buf[7]);
        let gp3 = PinValue::from_bytes(buf[8], buf[9]);
        Self { gp0, gp1, gp2, gp3 }
    }

    pub(crate) fn for_pin_number(&self, pin_number: PinNumber) -> Option<PinValue> {
        match pin_number {
            PinNumber::Gp0 => self.gp0,
            PinNumber::Gp1 => self.gp1,
            PinNumber::Gp2 => self.gp2,
            PinNumber::Gp3 => self.gp3,
        }
    }
}

/// Status of an individual GPIO pin.
#[derive(Debug, Clone, Copy)]
pub struct PinValue {
    /// Whether the pin is configured as an input or output.
    pub direction: GpioDirection,
    /// The logic level read on the pin (if input) or output on the pin.
    pub level: LogicLevel,
}

impl PinValue {
    fn from_bytes(level: u8, direction: u8) -> Option<Self> {
        let level = logic_level_from_byte(level);
        let direction = direction_from_byte(direction);
        if let (Some(direction), Some(level)) = (direction, level) {
            Some(Self { direction, level })
        } else {
            None
        }
    }
}

fn logic_level_from_byte(byte: u8) -> Option<LogicLevel> {
    match byte {
        NOT_GPIO_LEVEL => None,
        0x00 => Some(LogicLevel::Low),
        0x01 => Some(LogicLevel::High),
        _ => unreachable!("Invalid byte '{byte:X}' for GPIO logic level"),
    }
}

fn direction_from_byte(byte: u8) -> Option<GpioDirection> {
    match byte {
        NOT_GPIO_DIRECTION => None,
        0x00 => Some(GpioDirection::Output),
        0x01 => Some(GpioDirection::Input),
        _ => unreachable!("Invalid byte '{byte:X}' for GPIO pin direction"),
    }
}

/// Changes to make to GPIO pin direction and logic level.
///
/// Note that you can "set" the logic level for an input pin. This reflects the MCP2221
/// interface (in `Set GPIO Output Values`, section 3.1.11) but it naturally does not
/// take effect unless and until the pin is set to be an output.
#[derive(Default, Debug)]
pub struct ChangeGpioValues {
    gp0_direction: Option<GpioDirection>,
    gp0_level: Option<LogicLevel>,
    gp1_direction: Option<GpioDirection>,
    gp1_level: Option<LogicLevel>,
    gp2_direction: Option<GpioDirection>,
    gp2_level: Option<LogicLevel>,
    gp3_direction: Option<GpioDirection>,
    gp3_level: Option<LogicLevel>,
}

impl ChangeGpioValues {
    /// Create a struct with no pending changes.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_level_for_pin_number(
        &mut self,
        pin: PinNumber,
        level: LogicLevel,
    ) -> &mut Self {
        match pin {
            PinNumber::Gp0 => self.with_gp0_level(level),
            PinNumber::Gp1 => self.with_gp1_level(level),
            PinNumber::Gp2 => self.with_gp2_level(level),
            PinNumber::Gp3 => self.with_gp3_level(level),
        }
    }

    pub(crate) fn with_direction_for_pin_number(
        &mut self,
        pin: PinNumber,
        direction: GpioDirection,
    ) -> &mut Self {
        match pin {
            PinNumber::Gp0 => self.with_gp0_direction(direction),
            PinNumber::Gp1 => self.with_gp1_direction(direction),
            PinNumber::Gp2 => self.with_gp2_direction(direction),
            PinNumber::Gp3 => self.with_gp3_direction(direction),
        }
    }

    /// Set the direction of GP0.
    pub fn with_gp0_direction(&mut self, direction: GpioDirection) -> &mut Self {
        self.gp0_direction = Some(direction);
        self
    }

    /// Set the logic level of GP0.
    ///
    /// Not this will only take effect if GP0 is set to be a GPIO output pin.
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
    ///
    /// Not this will only take effect if GP0 is set to be a GPIO output pin.
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
    ///
    /// Not this will only take effect if GP0 is set to be a GPIO output pin.
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
    ///
    /// Not this will only take effect if GP0 is set to be a GPIO output pin.
    pub fn with_gp3_level(&mut self, level: LogicLevel) -> &mut Self {
        self.gp3_level = Some(level);
        self
    }

    /// Encode self in the format expected by Set GPIO Output Values.
    ///
    /// See section 3.1.11 of the datasheet.
    pub(crate) fn apply_to_buffer(&self, buf: &mut [u8; 64]) {
        // Each logic level setting and direction setting has a preceding byte
        // that determines if a new value is to be loaded. If that "enable" byte
        // is non-zero, the following byte is the loaded.
        //
        // The command write buffer is initially zeroed, which means that we only
        // need to change the write buffer where we intend to change a setting.

        // GP0
        if let Some(level) = self.gp0_level {
            buf[2] = 0x01;
            buf[3] = level.into();
        }
        if let Some(direction) = self.gp0_direction {
            buf[4] = 0x01;
            buf[5] = direction.into();
        }

        // GP1
        if let Some(level) = self.gp1_level {
            buf[6] = 0x01;
            buf[7] = level.into();
        }
        if let Some(direction) = self.gp1_direction {
            buf[8] = 0x01;
            buf[9] = direction.into();
        }

        // GP2
        if let Some(level) = self.gp2_level {
            buf[10] = 0x01;
            buf[11] = level.into();
        }
        if let Some(direction) = self.gp2_direction {
            buf[12] = 0x01;
            buf[13] = direction.into();
        }

        // GP3
        if let Some(level) = self.gp3_level {
            buf[14] = 0x01;
            buf[15] = level.into();
        }
        if let Some(direction) = self.gp3_direction {
            buf[16] = 0x01;
            buf[17] = direction.into();
        }
    }
}
