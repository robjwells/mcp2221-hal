use bit_field::BitField;

use crate::{GpioDirection, LogicLevel};

#[derive(Debug)]
pub struct FlashData {
    /// Chip settings.
    pub chip_settings: ChipSettings,
    /// General-purpose pins power-up settings.
    pub gp_settings: GPSettings,
    /// Manufacturer string descriptor used during USB enumeration.
    pub usb_manufacturer_descriptor: String,
    /// Product string descriptor used during USB enumeration.
    pub usb_product_descriptor: String,
    /// Serial number used during USB enumeration.
    pub usb_serial_number_descriptor: String,
    /// Factory-set serial number.
    ///
    /// Always "01234567" for the MCP2221A. This cannot be changed.
    pub chip_factory_serial_number: String,
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
/// Chip settings stored in the MCP2221's flash memory.
///
/// Byte and bit addresses in this documentation refer to their position when _reading_
/// from the MCP2221. For their position in the write buffer, subtract two from
/// the byte address.
///
/// **PLEASE NOTE** that for the **DAC** and **ADC** reference voltage source settings,
/// according to the datasheet, reading a 1 means one setting, but writing a 1 means the
/// opposite. This means, for instance, that blindly attempting to round-trip settings
/// read from flash memory would cause a change in the chip's behaviour.
///
/// This seems like it could be a mistake in the datasheet. It is very odd and [I have
/// asked Microchip][mcp-forum] about it. I've not yet been able to test the behaviour
/// myself so, for now, this driver acts in accordance with the datasheet.
///
/// [mcp-forum]: https://forum.microchip.com/s/topic/a5CV40000003RuvMAE/t400836
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
    /// The requested mA value during the USB enumeration. Please consult the USB 2.0
    /// specification on the correct values for power and attributes.
    ///
    /// Note the datasheet says the actual value is the byte value multiplied by 2.
    /// The value in this struct has already been multiplied by 2 for convenience.
    ///
    /// As the halved value is stored as a single byte by the MCP2221A, the maximum
    /// possible value is 510 mA (stored as `255u8` on the chip);
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

    pub(crate) fn apply_to_write_buffer(&self, buf: &mut [u8; 64]) {
        // Note the bytes positions when writing are -2 from the position when reading.
        buf[2].set_bit(7, self.cdc_serial_number_enumeration_enabled);
        buf[2].set_bit(6, self.led_uart_rx_initial_value.into());
        buf[2].set_bit(5, self.led_uart_tx_initial_value.into());
        buf[2].set_bit(4, self.led_i2c_initial_value.into());
        buf[2].set_bit(3, self.sspnd_pin_initial_value.into());
        buf[2].set_bit(2, self.usbcfg_pin_initial_value.into());
        // TODO: support security settings.
        buf[2].set_bits(0..=1, ChipConfigurationSecurity::Unsecured.into());

        // Byte 3 (write) / byte 5 (read)
        buf[3].set_bits(0..=4, self.clock_output_divider);

        // Byte 4 (write) / byte 6 (read) -- DAC settings
        buf[4].set_bits(6..=7, self.dac_reference_voltage.into());
        buf[4].set_bit(5, self.dac_reference_source.into());
        buf[4].set_bits(0..=4, self.dac_power_up_value);

        // Byte 5 (write) / byte 6 (read) -- Interrupts and ADC
        buf[5].set_bit(6, self.interrupt_on_negative_edge);
        buf[5].set_bit(5, self.interrupt_on_positive_edge);
        buf[5].set_bits(3..=4, self.adc_reference_voltage.into());
        buf[5].set_bit(2, self.adc_reference_source.into());

        // Bytes 6 & 7 -- USB Vendor ID (VID)
        let vid_bytes = self.usb_vendor_id.to_le_bytes();
        buf[6] = vid_bytes[0];
        buf[7] = vid_bytes[1];

        // Bytes 8 & 9 -- USB Product ID (PID)
        let pid_bytes = self.usb_product_id.to_le_bytes();
        buf[6] = pid_bytes[0];
        buf[7] = pid_bytes[1];

        // Bytes 10 & 11 -- USB power settings
        buf[10] = self.usb_power_attributes;
        // Note that the stored value is _half_ the actual requested mA.
        // When reading we double the value to be less confusing to users.
        buf[11] = (self.usb_requested_number_of_ma / 2) as u8;

        // TODO: Password support (bytes 12..=19).
    }
}

#[derive(Debug, Clone, Copy)]
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

impl From<ChipConfigurationSecurity> for u8 {
    fn from(value: ChipConfigurationSecurity) -> Self {
        match value {
            ChipConfigurationSecurity::PermanentlyLocked => 0b10,
            ChipConfigurationSecurity::PasswordProtected => 0b01,
            ChipConfigurationSecurity::Unsecured => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

impl From<VrmVoltageReference> for u8 {
    fn from(value: VrmVoltageReference) -> Self {
        match value {
            VrmVoltageReference::V4_096 => 0b11,
            VrmVoltageReference::V2_048 => 0b10,
            VrmVoltageReference::V1_024 => 0b01,
            VrmVoltageReference::Off => 0b00,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DacVoltageReferenceSource {
    VRM,
    VDD,
}

impl From<bool> for DacVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::VRM } else { Self::VDD }
    }
}

impl From<DacVoltageReferenceSource> for bool {
    fn from(value: DacVoltageReferenceSource) -> Self {
        // This is the opposite to what the bit means when reading(!) and when reading &
        // writing SRAM. I've asked about it on the Microchip forum but for now I'll
        // follow the datasheet.
        match value {
            DacVoltageReferenceSource::VRM => false,
            DacVoltageReferenceSource::VDD => true,
        }
    }
}

// Necessary because the DAC and ADC have inverted meanings for
// voltage reference source = 1.
#[derive(Debug, Clone, Copy)]
pub enum AdcVoltageReferenceSource {
    VRM,
    VDD,
}

impl From<bool> for AdcVoltageReferenceSource {
    fn from(value: bool) -> Self {
        if value { Self::VDD } else { Self::VRM }
    }
}

impl From<AdcVoltageReferenceSource> for bool {
    fn from(value: AdcVoltageReferenceSource) -> Self {
        // As with the DAC, this is also the opposite of the meaning of the bit when
        // reading the flash settings.
        match value {
            AdcVoltageReferenceSource::VRM => false,
            AdcVoltageReferenceSource::VDD => true,
        }
    }
}

#[derive(Debug)]
pub struct GPSettings {
    pub gp0: Gp0Settings,
    pub gp1: Gp1Settings,
    pub gp2: Gp2Settings,
    pub gp3: Gp3Settings,
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

    pub(crate) fn apply_to_write_buffer(&self, buf: &mut [u8; 64]) {
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

#[allow(non_camel_case_types)]
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

#[allow(non_camel_case_types)]
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

#[allow(non_camel_case_types)]
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

#[allow(non_camel_case_types)]
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
