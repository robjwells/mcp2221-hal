use bit_field::BitField;

use crate::{GpioDirection, LogicLevel};

#[derive(Debug)]
pub struct FlashData {
    /// Chip settings.
    chip_settings: ChipSettings,
    /// General-purpose pins power-up settings.
    gp_settings: GPSettings,
    /// Manufacturer string descriptor used during USB enumeration.
    usb_manufacturer_descriptor: String,
    /// Product string descriptor used during USB enumeration.
    usb_product_descriptor: String,
    /// Serial number used during USB enumeration.
    usb_serial_number_descriptor: String,
    /// Factory-set serial number.
    ///
    /// Always "01234567" for the MCP2221A. This cannot be changed.
    chip_factory_serial_number: String,
}

impl FlashData {
    pub(crate) fn from_buffers(
        chip_settings: &[u8; 64],
        gp_settings: &[u8; 64],
        usb_mfr: &[u8; 64],
        usb_product: &[u8; 64],
        usb_serial: &[u8; 64],
        chip_factory_serial: &[u8; 64],
    ) -> Self {
        Self {
            chip_settings: ChipSettings::from_buffer(chip_settings),
            gp_settings: GPSettings::from_buffer(gp_settings),
            usb_manufacturer_descriptor: FlashData::buffer_to_string(usb_mfr),
            usb_product_descriptor: FlashData::buffer_to_string(usb_product),
            usb_serial_number_descriptor: FlashData::buffer_to_string(usb_serial),
            chip_factory_serial_number: FlashData::buffer_to_chip_factory_serial(
                chip_factory_serial,
            ),
        }
    }

    fn buffer_to_string(buf: &[u8; 64]) -> String {
        assert_eq!(buf[3], 0x03, "String response sanity check.");

        let n_bytes = buf[2] as usize - 2;
        // Sanity-check the string length.
        assert!(n_bytes <= 60, "String longer than specified.");
        assert_eq!(n_bytes % 2, 0, "Odd number of utf-16 bytes received.");

        // (buf[2] - 2) UTF-16 characters laid out in little-endian order
        // from buf[4] onwards. These strings are at most 30 characters
        // (60 bytes) long. See table 3-7 in the datasheet.
        let n_utf16_chars = n_bytes / 2;
        let mut str_utf16 = Vec::with_capacity(n_utf16_chars);
        for char_number in 0..n_utf16_chars {
            let low_idx = 4 + 2 * char_number;
            let high_idx = 4 + 2 * char_number + 1;
            let utf16 = u16::from_le_bytes([buf[low_idx], buf[high_idx]]);
            str_utf16.push(utf16);
        }

        String::from_utf16(str_utf16.as_slice()).expect("Invalid Unicode string.")
    }

    // Chip factory serial is ASCII chars, and always "01234567".
    fn buffer_to_chip_factory_serial(buf: &[u8; 64]) -> String {
        let length = buf[2] as usize;
        String::from_utf8(buf[4..(4 + length)].to_vec())
            .expect("Chip factory serial not ASCII as expected.")
    }
}

#[derive(Debug)]
pub struct ChipSettings {
    /// Whether a serial number descriptor will be presented during the
    /// USB enumeration of the CDC interface.
    ///
    /// Byte 4 bit 7.
    pub cdc_serial_number_enumeration_enabled: bool,
    /// This value represents the logic level signaled when no UART Rx
    /// activity takes place. When the UART Rx (of the MCP2221A) is
    /// receiving data, the LEDUARTRX pin will take the negated value of
    /// this bit.
    ///
    /// Byte 4 bit 6.
    pub led_uart_rx_initial_value: LogicLevel,
    /// This value represents the logic level signaled when no UART Tx
    /// activity takes place. When the UART Tx (of the MCP2221A) is
    /// sending data, the LEDUARTTX pin will take the negated value of
    /// this bit.
    ///
    /// Byte 4 bit 5.
    pub led_uart_tx_initial_value: LogicLevel,
    /// This value represents the logic level signaled when no I2C traffic
    /// occurs. When the I2C traffic is active, the LEDI2C pin (if enabled)
    /// will take the negated value of this bit.
    ///
    /// Byte 4 bit 4.
    pub led_i2c_initial_value: LogicLevel,
    /// This value represents the logic level signaled when the device is
    /// not in Suspend mode. Upon entering Suspend mode, the SSPND pin (if
    /// enabled) will take the negated value of this bit.
    ///
    /// Byte 4 bit 3.
    pub sspnd_pin_initial_value: LogicLevel,
    /// This value represents the logic level signaled when the device is
    /// not USB configured. When the device will be USB configured, the
    /// USBCFG pin (if enabled) will take the negated value of this bit.
    ///
    /// Byte 4 bit 2.
    pub usbcfg_pin_initial_value: LogicLevel,
    /// Chip configuration security option.
    ///
    /// Byte 4 bits 1 and 0.
    pub chip_configuration_security: ChipConfigurationSecurity,
    /// Clock Output divider value.
    ///
    /// If the GP pin (exposing the clock output) is enabled for clock
    /// output operation, the divider value will be used on the 48 MHz USB
    /// internal clock and its divided output will be sent to this pin.
    ///
    /// Byte 5 bits 4..=0. Value in range 0..=31.
    pub clock_output_divider: u8,
    /// DAC reference voltage (Vrm setting)
    ///
    /// Byte 6 bits 7 & 6.
    pub dac_reference_voltage: VrmVoltageReference,
    /// DAC reference source (Vrm or Vdd)
    ///
    /// Byte 6 bit 5.
    pub dac_reference_source: DacVoltageReferenceSource,
    /// Power-up DAC value.
    ///
    /// Byte 6 bits 4..=0. Value in range 0..=31.
    pub dac_power_up_value: u8,
    /// Interrupt detection for negative edge.
    ///
    /// Byte 7 bit 6.
    pub interrupt_on_negative_edge: bool,
    /// Interrupt detection for positive edge.
    ///
    /// Byte 7 bit 5.
    pub interrupt_on_positive_edge: bool,
    /// ADC reference voltage (Vrm setting)
    ///
    /// Byte 7 bits 4 & 3.
    pub adc_reference_voltage: VrmVoltageReference,
    /// ADC reference source (Vrm or Vdd)
    ///
    /// Note the datasheet "effect" column says this is the DAC reference,
    /// but it appears to be a typo. The DAC and ADC have their own
    /// voltage references (see section 1.8.1.1 of the datasheet).
    ///
    /// Byte 7 bit 2.
    pub adc_reference_source: AdcVoltageReferenceSource,
    /// USB Vendor ID (VID)
    ///
    /// Byte 8 and 9.
    pub usb_vendor_id: u16,
    /// USB Product ID (PID)
    ///
    /// Byte 10 and 11.
    pub usb_product_id: u16,
    /// USB power attributes.
    ///
    /// This value will be used by the MCP2221A's USB Configuration
    /// Descriptor (power attributes value) during the USB enumeration.
    ///
    /// Please consult the USB 2.0 specification on the correct values
    /// for power and attributes.
    ///
    /// Byte 12.
    pub usb_power_attributes: u8,
    /// USB requested number of mA.
    ///
    /// The requested mA value during the USB enumeration.
    ///
    /// Please consult the USB 2.0 specification on the correct values
    /// for power and attributes.
    ///
    /// Note the datasheet said the actual value is the byte value
    /// multiplied by 2. The value in this struct has been multiplied
    /// by 2 for convenience.
    ///
    /// Byte 13.
    pub usb_requested_number_of_ma: u16,
}

impl ChipSettings {
    pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
        use bit_field::BitField;
        Self {
            cdc_serial_number_enumeration_enabled: buf[4].get_bit(7),
            led_uart_rx_initial_value: buf[4].get_bit(6).into(),
            led_uart_tx_initial_value: buf[4].get_bit(5).into(),
            led_i2c_initial_value: buf[4].get_bit(4).into(),
            sspnd_pin_initial_value: buf[4].get_bit(3).into(),
            usbcfg_pin_initial_value: buf[4].get_bit(2).into(),
            chip_configuration_security: buf[4].get_bits(0..=1).into(),
            clock_output_divider: buf[5].get_bits(0..=4),
            dac_reference_voltage: buf[6].get_bits(6..=7).into(),
            dac_reference_source: buf[6].get_bit(5).into(),
            dac_power_up_value: buf[6].get_bits(0..=4),
            interrupt_on_negative_edge: buf[7].get_bit(6),
            interrupt_on_positive_edge: buf[7].get_bit(5),
            adc_reference_voltage: buf[7].get_bits(3..=4).into(),
            adc_reference_source: buf[7].get_bit(2).into(),
            usb_vendor_id: u16::from_le_bytes([buf[8], buf[9]]),
            usb_product_id: u16::from_le_bytes([buf[10], buf[11]]),
            usb_power_attributes: buf[12],
            usb_requested_number_of_ma: buf[13] as u16 * 2,
        }
    }
}

#[derive(Debug)]
pub enum ChipConfigurationSecurity {
    PermanentlyLocked,
    PasswordProtected,
    Unsecured,
}

impl From<u8> for ChipConfigurationSecurity {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Unsecured,
            0b01 => Self::PasswordProtected,
            0b10 | 0b11 => Self::PermanentlyLocked,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
/// Setting of the internal voltage reference (VRM)
pub enum VrmVoltageReference {
    /// 4.096V
    ///
    /// Only available if VDD is above this voltage.
    V4_096,
    /// 2.048V
    V2_048,
    /// 1.024V
    V1_024,
    Off,
}

impl From<u8> for VrmVoltageReference {
    fn from(value: u8) -> Self {
        assert!(value <= 0b11, "Incorrect use of the from constructor.");
        match value {
            0b00 => Self::Off,
            0b01 => Self::V1_024,
            0b10 => Self::V2_048,
            0b11 => Self::V4_096,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum DacVoltageReferenceSource {
    VRM,
    VDD,
}

impl From<bool> for DacVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::VRM } else { Self::VDD }
    }
}

// Necessary because the DAC and ADC have inverted meanings for
// voltage reference source = 1.
#[derive(Debug)]
pub enum AdcVoltageReferenceSource {
    VRM,
    VDD,
}

impl From<bool> for AdcVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::VDD } else { Self::VRM }
    }
}

#[derive(Debug)]
pub struct GPSettings {
    gp0: Gp0Settings,
    gp1: Gp1Settings,
    gp2: Gp2Settings,
    gp3: Gp3Settings,
}

impl GPSettings {
    fn from_buffer(buf: &[u8; 64]) -> Self {
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

#[allow(non_camel_case_types)]
#[derive(Debug)]
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

#[allow(non_camel_case_types)]
#[derive(Debug)]
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

#[allow(non_camel_case_types)]
#[derive(Debug)]
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

#[allow(non_camel_case_types)]
#[derive(Debug)]
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

#[derive(Debug)]
struct Gp0Settings {
    /// GP0 power-up output value.
    ///
    /// When GP0 is set as an output GPIO, this value will be present at
    /// the GP0 pin at power-up/reset.
    ///
    /// Byte 4 bit 4.
    power_up_value: LogicLevel,
    /// GP0 power-up direction.
    ///
    /// Works only when GP0 is set for GPIO operation.
    ///
    /// Byte 4 bit 3.
    power_up_direction: GpioDirection,
    /// GP0 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 4 bits 0..=2.
    power_up_designation: Gp0Designation,
}

#[derive(Debug)]
struct Gp1Settings {
    /// GP1 power-up output value.
    ///
    /// When GP1 is set as an output GPIO, this value will be present at
    /// the GP1 pin at power-up/reset.
    ///
    /// Byte 5 bit 4.
    power_up_value: LogicLevel,
    /// GP1 power-up direction.
    ///
    /// Works only when GP1 is set for GPIO operation.
    ///
    /// Byte 5 bit 3.
    power_up_direction: GpioDirection,
    /// GP1 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 5 bits 0..=2.
    power_up_designation: Gp1Designation,
}

#[derive(Debug)]
struct Gp2Settings {
    /// GP2 power-up output value.
    ///
    /// When GP2 is set as an output GPIO, this value will be present at
    /// the GP2 pin at power-up/reset.
    ///
    /// Byte 6 bit 4.
    power_up_value: LogicLevel,
    /// GP2 power-up direction.
    ///
    /// Works only when GP2 is set for GPIO operation.
    ///
    /// Byte 6 bit 3.
    power_up_direction: GpioDirection,
    /// GP2 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 6 bits 0..=2.
    power_up_designation: Gp2Designation,
}

#[derive(Debug)]
struct Gp3Settings {
    /// GP3 power-up output value.
    ///
    /// When GP3 is set as an output GPIO, this value will be present at
    /// the GP3 pin at power-up/reset.
    ///
    /// Byte 7 bit 4.
    power_up_value: LogicLevel,
    /// GP3 power-up direction.
    ///
    /// Works only when GP3 is set for GPIO operation.
    ///
    /// Byte 7 bit 3.
    power_up_direction: GpioDirection,
    /// GP3 designation.
    ///
    /// Setting of the pin's function.
    ///
    /// Byte 7 bits 0..=2.
    power_up_designation: Gp3Designation,
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
