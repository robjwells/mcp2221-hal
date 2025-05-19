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
