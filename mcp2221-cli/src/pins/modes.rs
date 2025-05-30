use clap::{Parser, ValueEnum};
use mcp2221_hal::gpio::{GpioDirection, LogicLevel};
use mcp2221_hal::settings::{self as hal, GpSettings};

/// Set the mode for each of the GPx pins.
///
/// Each pin supports digital input and output, as well as pin-specific
/// alternate functions (aka designations). If the pin is set to digital
/// output, its output value is also set.
///
/// For GPIO (digital) input and output, the following aliases are recognised
/// for each pin and are not repeated in the per-option help text:
///
/// - gpio-output-high:  high
/// - gpio-output-low:   low
/// - gpio-input:        input, in
///
/// Further aliases are available for each function either as a convenience or
/// to match the pin function name(s) in the datasheet.
#[derive(Debug, Parser)]
#[command(verbatim_doc_comment)]
pub(crate) struct GpModes {
    #[arg(long, default_value = "false")]
    /// Set the GP pin configuration in flash memory rather than SRAM.
    ///
    /// This will not change the current GP pin configuration in SRAM,
    /// and will only be observed after resetting the MCP2221.
    pub flash: bool,
    #[command(flatten)]
    pub pin_configs: PinModes,
}

#[derive(Debug, Parser)]
#[group(required = true, multiple = true)]
pub(crate) struct PinModes {
    /// GP0 pin settings
    ///
    /// Note the following additional aliases:
    ///
    /// - uart-receive-led:  led_uart_rx, led_urx
    /// - usb-suspend-state: suspend, sspnd
    #[arg(short = '0', long, id = "GP0_MODE", verbatim_doc_comment)]
    pub gp0: Option<Gp0Mode>,
    /// GP1 pin settings
    ///
    /// Note the following additional aliases:
    ///
    /// - clock-output:       clkr, clock
    /// - analog-input:       adc, adc1
    /// - uart-transmit-led:  led_uart_tx
    /// - interrupt:          ioc
    #[arg(short = '1', long, id = "GP1_MODE", verbatim_doc_comment)]
    pub gp1: Option<Gp1Mode>,
    /// GP2 pin settings
    ///
    /// Note the following additional aliases:
    ///
    /// - usb-device-configured: usbcfg
    /// - analog-input:          adc, adc2
    /// - analog-output:         dac
    #[arg(short = '2', long, id = "GP2_MODE", verbatim_doc_comment)]
    pub gp2: Option<Gp2Mode>,
    /// GP3 pin settings
    ///
    /// Note the following additional aliases:
    ///
    /// - i2c-led:        led_i2c
    /// - analog-input:   adc, adc3
    /// - analog-output:  dac
    #[arg(short = '3', long, id = "GP3_MODE", verbatim_doc_comment)]
    pub gp3: Option<Gp3Mode>,
}

impl PinModes {
    pub(crate) fn merge_into_existing(&self, settings: &mut GpSettings) {
        if let Some(gp0) = self.gp0 {
            let (mode, level, direction) = gp0.components();
            settings.gp0_mode = mode;
            if let Some(level) = level {
                settings.gp0_value = level;
            }
            if let Some(direction) = direction {
                settings.gp0_direction = direction;
            }
        }
        if let Some(gp1) = self.gp1 {
            let (mode, level, direction) = gp1.components();
            settings.gp1_mode = mode;
            if let Some(level) = level {
                settings.gp1_value = level;
            }
            if let Some(direction) = direction {
                settings.gp1_direction = direction;
            }
        }
        if let Some(gp2) = self.gp2 {
            let (mode, level, direction) = gp2.components();
            settings.gp2_mode = mode;
            if let Some(level) = level {
                settings.gp2_value = level;
            }
            if let Some(direction) = direction {
                settings.gp2_direction = direction;
            }
        }
        if let Some(gp3) = self.gp3 {
            let (mode, level, direction) = gp3.components();
            settings.gp3_mode = mode;
            if let Some(level) = level {
                settings.gp3_value = level;
            }
            if let Some(direction) = direction {
                settings.gp3_direction = direction;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Gp0Mode {
    /// UART receive indicator (LED_UART_RX)
    #[value(aliases = ["led_uart_rx", "led_urx"])]
    UartReceiveLed,
    /// USB Suspend state indicator (SSPND)
    #[value(aliases = ["suspend", "sspnd"])]
    UsbSuspendState,
    /// Digital output, set high.
    #[value(aliases = ["high"])]
    GpioOutputHigh,
    /// Digital output, set low.
    #[value(aliases = ["low"])]
    GpioOutputLow,
    /// Digital input.
    #[value(aliases = ["input", "in"])]
    GpioInput,
}

impl Gp0Mode {
    fn components(&self) -> (hal::Gp0Mode, Option<LogicLevel>, Option<GpioDirection>) {
        use hal::Gp0Mode::*;
        match self {
            Gp0Mode::UartReceiveLed => (UartReceiveIndicator, None, None),
            Gp0Mode::UsbSuspendState => (UsbSuspendState, None, None),
            Gp0Mode::GpioOutputHigh => (Gpio, Some(LogicLevel::High), Some(GpioDirection::Output)),
            Gp0Mode::GpioOutputLow => (Gpio, Some(LogicLevel::Low), Some(GpioDirection::Output)),
            Gp0Mode::GpioInput => (Gpio, None, Some(GpioDirection::Input)),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Gp1Mode {
    /// Clock reference output (CLKR).
    #[value(aliases = ["clock", "clkr"])]
    ClockOutput,
    /// Analog input (ADC channel 1).
    #[value(aliases = ["adc", "adc1"])]
    AnalogInput,
    /// Indicates UART traffic sent by the MCP2221.
    #[value(aliases = ["led_uart_tx", "led_utx"])]
    UartTransmitLed,
    /// Edge-triggered interrupt detection (IOC).
    Interrupt,
    /// Digital output, set high.
    #[value(aliases = ["high"])]
    GpioOutputHigh,
    /// Digital output, set low.
    #[value(aliases = ["low"])]
    GpioOutputLow,
    /// Digital input.
    #[value(aliases = ["input", "in"])]
    GpioInput,
}

impl Gp1Mode {
    fn components(&self) -> (hal::Gp1Mode, Option<LogicLevel>, Option<GpioDirection>) {
        use hal::Gp1Mode::*;
        match self {
            Gp1Mode::ClockOutput => (ClockOutput, None, None),
            Gp1Mode::AnalogInput => (AnalogInput, None, None),
            Gp1Mode::UartTransmitLed => (UartTransmitIndicator, None, None),
            Gp1Mode::Interrupt => (InterruptDetection, None, None),
            Gp1Mode::GpioOutputHigh => (Gpio, Some(LogicLevel::High), Some(GpioDirection::Output)),
            Gp1Mode::GpioOutputLow => (Gpio, Some(LogicLevel::Low), Some(GpioDirection::Output)),
            Gp1Mode::GpioInput => (Gpio, None, Some(GpioDirection::Input)),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Gp2Mode {
    /// USB device-configured status indicator (USBCFG).
    #[value(aliases = ["usbcfg"])]
    UsbDeviceConfigured,
    /// Analog input (ADC channel 2).
    #[value(aliases = ["adc", "adc2"])]
    AnalogInput,
    /// Analog output (DAC).
    #[value(aliases = ["dac"])]
    AnalogOutput,
    /// Digital output, set high.
    #[value(aliases = ["high"])]
    GpioOutputHigh,
    /// Digital output, set low.
    #[value(aliases = ["low"])]
    GpioOutputLow,
    /// Digital input.
    #[value(aliases = ["input", "in"])]
    GpioInput,
}

impl Gp2Mode {
    fn components(&self) -> (hal::Gp2Mode, Option<LogicLevel>, Option<GpioDirection>) {
        use hal::Gp2Mode::*;
        match self {
            Gp2Mode::GpioOutputHigh => (Gpio, Some(LogicLevel::High), Some(GpioDirection::Output)),
            Gp2Mode::GpioOutputLow => (Gpio, Some(LogicLevel::Low), Some(GpioDirection::Output)),
            Gp2Mode::GpioInput => (Gpio, None, Some(GpioDirection::Input)),
            Gp2Mode::UsbDeviceConfigured => (UsbDeviceConfiguredStatus, None, None),
            Gp2Mode::AnalogInput => (AnalogInput, None, None),
            Gp2Mode::AnalogOutput => (AnalogOutput, None, None),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Gp3Mode {
    /// I2C activity indicator (LED_I2C).
    #[value(aliases = ["led_i2c"])]
    I2cLed,
    /// Analog input (ADC channel 3).
    #[value(aliases = ["adc", "adc3"])]
    AnalogInput,
    /// Analog output (DAC).
    #[value(aliases = ["dac"])]
    AnalogOutput,
    /// Digital output, set high.
    #[value(aliases = ["high"])]
    GpioOutputHigh,
    /// Digital output, set low.
    #[value(aliases = ["low"])]
    GpioOutputLow,
    /// Digital input.
    #[value(aliases = ["input", "in"])]
    GpioInput,
}

impl Gp3Mode {
    fn components(&self) -> (hal::Gp3Mode, Option<LogicLevel>, Option<GpioDirection>) {
        use hal::Gp3Mode::*;
        match self {
            Gp3Mode::GpioOutputHigh => (Gpio, Some(LogicLevel::High), Some(GpioDirection::Output)),
            Gp3Mode::GpioOutputLow => (Gpio, Some(LogicLevel::Low), Some(GpioDirection::Output)),
            Gp3Mode::GpioInput => (Gpio, None, Some(GpioDirection::Input)),
            Gp3Mode::I2cLed => (I2cActivityIndicator, None, None),
            Gp3Mode::AnalogInput => (AnalogInput, None, None),
            Gp3Mode::AnalogOutput => (AnalogOutput, None, None),
        }
    }
}
