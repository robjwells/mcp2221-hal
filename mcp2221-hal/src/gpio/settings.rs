use bit_field::BitField;

use crate::Error;

use super::{GpioDirection, LogicLevel, PinNumber};

/// The source of the GP settings determines their position in the read buffer.
enum GpSettingsSource {
    /// GP settings at bytes 4..=7
    Flash,
    /// GP settings at bytes 22..=25
    Sram,
}

/// General-purpose pins settings.
///
/// Each of the four GP pins supports GPIO operation, one exclusive ("dedicated")
/// function, and "alternate" functions (ADC and DAC).
///
/// See table 1-5 of the datasheet for an overview, section 1.7 for the details of
/// each function, and section 1.6.2.4 for use of the interrupt pin function.
#[derive(Debug)]
pub struct GpSettings {
    /// GP pin 0.
    pub gp0: Gp0Settings,
    /// GP pin 1.
    pub gp1: Gp1Settings,
    /// GP pin 2.
    pub gp2: Gp2Settings,
    /// GP pin 3.
    pub gp3: Gp3Settings,
}

impl GpSettings {
    /// Parse GP pin settings read from flash memory.
    pub fn try_from_flash_buffer(buf: &[u8; 64]) -> Result<Self, Error> {
        GpSettings::try_from_buffer(GpSettingsSource::Flash, buf)
    }

    /// Parse GP pin settings read from SRAM.
    pub fn try_from_sram_buffer(buf: &[u8; 64]) -> Result<Self, Error> {
        GpSettings::try_from_buffer(GpSettingsSource::Sram, buf)
    }

    fn try_from_buffer(source: GpSettingsSource, buf: &[u8; 64]) -> Result<Self, Error> {
        let start_byte = match source {
            GpSettingsSource::Flash => 4,
            GpSettingsSource::Sram => 22,
        };

        Ok(Self {
            gp0: Gp0Settings::new(
                buf[start_byte].get_bit(4).into(),
                buf[start_byte].get_bit(3).into(),
                buf[start_byte].get_bits(0..=2).try_into()?,
            ),
            gp1: Gp1Settings::new(
                buf[start_byte + 1].get_bit(4).into(),
                buf[start_byte + 1].get_bit(3).into(),
                buf[start_byte + 1].get_bits(0..=2).try_into()?,
            ),
            gp2: Gp2Settings::new(
                buf[start_byte + 2].get_bit(4).into(),
                buf[start_byte + 2].get_bit(3).into(),
                buf[start_byte + 2].get_bits(0..=2).try_into()?,
            ),
            gp3: Gp3Settings::new(
                buf[start_byte + 3].get_bit(4).into(),
                buf[start_byte + 3].get_bit(3).into(),
                buf[start_byte + 3].get_bits(0..=2).try_into()?,
            ),
        })
    }

    pub(crate) fn configure_as_gpio(&mut self, pin: PinNumber, direction: GpioDirection) {
        match pin {
            PinNumber::Gp0 => {
                self.gp0.designation = Gp0Designation::GPIO;
                self.gp0.direction = direction;
            }
            PinNumber::Gp1 => {
                self.gp1.designation = Gp1Designation::GPIO;
                self.gp1.direction = direction;
            }
            PinNumber::Gp2 => {
                self.gp2.designation = Gp2Designation::GPIO;
                self.gp2.direction = direction;
            }
            PinNumber::Gp3 => {
                self.gp3.designation = Gp3Designation::GPIO;
                self.gp3.direction = direction;
            }
        }
    }

    pub(crate) fn apply_to_flash_buffer(&self, buf: &mut [u8; 64]) {
        // Byte 2 -- GP0
        buf[2].set_bit(4, self.gp0.value.into());
        buf[2].set_bit(3, self.gp0.direction.into());
        buf[2].set_bits(0..=2, self.gp0.designation.into());

        // Byte 3 -- GP1
        buf[3].set_bit(4, self.gp1.value.into());
        buf[3].set_bit(3, self.gp1.direction.into());
        buf[3].set_bits(0..=2, self.gp1.designation.into());

        // Byte 4 -- GP2
        buf[4].set_bit(4, self.gp2.value.into());
        buf[4].set_bit(3, self.gp2.direction.into());
        buf[4].set_bits(0..=2, self.gp2.designation.into());

        // Byte 5 -- GP3
        buf[5].set_bit(4, self.gp3.value.into());
        buf[5].set_bit(3, self.gp3.direction.into());
        buf[5].set_bits(0..=2, self.gp3.designation.into());
    }
}

/// Helper to return invalid pin mode errors
#[doc(hidden)]
macro_rules! pin_mode_err {
    ($pin:literal, $mode:ident) => {
        return Err(Error::InvalidPinModeFromDevice {
            pin: $pin,
            mode: $mode,
        })
    };
}

/// GP pin 0 operation mode.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp0Designation {
    /// Indicates UART traffic received by the MCP2221.
    ///
    /// This pin will pulse low for a few milliseconds to provide a visual indication
    /// of the UART Rx traffic. See section 1.7.1.4.
    LED_UART_RX,
    /// USB Suspend state.
    ///
    /// Reflects the USB state (Suspend/Resume). This pin is active-low when the Suspend
    /// state has been issued by the USB host. The pin drives high after the Resume
    /// state is achieved. See section 1.7.1.2.
    SSPND,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or a digital output.
    GPIO,
}

#[doc(hidden)]
impl TryFrom<u8> for Gp0Designation {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b010 => Self::LED_UART_RX,
            0b001 => Self::SSPND,
            0b000 => Self::GPIO,
            _ => pin_mode_err!("GP0", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp0Designation> for u8 {
    fn from(value: Gp0Designation) -> Self {
        match value {
            Gp0Designation::SSPND => 0b010,
            Gp0Designation::LED_UART_RX => 0b001,
            Gp0Designation::GPIO => 0b000,
        }
    }
}

/// GP pin 1 operation mode.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp1Designation {
    /// Digital clock output.
    ///
    /// The nominal frequency is 12MHz (the MCP2221's internal clock speed), ±0.25%,
    /// but other frequencies and duty cycles are possible. See register 1-2 for the
    /// values for these, as well as the flash and SRAM settings command sections.
    /// See section 1.9.
    ClockOutput,
    /// Interrupt-on-change.
    ///
    /// This mode makes GP1 sensitive to positive and negative edges. Interrupts can
    /// be triggered on either or both. See section 1.10.
    InterruptDetection,
    /// Indicates UART traffic sent by the MCP2221.
    ///
    /// This pin will pulse low for a few milliseconds to provide a visual indication
    /// of the UART Tx traffic. See section 1.7.1.5.
    LED_UART_TX,
    /// Analog-to-digital channel 1.
    ///
    /// Sets GP1 to an analog input tied to the first channel of the 10-bit ADC. See
    /// section 1.8.2.
    ADC1,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or a digital output.
    GPIO,
}

#[doc(hidden)]
impl TryFrom<u8> for Gp1Designation {
    type Error = Error;
    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        // Note the case order here matches the order in the datasheet.
        Ok(match mode {
            0b001 => Self::ClockOutput,
            0b100 => Self::InterruptDetection,
            0b011 => Self::LED_UART_TX,
            0b010 => Self::ADC1,
            0b000 => Self::GPIO,
            _ => pin_mode_err!("GP1", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp1Designation> for u8 {
    fn from(value: Gp1Designation) -> Self {
        match value {
            Gp1Designation::InterruptDetection => 0b100,
            Gp1Designation::LED_UART_TX => 0b11,
            Gp1Designation::ADC1 => 0b010,
            Gp1Designation::ClockOutput => 0b001,
            Gp1Designation::GPIO => 0b000,
        }
    }
}

/// GP pin 2 operation mode.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp2Designation {
    /// Digital-to-analog.
    ///
    /// Sets GP2 to an analog output tied to the output of the 5-bit DAC. Note there
    /// is only one DAC output, so if both GP2 and GP3 are configured as DAC pins
    /// they will output the same value. See section 1.8.3.
    DAC1,
    /// Analog-to-digital channel 2.
    ///
    /// Sets GP2 to an analog input tied to the second channel of the 10-bit ADC.
    /// See section 1.8.2.
    ADC2,
    /// USB configure pin.
    ///
    /// This pin starts out low during power-up or after reset and goes high after the
    /// device successfully configures to the USB. The pin will go low when in Suspend
    /// mode and high when the USB resumes. See section 1.7.1.3.
    USBCFG,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or a digital output.
    GPIO,
}

#[doc(hidden)]
impl TryFrom<u8> for Gp2Designation {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b011 => Self::DAC1,
            0b010 => Self::ADC2,
            0b001 => Self::USBCFG,
            0b000 => Self::GPIO,
            _ => pin_mode_err!("GP2", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp2Designation> for u8 {
    fn from(value: Gp2Designation) -> Self {
        // The datasheet incorrectly lists "clock output" when writing the GP2 settings
        // but it should be USBCFG.
        match value {
            Gp2Designation::DAC1 => 0b011,
            Gp2Designation::ADC2 => 0b010,
            Gp2Designation::USBCFG => 0b001,
            Gp2Designation::GPIO => 0b000,
        }
    }
}

/// GP pin 3 operation mode.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Gp3Designation {
    /// Digital-to-analog.
    ///
    /// Sets GP3 to an analog output tied to the output of the 5-bit DAC. Note there
    /// is only one DAC output, so if both GP2 and GP3 are configured as DAC pins
    /// they will output the same value. See section 1.8.3.
    DAC2,
    /// Analog-to-digital channel 3.
    ///
    /// Sets GP3 to an analog input tied to the third channel of the 10-bit ADC.
    /// See section 1.8.2.
    ADC3,
    /// Indicates I2C activity.
    ///
    /// This pin will pulse low for a few milliseconds to provide a visual indication
    /// of the I2C traffic. See section 1.7.1.6.
    LED_I2C,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or a digital output.
    GPIO,
}

#[doc(hidden)]
impl TryFrom<u8> for Gp3Designation {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b011 => Self::DAC2,
            0b010 => Self::ADC3,
            0b001 => Self::LED_I2C,
            0b000 => Self::GPIO,
            _ => pin_mode_err!("GP3", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp3Designation> for u8 {
    fn from(value: Gp3Designation) -> Self {
        match value {
            Gp3Designation::DAC2 => 0b011,
            Gp3Designation::ADC3 => 0b010,
            Gp3Designation::LED_I2C => 0b001,
            Gp3Designation::GPIO => 0b000,
        }
    }
}

/// Helper to generate the constructors for each of the pin settings structs.
#[doc(hidden)]
macro_rules! pin_settings_new {
    ($gp_settings_type:ty, $mode_type:ty) => {
        impl $gp_settings_type {
            fn new(value: LogicLevel, direction: GpioDirection, designation: $mode_type) -> Self {
                Self {
                    value,
                    direction,
                    designation,
                }
            }
        }
    };
}

/// GP pin 0 configuration.
#[derive(Debug)]
pub struct Gp0Settings {
    /// GP0 power-up output value.
    ///
    /// When GP0 is set as an output GPIO, this value will be present at
    /// the GP0 pin at power-up/reset.
    ///
    /// Byte 4 bit 4.
    pub value: LogicLevel,
    /// GP0 power-up direction.
    ///
    /// Works only when GP0 is set for GPIO operation.
    ///
    /// Byte 4 bit 3.
    pub direction: GpioDirection,
    /// GP0 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 4 bits 0..=2.
    pub designation: Gp0Designation,
}

pin_settings_new!(Gp0Settings, Gp0Designation);

/// GP pin 1 configuration.
#[derive(Debug)]
pub struct Gp1Settings {
    /// GP1 power-up output value.
    ///
    /// When GP1 is set as an output GPIO, this value will be present at
    /// the GP1 pin at power-up/reset.
    ///
    /// Byte 5 bit 4.
    pub value: LogicLevel,
    /// GP1 power-up direction.
    ///
    /// Works only when GP1 is set for GPIO operation.
    ///
    /// Byte 5 bit 3.
    pub direction: GpioDirection,
    /// GP1 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 5 bits 0..=2.
    pub designation: Gp1Designation,
}

pin_settings_new!(Gp1Settings, Gp1Designation);

impl Gp1Settings {
    /// Returns `true` if GP1 is set as an analog input.
    #[must_use]
    pub(crate) fn is_adc(&self) -> bool {
        matches!(self.designation, Gp1Designation::ADC1)
    }
}

/// GP pin 2 configuration.
#[derive(Debug)]
pub struct Gp2Settings {
    /// GP2 power-up output value.
    ///
    /// When GP2 is set as an output GPIO, this value will be present at
    /// the GP2 pin at power-up/reset.
    ///
    /// Byte 6 bit 4.
    pub value: LogicLevel,
    /// GP2 power-up direction.
    ///
    /// Works only when GP2 is set for GPIO operation.
    ///
    /// Byte 6 bit 3.
    pub direction: GpioDirection,
    /// GP2 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 6 bits 0..=2.
    pub designation: Gp2Designation,
}

pin_settings_new!(Gp2Settings, Gp2Designation);

impl Gp2Settings {
    /// Returns `true` if GP2 is set as an analog input.
    #[must_use]
    pub(crate) fn is_adc(&self) -> bool {
        matches!(self.designation, Gp2Designation::ADC2)
    }
}

/// GP pin 3 configuration.
#[derive(Debug)]
pub struct Gp3Settings {
    /// GP3 power-up output value.
    ///
    /// When GP3 is set as an output GPIO, this value will be present at
    /// the GP3 pin at power-up/reset.
    ///
    /// Byte 7 bit 4.
    pub value: LogicLevel,
    /// GP3 power-up direction.
    ///
    /// Works only when GP3 is set for GPIO operation.
    ///
    /// Byte 7 bit 3.
    pub direction: GpioDirection,
    /// GP3 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 7 bits 0..=2.
    pub designation: Gp3Designation,
}

pin_settings_new!(Gp3Settings, Gp3Designation);

impl Gp3Settings {
    /// Returns `true` if GP3 is set as an analog input.
    #[must_use]
    pub(crate) fn is_adc(&self) -> bool {
        matches!(self.designation, Gp3Designation::ADC3)
    }
}
