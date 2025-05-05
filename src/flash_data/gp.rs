use crate::types::{GpioDirection, LogicLevel};

use bit_field::BitField;

#[derive(Debug)]
pub struct GpSettings {
    pub gp0: Gp0Settings,
    pub gp1: Gp1Settings,
    pub gp2: Gp2Settings,
    pub gp3: Gp3Settings,
}

impl GpSettings {
    pub(super) fn from_buffer(buf: &[u8; 64]) -> Self {
        Self {
            gp0: (
                buf[4].get_bit(4).into(),
                buf[4].get_bit(3).into(),
                buf[4].get_bits(0..=2).into(),
            )
                .into(),
            gp1: (
                buf[5].get_bit(4).into(),
                buf[5].get_bit(3).into(),
                buf[5].get_bits(0..=2).into(),
            )
                .into(),
            gp2: (
                buf[6].get_bit(4).into(),
                buf[6].get_bit(3).into(),
                buf[6].get_bits(0..=2).into(),
            )
                .into(),
            gp3: (
                buf[7].get_bit(4).into(),
                buf[7].get_bit(3).into(),
                buf[7].get_bits(0..=2).into(),
            )
                .into(),
        }
    }
}

impl crate::commands::WriteCommandData for GpSettings {
    fn apply_to_buffer(&self, buf: &mut [u8; 64]) {
        // Byte 2 -- GP0
        buf[2].set_bit(4, self.gp0.power_up_value.into());
        buf[2].set_bit(3, self.gp0.power_up_direction.into());
        buf[2].set_bits(0..=2, self.gp0.power_up_designation.into());

        // Byte 3 -- GP1
        buf[3].set_bit(4, self.gp1.power_up_value.into());
        buf[3].set_bit(3, self.gp1.power_up_direction.into());
        buf[3].set_bits(0..=2, self.gp1.power_up_designation.into());

        // Byte 4 -- GP2
        buf[4].set_bit(4, self.gp2.power_up_value.into());
        buf[4].set_bit(3, self.gp2.power_up_direction.into());
        buf[4].set_bits(0..=2, self.gp2.power_up_designation.into());

        // Byte 5 -- GP3
        buf[5].set_bit(4, self.gp3.power_up_value.into());
        buf[5].set_bit(3, self.gp3.power_up_direction.into());
        buf[5].set_bits(0..=2, self.gp3.power_up_designation.into());
    }
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp0Designation {
    LED_UART_RX,
    SSPND,
    GPIO,
    DontCare,
}

impl From<u8> for Gp0Designation {
    fn from(value: u8) -> Self {
        assert!(value <= 0b111, "Incorrect use of the from constructor.");
        match value {
            0b010 => Self::LED_UART_RX,
            0b001 => Self::SSPND,
            0b000 => Self::GPIO,
            _ => Self::DontCare,
        }
    }
}

impl From<Gp0Designation> for u8 {
    fn from(value: Gp0Designation) -> Self {
        match value {
            Gp0Designation::SSPND => 0b010,
            Gp0Designation::LED_UART_RX => 0b001,
            Gp0Designation::GPIO => 0b000,
            Gp0Designation::DontCare => 0b111,
        }
    }
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp1Designation {
    ClockOutput,
    InterruptDetection,
    LED_UART_TX,
    ADC1,
    GPIO,
    DontCare,
}

impl From<u8> for Gp1Designation {
    fn from(value: u8) -> Self {
        assert!(value <= 0b111, "Incorrect use of the from constructor.");
        // Note the case order here matches the order in the datasheet.
        match value {
            0b001 => Self::ClockOutput,
            0b100 => Self::InterruptDetection,
            0b011 => Self::LED_UART_TX,
            0b010 => Self::ADC1,
            0b000 => Self::GPIO,
            _ => Self::DontCare,
        }
    }
}

impl From<Gp1Designation> for u8 {
    fn from(value: Gp1Designation) -> Self {
        match value {
            Gp1Designation::InterruptDetection => 0b100,
            Gp1Designation::LED_UART_TX => 0b11,
            Gp1Designation::ADC1 => 0b010,
            Gp1Designation::ClockOutput => 0b001,
            Gp1Designation::GPIO => 0b000,
            Gp1Designation::DontCare => 0b111,
        }
    }
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp2Designation {
    DAC1,
    ADC2,
    USBCFG,
    GPIO,
    DontCare,
}

impl From<u8> for Gp2Designation {
    fn from(value: u8) -> Self {
        assert!(value <= 0b111, "Incorrect use of the from constructor.");
        match value {
            0b011 => Self::DAC1,
            0b010 => Self::ADC2,
            0b001 => Self::USBCFG,
            0b000 => Self::GPIO,
            _ => Self::DontCare,
        }
    }
}

impl From<Gp2Designation> for u8 {
    fn from(value: Gp2Designation) -> Self {
        // The datasheet incorrectly lists "clock output" when writing the GP2 settings
        // but it should be USBCFG.
        match value {
            Gp2Designation::DAC1 => 0b011,
            Gp2Designation::ADC2 => 0b010,
            Gp2Designation::USBCFG => 0b001,
            Gp2Designation::GPIO => 0b000,
            Gp2Designation::DontCare => 0b111,
        }
    }
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp3Designation {
    DAC2,
    ADC3,
    LED_I2C,
    GPIO,
    DontCare,
}

impl From<u8> for Gp3Designation {
    fn from(value: u8) -> Self {
        assert!(value <= 0b111, "Incorrect use of the from constructor.");
        match value {
            0b011 => Self::DAC2,
            0b010 => Self::ADC3,
            0b001 => Self::LED_I2C,
            0b000 => Self::GPIO,
            _ => Self::DontCare,
        }
    }
}

impl From<Gp3Designation> for u8 {
    fn from(value: Gp3Designation) -> Self {
        match value {
            Gp3Designation::DAC2 => 0b011,
            Gp3Designation::ADC3 => 0b010,
            Gp3Designation::LED_I2C => 0b001,
            Gp3Designation::GPIO => 0b000,
            Gp3Designation::DontCare => 0b111,
        }
    }
}

#[derive(Debug)]
pub struct Gp0Settings {
    /// GP0 power-up output value.
    ///
    /// When GP0 is set as an output GPIO, this value will be present at
    /// the GP0 pin at power-up/reset.
    ///
    /// Byte 4 bit 4.
    pub power_up_value: LogicLevel,
    /// GP0 power-up direction.
    ///
    /// Works only when GP0 is set for GPIO operation.
    ///
    /// Byte 4 bit 3.
    pub power_up_direction: GpioDirection,
    /// GP0 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 4 bits 0..=2.
    pub power_up_designation: Gp0Designation,
}

#[derive(Debug)]
pub struct Gp1Settings {
    /// GP1 power-up output value.
    ///
    /// When GP1 is set as an output GPIO, this value will be present at
    /// the GP1 pin at power-up/reset.
    ///
    /// Byte 5 bit 4.
    pub power_up_value: LogicLevel,
    /// GP1 power-up direction.
    ///
    /// Works only when GP1 is set for GPIO operation.
    ///
    /// Byte 5 bit 3.
    pub power_up_direction: GpioDirection,
    /// GP1 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 5 bits 0..=2.
    pub power_up_designation: Gp1Designation,
}

#[derive(Debug)]
pub struct Gp2Settings {
    /// GP2 power-up output value.
    ///
    /// When GP2 is set as an output GPIO, this value will be present at
    /// the GP2 pin at power-up/reset.
    ///
    /// Byte 6 bit 4.
    pub power_up_value: LogicLevel,
    /// GP2 power-up direction.
    ///
    /// Works only when GP2 is set for GPIO operation.
    ///
    /// Byte 6 bit 3.
    pub power_up_direction: GpioDirection,
    /// GP2 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 6 bits 0..=2.
    pub power_up_designation: Gp2Designation,
}

#[derive(Debug)]
pub struct Gp3Settings {
    /// GP3 power-up output value.
    ///
    /// When GP3 is set as an output GPIO, this value will be present at
    /// the GP3 pin at power-up/reset.
    ///
    /// Byte 7 bit 4.
    pub power_up_value: LogicLevel,
    /// GP3 power-up direction.
    ///
    /// Works only when GP3 is set for GPIO operation.
    ///
    /// Byte 7 bit 3.
    pub power_up_direction: GpioDirection,
    /// GP3 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 7 bits 0..=2.
    pub power_up_designation: Gp3Designation,
}

impl From<(LogicLevel, GpioDirection, Gp0Designation)> for Gp0Settings {
    fn from(
        (power_up_value, power_up_direction, power_up_designation): (
            LogicLevel,
            GpioDirection,
            Gp0Designation,
        ),
    ) -> Self {
        Self {
            power_up_value,
            power_up_direction,
            power_up_designation,
        }
    }
}

impl From<(LogicLevel, GpioDirection, Gp1Designation)> for Gp1Settings {
    fn from(
        (power_up_value, power_up_direction, power_up_designation): (
            LogicLevel,
            GpioDirection,
            Gp1Designation,
        ),
    ) -> Self {
        Self {
            power_up_value,
            power_up_direction,
            power_up_designation,
        }
    }
}

impl From<(LogicLevel, GpioDirection, Gp2Designation)> for Gp2Settings {
    fn from(
        (power_up_value, power_up_direction, power_up_designation): (
            LogicLevel,
            GpioDirection,
            Gp2Designation,
        ),
    ) -> Self {
        Self {
            power_up_value,
            power_up_direction,
            power_up_designation,
        }
    }
}

impl From<(LogicLevel, GpioDirection, Gp3Designation)> for Gp3Settings {
    fn from(
        (power_up_value, power_up_direction, power_up_designation): (
            LogicLevel,
            GpioDirection,
            Gp3Designation,
        ),
    ) -> Self {
        Self {
            power_up_value,
            power_up_direction,
            power_up_designation,
        }
    }
}
