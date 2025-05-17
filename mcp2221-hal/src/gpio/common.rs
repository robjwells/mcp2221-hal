#[derive(Debug, Clone, Copy)]
/// GPIO pin level setting.
pub enum LogicLevel {
    High,
    Low,
}

impl From<bool> for LogicLevel {
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

impl From<LogicLevel> for bool {
    fn from(value: LogicLevel) -> Self {
        match value {
            LogicLevel::High => true,
            LogicLevel::Low => false,
        }
    }
}

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
    Input,
    Output,
}

impl From<bool> for GpioDirection {
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}

impl From<GpioDirection> for bool {
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => true,
            GpioDirection::Output => false,
        }
    }
}

impl From<GpioDirection> for u8 {
    /// Convert a [`GpioDirection`] to 1 (if input) or 0 (if output).
    fn from(value: GpioDirection) -> Self {
        match value {
            GpioDirection::Input => 1,
            GpioDirection::Output => 0,
        }
    }
}
