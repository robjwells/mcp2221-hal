use super::{GpioDirection, LogicLevel};

/// Byte returned for a GP pin's logic level ("value") if the pin is not in GPIO mode.
const NOT_GPIO_LEVEL: u8 = 0xEE;
/// Byte returned for a GP pin's direction if the pin is not in GPIO mode.
const NOT_GPIO_DIRECTION: u8 = 0xEF;

#[derive(Debug)]
pub struct GpioValues {
    gp0: Option<PinValue>,
    gp1: Option<PinValue>,
    gp2: Option<PinValue>,
    gp3: Option<PinValue>,
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
}

#[derive(Debug)]
pub struct PinValue {
    direction: GpioDirection,
    level: LogicLevel,
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
