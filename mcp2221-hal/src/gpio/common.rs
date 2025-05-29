/// GPIO pin logic level.
///
/// When used with input pins, this is the level read on the pin. With output
/// pins it is the level output on that pin by the MCP2221.
#[derive(Debug, Clone, Copy)]
pub enum LogicLevel {
    /// A high logic level (nominally the supply voltage).
    High,
    /// A low logic level (nominally 0V).
    Low,
}

impl LogicLevel {
    /// Returns `true` if the logic level is [`High`].
    ///
    /// [`High`]: LogicLevel::High
    #[must_use]
    pub fn is_high(self) -> bool {
        matches!(self, Self::High)
    }

    /// Returns `true` if the logic level is [`Low`].
    ///
    /// [`Low`]: LogicLevel::Low
    #[must_use]
    pub fn is_low(self) -> bool {
        matches!(self, Self::Low)
    }
}

#[doc(hidden)]
impl From<bool> for LogicLevel {
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

#[doc(hidden)]
impl From<LogicLevel> for bool {
    fn from(value: LogicLevel) -> Self {
        match value {
            LogicLevel::High => true,
            LogicLevel::Low => false,
        }
    }
}

#[doc(hidden)]
impl From<LogicLevel> for u8 {
    /// Convert a [`LogicLevel`] to 1 (if high) or 0 (if low).
    fn from(value: LogicLevel) -> Self {
        match value {
            LogicLevel::High => 1,
            LogicLevel::Low => 0,
        }
    }
}

/// GPIO pin direction.
#[derive(Debug, Clone, Copy)]
pub enum GpioDirection {
    /// Digital input.
    Input,
    /// Digital output.
    Output,
}

impl GpioDirection {
    /// Returns `true` if the gpio direction is [`Input`].
    ///
    /// [`Input`]: GpioDirection::Input
    #[must_use]
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input)
    }

    /// Returns `true` if the gpio direction is [`Output`].
    ///
    /// [`Output`]: GpioDirection::Output
    #[must_use]
    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output)
    }
}

#[doc(hidden)]
impl From<bool> for GpioDirection {
    /// Helper for use with [`bit_field::BitField::get_bit`], which returns a 1
    /// bit as `true` and 0 as false.
    ///
    /// For the MCP2221, 1 means GPIO input and 0 means GPIO output.
    ///
    /// ## Datasheet
    ///
    /// See, for example, table 3-13 (Write Flash Data: Write GP Settings),
    /// or table 3-35 (Get GPIO Values).
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}

#[doc(hidden)]
impl From<GpioDirection> for bool {
    /// Helper for use with [`bit_field::BitField::get_bit`], which takes `true`
    /// to set a bit to 1.
    ///
    /// For the MCP2221, 1 means GPIO input and 0 means GPIO output.
    ///
    /// ## Datasheet
    ///
    /// See, for example, table 3-13 (Write Flash Data: Write GP Settings),
    /// or table 3-35 (Get GPIO Values).
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => true,
            GpioDirection::Output => false,
        }
    }
}

#[doc(hidden)]
impl From<GpioDirection> for u8 {
    /// Convert a [`GpioDirection`] to 1 (if input) or 0 (if output).
    ///
    /// Note that the inverse conversion (`u8` to `GpioDirection`) is not
    /// implemented as a sentinel value is used to mean that the pin is not
    /// set for GPIO operation.
    ///
    /// ## Datasheet
    ///
    /// See table 3-32 (Set GPIO Output Values) for the use of 0x01 to mean
    /// input and 0x00 to mean output. This is the only case where we need
    /// a full byte to represent the direction.
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => 1,
            GpioDirection::Output => 0,
        }
    }
}
