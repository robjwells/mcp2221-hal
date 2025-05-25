#[derive(Debug, Clone, Copy)]
/// GPIO pin logic level.
///
/// When used with input pins, this is the level read on the pin. With output
/// pins it is the level output on that pin by the MCP2221.
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

#[derive(Debug, Clone, Copy)]
/// GPIO pin direction.
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
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}

#[doc(hidden)]
impl From<GpioDirection> for bool {
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
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => 1,
            GpioDirection::Output => 0,
        }
    }
}
