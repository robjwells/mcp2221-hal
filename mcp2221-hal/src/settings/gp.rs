use crate::Error;
use crate::gpio::{GpioDirection, LogicLevel};

use bit_field::BitField;

/// GP pin settings.
///
/// The MCP2221 has four general-purpose (GP) pins, which can each be configured for a
/// number of different functions. All support standard GPIO digital input and output,
/// but the particular modes available differ by pin. See the individual pin mode enums
/// for further details about the modes available.
///
/// The MCP2221 always reports the pin direction and pin value, even though direction is
/// only used in GPIO mode, and value is only used in GPIO output mode. Likewise, these
/// can be changed but only have an effect in the appropriate GPIO mode.
///
/// <div class="warning">
///
/// If GP pin GPIO direction or output value are changed via [`MCP2221::gpio_write`],
/// then the GP settings read from SRAM may not reflect the current configuration of
/// the GP pins. This appears to be a bug in the MCP2221 firmware.
///
/// [`MCP2221::gpio_write`]: crate::MCP2221::gpio_write
///
/// </div>
///
/// ## Datasheet
///
/// See table 1-5 of the datasheet for an overview of the pin modes available for each
/// pin and section 1.7 for the details of each function.
#[derive(Debug)]
pub struct GpSettings {
    /// GP0 pin mode.
    pub gp0_mode: Gp0Mode,
    /// GP1 pin mode.
    pub gp1_mode: Gp1Mode,
    /// GP2 pin mode.
    pub gp2_mode: Gp2Mode,
    /// GP3 pin mode
    pub gp3_mode: Gp3Mode,

    /// GP0 GPIO output value.
    pub gp0_value: LogicLevel,
    /// GP0 GPIO direction.
    pub gp0_direction: GpioDirection,

    /// GP1 GPIO output value
    pub gp1_value: LogicLevel,
    /// GP1 GPIO direction.
    pub gp1_direction: GpioDirection,

    /// GP2 GPIO output value.
    pub gp2_value: LogicLevel,
    /// GP2 GPIO direction.
    pub gp2_direction: GpioDirection,

    /// GP3 GPIO output value
    pub gp3_value: LogicLevel,
    /// GP3 GPIO output direction.
    pub gp3_direction: GpioDirection,
}

/// In the Read Flash Data - Read GP Settings HID command response, the GP settings
/// start at byte 4.
///
/// ## Datasheet
///
/// See table 3-6 in section 3.1.2.1.
const FLASH_START_BYTE: usize = 4;
/// In the Get SRAM Settings HID command response, the GP settings start at byte 22.
///
/// ## Datasheet
///
/// See table 3-39 in section 3.1.14.1.
const SRAM_START_BYTE: usize = 22;

impl GpSettings {
    /// Parse GP pin settings read from flash memory.
    pub(crate) fn try_from_flash_buffer(buf: &[u8; 64]) -> Result<Self, Error> {
        GpSettings::try_from_buffer(FLASH_START_BYTE, buf)
    }

    /// Parse GP pin settings read from SRAM.
    pub(crate) fn try_from_sram_buffer(buf: &[u8; 64]) -> Result<Self, Error> {
        GpSettings::try_from_buffer(SRAM_START_BYTE, buf)
    }

    /// Parse the GP settings contained in each byte.
    ///
    /// ## Datasheet
    ///
    /// See table 3-6 for the flash response layout, and table 3-39 for the
    /// SRAM response layout. The layouts are the same but start at different
    /// byte offsets.
    fn try_from_buffer(start_byte: usize, buf: &[u8; 64]) -> Result<Self, Error> {
        Ok(Self {
            // GP0, byte 4/22 (flash/SRAM)
            gp0_value: buf[start_byte].get_bit(4).into(),
            gp0_direction: buf[start_byte].get_bit(3).into(),
            gp0_mode: buf[start_byte].get_bits(0..=2).try_into()?,

            // GP1, byte 5/23
            gp1_value: buf[start_byte + 1].get_bit(4).into(),
            gp1_direction: buf[start_byte + 1].get_bit(3).into(),
            gp1_mode: buf[start_byte + 1].get_bits(0..=2).try_into()?,

            // GP2, byte 6/24
            gp2_value: buf[start_byte + 2].get_bit(4).into(),
            gp2_direction: buf[start_byte + 2].get_bit(3).into(),
            gp2_mode: buf[start_byte + 2].get_bits(0..=2).try_into()?,

            // GP3, byte 7/25
            gp3_value: buf[start_byte + 3].get_bit(4).into(),
            gp3_direction: buf[start_byte + 3].get_bit(3).into(),
            gp3_mode: buf[start_byte + 3].get_bits(0..=2).try_into()?,
        })
    }

    /// Apply the settings to a buffer for writing to flash memory.
    ///
    /// ## Datasheet
    ///
    /// See table 3-13 for the buffer layout. Note that GP settings in SRAM
    /// are changed in a completely different manner.[]
    pub(crate) fn apply_to_flash_buffer(&self, buf: &mut [u8; 64]) {
        // Byte 2 -- GP0
        buf[2].set_bit(4, self.gp0_value.into());
        buf[2].set_bit(3, self.gp0_direction.into());
        buf[2].set_bits(0..=2, self.gp0_mode.into());

        // Byte 3 -- GP1
        buf[3].set_bit(4, self.gp1_value.into());
        buf[3].set_bit(3, self.gp1_direction.into());
        buf[3].set_bits(0..=2, self.gp1_mode.into());

        // Byte 4 -- GP2
        buf[4].set_bit(4, self.gp2_value.into());
        buf[4].set_bit(3, self.gp2_direction.into());
        buf[4].set_bits(0..=2, self.gp2_mode.into());

        // Byte 5 -- GP3
        buf[5].set_bit(4, self.gp3_value.into());
        buf[5].set_bit(3, self.gp3_direction.into());
        buf[5].set_bits(0..=2, self.gp3_mode.into());
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

/// GP0 pin function mode.
///
/// GP0 is the most limited of the pins, and does not support analog input or output.
///
/// The short pin function names used in the datasheet are indicated in parentheses.
///
/// See section 1.7 and table 1-5 of the datasheet for information on the pin modes,
/// and which modes can be used by each pin.
#[derive(Debug, Clone, Copy)]
pub enum Gp0Mode {
    /// UART traffic received indicator (LED_URX).
    ///
    /// The pin will pulse low for a few milliseconds to provide a visual indication
    /// of the UART receive traffic. See section 1.7.1.4 of the datasheet.
    UartReceiveIndicator,
    /// USB Suspend state (SSPND).
    ///
    /// Reflects the USB state (Suspend/Resume). This pin is active-low when the Suspend
    /// state has been issued by the USB host. The pin drives high after the Resume
    /// state is achieved. See section 1.7.1.2 of the datasheet.
    UsbSuspendState,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or output.
    Gpio,
}

#[doc(hidden)]
impl TryFrom<u8> for Gp0Mode {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b010 => Self::UartReceiveIndicator,
            0b001 => Self::UsbSuspendState,
            0b000 => Self::Gpio,
            _ => pin_mode_err!("GP0", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp0Mode> for u8 {
    fn from(value: Gp0Mode) -> Self {
        match value {
            Gp0Mode::UsbSuspendState => 0b010,
            Gp0Mode::UartReceiveIndicator => 0b001,
            Gp0Mode::Gpio => 0b000,
        }
    }
}

/// GP1 pin function mode.
///
/// The short pin function names used in the datasheet are indicated in parentheses.
///
/// See section 1.7 and table 1-5 of the datasheet for information on the pin modes,
/// and which modes can be used by each pin.
#[derive(Debug, Clone, Copy)]
pub enum Gp1Mode {
    /// Digital clock output (CLK_OUT or CLKR).
    ///
    /// The nominal frequency is 12MHz (the MCP2221's internal clock speed), ±0.25%,
    /// but other frequencies and duty cycles are possible. See register 1-2 for the
    /// values for these, as well as the flash and SRAM settings command sections.
    /// See section 1.9.
    ClockOutput,
    /// Interrupt-on-change (IOC).
    ///
    /// This mode makes GP1 sensitive to positive and negative edges. Interrupts can
    /// be triggered on either or both. See section 1.10 of the datasheet.
    InterruptDetection,
    /// UART traffic transmitted indicator (LED_UTX).
    ///
    /// This pin will pulse low for a few milliseconds to provide a visual indication
    /// of the UART transmit traffic. See section 1.7.1.5 of the datasheet.
    UartTransmitIndicator,
    /// Analog-to-digital channel 1 (ADC1).
    ///
    /// Sets GP1 to an analog input connected to the first channel of the 10-bit ADC.
    /// See section 1.8.2 of the datasheet.
    AnalogInput,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or output.
    Gpio,
}

impl Gp1Mode {
    /// Returns true if GP1 is configured as an analog input.
    pub(crate) fn is_analog_input(&self) -> bool {
        matches!(self, Self::AnalogInput)
    }
}

#[doc(hidden)]
impl TryFrom<u8> for Gp1Mode {
    type Error = Error;
    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        // Note the case order here matches the order in the datasheet.
        Ok(match mode {
            0b001 => Self::ClockOutput,
            0b100 => Self::InterruptDetection,
            0b011 => Self::UartTransmitIndicator,
            0b010 => Self::AnalogInput,
            0b000 => Self::Gpio,
            _ => pin_mode_err!("GP1", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp1Mode> for u8 {
    fn from(value: Gp1Mode) -> Self {
        match value {
            Gp1Mode::InterruptDetection => 0b100,
            Gp1Mode::UartTransmitIndicator => 0b11,
            Gp1Mode::AnalogInput => 0b010,
            Gp1Mode::ClockOutput => 0b001,
            Gp1Mode::Gpio => 0b000,
        }
    }
}

/// GP2 pin function mode.
///
/// The short pin function names used in the datasheet are indicated in parentheses.
///
/// See section 1.7 and table 1-5 of the datasheet for information on the pin modes,
/// and which modes can be used by each pin.
#[derive(Debug, Clone, Copy)]
pub enum Gp2Mode {
    /// Digital-to-analog (DAC1).
    ///
    /// Sets GP2 to an analog output connected to the output of the 5-bit DAC. Note
    /// there is only one DAC output, so if both GP2 and GP3 are configured as analog
    /// output pins they will have the same value. See section 1.8.3 of the datasheet.
    AnalogOutput,
    /// Analog-to-digital channel 2 (ADC2).
    ///
    /// Sets GP2 to an analog input connected to the second channel of the 10-bit ADC.
    /// See section 1.8.2 of the datasheet.
    AnalogInput,
    /// USB device-configured status (USBCFG).
    ///
    /// This pin starts out low during power-up or after reset and goes high after the
    /// device successfully configures to the USB. The pin will go low when in Suspend
    /// mode and high when the USB resumes. See section 1.7.1.3 of the datasheet.
    UsbDeviceConfiguredStatus,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or output.
    Gpio,
}

impl Gp2Mode {
    /// Returns true if GP2 is configured as an analog input.
    pub(crate) fn is_analog_input(&self) -> bool {
        matches!(self, Self::AnalogInput)
    }
}

#[doc(hidden)]
impl TryFrom<u8> for Gp2Mode {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b011 => Self::AnalogOutput,
            0b010 => Self::AnalogInput,
            0b001 => Self::UsbDeviceConfiguredStatus,
            0b000 => Self::Gpio,
            _ => pin_mode_err!("GP2", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp2Mode> for u8 {
    fn from(value: Gp2Mode) -> Self {
        // The datasheet incorrectly lists "clock output" when writing the GP2 settings
        // but it should be USBCFG.
        match value {
            Gp2Mode::AnalogOutput => 0b011,
            Gp2Mode::AnalogInput => 0b010,
            Gp2Mode::UsbDeviceConfiguredStatus => 0b001,
            Gp2Mode::Gpio => 0b000,
        }
    }
}

/// GP3 pin function mode.
///
/// The short pin function names used in the datasheet are indicated in parentheses.
///
/// See section 1.7 and table 1-5 of the datasheet for information on the pin modes,
/// and which modes can be used by each pin.
#[derive(Debug, Clone, Copy)]
pub enum Gp3Mode {
    /// Digital-to-analog (DAC2).
    ///
    /// Sets GP3 to an analog output connected to the output of the 5-bit DAC. Note
    /// there is only one DAC output, so if both GP2 and GP3 are configured as analog
    /// output pins they will have the same value. See section 1.8.3 of the datasheet.
    AnalogOutput,
    /// Analog-to-digital channel 3 (ADC3).
    ///
    /// Sets GP3 to an analog input connected to the third channel of the 10-bit ADC.
    /// See section 1.8.2 of the datasheet.
    AnalogInput,
    /// Indicates I2C activity (LED_I2C).
    ///
    /// This pin will pulse low for a few milliseconds to provide a visual indication
    /// of the I2C traffic. See section 1.7.1.6 of the datasheet.
    I2cActivityIndicator,
    /// GPIO pin function.
    ///
    /// The pin operates as a digital input or output.
    GPIO,
}

impl Gp3Mode {
    /// Returns true if GP2 is configured as an analog input.
    pub(crate) fn is_analog_input(&self) -> bool {
        matches!(self, Self::AnalogInput)
    }
}

#[doc(hidden)]
impl TryFrom<u8> for Gp3Mode {
    type Error = Error;

    fn try_from(mode: u8) -> Result<Self, Error> {
        assert!(mode <= 0b111, "Incorrect use of the from constructor.");
        Ok(match mode {
            0b011 => Self::AnalogOutput,
            0b010 => Self::AnalogInput,
            0b001 => Self::I2cActivityIndicator,
            0b000 => Self::GPIO,
            _ => pin_mode_err!("GP3", mode),
        })
    }
}

#[doc(hidden)]
impl From<Gp3Mode> for u8 {
    fn from(value: Gp3Mode) -> Self {
        match value {
            Gp3Mode::AnalogOutput => 0b011,
            Gp3Mode::AnalogInput => 0b010,
            Gp3Mode::I2cActivityIndicator => 0b001,
            Gp3Mode::GPIO => 0b000,
        }
    }
}
