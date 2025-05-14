use clap::{Parser, ValueEnum};
use mcp2221_hal::gpio;

#[derive(Debug, Parser)]
#[command(flatten_help = true)]
pub(crate) enum PinsCommand {
    Read,
    #[command(flatten_help = true)]
    SetMode(GpModes),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum GpioDirection {
    Output,
    Input,
}

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
    pub pin_configs: PinConfigs,
}

#[derive(Debug, Parser)]
#[group(required = true, multiple = true)]
pub(crate) struct PinConfigs {
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

impl PinConfigs {
    pub(crate) fn merge_into_existing(&self, gp_settings: &mut gpio::GpSettings) {
        if let Some(gp0) = self.gp0 {
            gp0.merge_into_existing(&mut gp_settings.gp0);
        }
        if let Some(gp1) = self.gp1 {
            gp1.merge_into_existing(&mut gp_settings.gp1);
        }
        if let Some(gp2) = self.gp2 {
            gp2.merge_into_existing(&mut gp_settings.gp2);
        }
        if let Some(gp3) = self.gp3 {
            gp3.merge_into_existing(&mut gp_settings.gp3);
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

impl From<&Gp0Mode> for gpio::Gp0Designation {
    fn from(value: &Gp0Mode) -> Self {
        match value {
            Gp0Mode::UartReceiveLed => Self::LED_UART_RX,
            Gp0Mode::UsbSuspendState => Self::SSPND,
            Gp0Mode::GpioOutputHigh | Gp0Mode::GpioOutputLow | Gp0Mode::GpioInput => Self::GPIO,
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

impl From<&Gp1Mode> for gpio::Gp1Designation {
    fn from(value: &Gp1Mode) -> Self {
        match value {
            Gp1Mode::ClockOutput => Self::ClockOutput,
            Gp1Mode::AnalogInput => Self::ADC1,
            Gp1Mode::UartTransmitLed => Self::LED_UART_TX,
            Gp1Mode::Interrupt => Self::InterruptDetection,
            Gp1Mode::GpioOutputHigh | Gp1Mode::GpioOutputLow | Gp1Mode::GpioInput => Self::GPIO,
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

impl From<&Gp2Mode> for gpio::Gp2Designation {
    fn from(value: &Gp2Mode) -> Self {
        match value {
            Gp2Mode::UsbDeviceConfigured => Self::USBCFG,
            Gp2Mode::AnalogInput => Self::ADC2,
            Gp2Mode::AnalogOutput => Self::DAC1,
            Gp2Mode::GpioOutputHigh | Gp2Mode::GpioOutputLow | Gp2Mode::GpioInput => Self::GPIO,
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

impl From<&Gp3Mode> for gpio::Gp3Designation {
    fn from(value: &Gp3Mode) -> Self {
        match value {
            Gp3Mode::I2cLed => Self::LED_I2C,
            Gp3Mode::AnalogInput => Self::ADC3,
            Gp3Mode::AnalogOutput => Self::DAC2,
            Gp3Mode::GpioOutputHigh | Gp3Mode::GpioOutputLow | Gp3Mode::GpioInput => Self::GPIO,
        }
    }
}

/// Update a &mut GP settings type from the HAL with the settings from the compact
/// GP pin settings structs used in the CLI.
macro_rules! merge_impl {
    ($cli_type:ty, $hal_type:ty) => {
        impl $cli_type {
            pub(crate) fn merge_into_existing(&self, settings: &mut $hal_type) {
                // Update the GP pin mode (function).
                settings.designation = self.into();
                // If the pin is set to GPIO operation, also update the direction
                // and, if an output, the value.
                match self {
                    <$cli_type>::GpioOutputHigh => {
                        settings.direction = gpio::GpioDirection::Output;
                        settings.value = gpio::LogicLevel::High;
                    }
                    <$cli_type>::GpioOutputLow => {
                        settings.direction = gpio::GpioDirection::Output;
                        settings.value = gpio::LogicLevel::Low;
                    }
                    <$cli_type>::GpioInput => {
                        settings.direction = gpio::GpioDirection::Input;
                    }
                    _ => {}
                };
            }
        }
    };
}

merge_impl!(Gp0Mode, gpio::Gp0Settings);
merge_impl!(Gp1Mode, gpio::Gp1Settings);
merge_impl!(Gp2Mode, gpio::Gp2Settings);
merge_impl!(Gp3Mode, gpio::Gp3Settings);
