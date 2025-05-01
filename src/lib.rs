#![allow(dead_code)]
#![allow(unused_variables)]

use error::Error;
use flash_data::FlashData;
use status::Status;

const MICROCHIP_VENDOR_ID: u16 = 1240;
const MCP2221A_PRODUCT_ID: u16 = 221;

mod error {
    #[derive(Debug)]
    pub enum Error {
        NoDeviceFound,
        CommandFailed(u8),
        MismatchedCommandCodeEcho { sent: u8, received: u8 },
        HidApi(hidapi::HidError),
    }

    impl From<hidapi::HidError> for Error {
        fn from(value: hidapi::HidError) -> Self {
            Self::HidApi(value)
        }
    }
}

pub struct MCP2221 {
    inner: hidapi::HidDevice,
    write_buffer: [u8; 65],
    read_buffer: [u8; 64],
}

/// USB device functionality
impl MCP2221 {
    pub fn open() -> Result<Self, Error> {
        MCP2221::open_with_vid_pid(MICROCHIP_VENDOR_ID, MCP2221A_PRODUCT_ID)
    }

    pub fn open_with_vid_pid(vendor_id: u16, product_id: u16) -> Result<Self, Error> {
        let hidapi = hidapi::HidApi::new()?;
        let device = hidapi.open(vendor_id, product_id)?;
        Ok(Self {
            inner: device,
            write_buffer: [0u8; 65],
            read_buffer: [0u8; 64],
        })
    }

    pub fn usb_device_info(&self) -> Result<hidapi::DeviceInfo, Error> {
        let info = self.inner.get_device_info()?;
        Ok(info)
    }
}

/// HID Commands
impl MCP2221 {
    pub fn status(&mut self) -> Result<Status, Error> {
        self.set_command(Command::StatusSetParameters);
        let _ = self._try_transfer()?;
        Ok(Status::from_buffer(&self.read_buffer))
    }

    pub fn read_flash_data(&mut self) -> Result<FlashData, Error> {
        self.set_command(Command::ReadFlashData(ReadFlashDataSubCode::ChipSettings));
        let _ = self._try_transfer()?;
        let chip_settings = self.read_buffer;

        self.set_command(Command::ReadFlashData(ReadFlashDataSubCode::GPSettings));
        let _ = self._try_transfer()?;
        let gp_settings = self.read_buffer;

        self.set_command(Command::ReadFlashData(
            ReadFlashDataSubCode::UsbManufacturerDescriptor,
        ));
        let _ = self._try_transfer()?;
        let usb_mfr = self.read_buffer;

        self.set_command(Command::ReadFlashData(
            ReadFlashDataSubCode::UsbProductDescriptor,
        ));
        let _ = self._try_transfer()?;
        let usb_product = self.read_buffer;

        self.set_command(Command::ReadFlashData(
            ReadFlashDataSubCode::UsbSerialNumberDescriptor,
        ));
        let _ = self._try_transfer()?;
        let usb_serial = self.read_buffer;

        self.set_command(Command::ReadFlashData(
            ReadFlashDataSubCode::ChipFactorySerialNumber,
        ));
        let _ = self._try_transfer()?;
        let chip_factory_serial = self.read_buffer;

        Ok(FlashData::from_buffers(
            &chip_settings,
            &gp_settings,
            &usb_mfr,
            &usb_product,
            &usb_serial,
            &chip_factory_serial,
        ))
    }

    /// Write the appropriate command byte to write_buffer[1].
    ///
    /// write_buffer starts with the dummy/default report number, so the
    /// actual MCP command is at write_buffer[1..=65].
    fn set_command(&mut self, c: Command) {
        use Command::*;
        use ReadFlashDataSubCode::*;
        let (command_byte, sub_command_byte): (u8, Option<u8>) = match c {
            StatusSetParameters => (0x10, None),
            ReadFlashData(ChipSettings) => (0xB0, Some(0x00)),
            ReadFlashData(GPSettings) => (0xB0, Some(0x01)),
            ReadFlashData(UsbManufacturerDescriptor) => (0xB0, Some(0x02)),
            ReadFlashData(UsbProductDescriptor) => (0xB0, Some(0x03)),
            ReadFlashData(UsbSerialNumberDescriptor) => (0xB0, Some(0x04)),
            ReadFlashData(ChipFactorySerialNumber) => (0xB0, Some(0x05)),
        };
        self.write_buffer[1] = command_byte;
        if let Some(sub_command_byte) = sub_command_byte {
            self.write_buffer[2] = sub_command_byte;
        }
    }

    /// Write the current output buffer state to the MCP and read from it.
    fn _try_transfer(&mut self) -> Result<(usize, usize), Error> {
        let written = self.inner.write(&self.write_buffer)?;
        let read = self.inner.read(&mut self.read_buffer)?;
        // Check command-code echo.
        if self.read_buffer[0] != self.write_buffer[1] {
            return Err(Error::MismatchedCommandCodeEcho {
                sent: self.write_buffer[1],
                received: self.read_buffer[0],
            });
        }
        // Zero write buffer to prevent pollution from previous commands.
        self.write_buffer = [0; 65];
        // Check success code.
        match self.read_buffer[1] {
            0x00 => Ok((written, read)),
            code => Err(Error::CommandFailed(code)),
        }
    }
}

enum Command {
    StatusSetParameters,
    ReadFlashData(ReadFlashDataSubCode),
}

enum ReadFlashDataSubCode {
    ChipSettings,
    GPSettings,
    UsbManufacturerDescriptor,
    UsbProductDescriptor,
    UsbSerialNumberDescriptor,
    ChipFactorySerialNumber,
}

#[derive(Debug)]
pub enum LogicLevel {
    High,
    Low,
}

impl From<bool> for LogicLevel {
    fn from(value: bool) -> Self {
        if value { Self::High } else { Self::Low }
    }
}

#[derive(Debug)]
pub enum GpioDirection {
    Input,
    Output,
}

impl From<bool> for GpioDirection {
    fn from(value: bool) -> Self {
        if value { Self::Input } else { Self::Output }
    }
}

pub mod status {
    /// Current status of the device.
    ///
    /// Bytes in documentation are numbered from 0 through 63 and correspond
    /// to table 3-1 in section 3.1.1 (STATUS/SET PARAMETERS) of the datasheet.
    #[derive(Debug)]
    pub struct Status {
        /// The requested I2C transfer length.
        ///
        /// Bytes 9 & 10.
        pub i2c_transfer_requested_length: u16,
        /// The already transferred (through I2C) number of bytes.
        ///
        /// Bytes 11 & 12.
        pub i2c_transfer_completed_length: u16,
        /// Byte 13.
        pub i2c_internal_data_buffer_counter: u8,
        /// Byte 14.
        pub i2c_communication_speed_divider: u8,
        /// Byte 15.
        pub i2c_timeout_value: u8,
        /// Bytes 16 & 17.
        pub i2c_address_being_used: u16,
        /// Byte 22.
        pub i2c_scl_line_high: bool,
        /// Byte 23.
        pub i2c_sda_line_high: bool,
        /// Byte 24.
        pub interrupt_edge_detector_state: u8,
        /// I2C Read pending value.
        ///
        /// Byte 25. This field is used by the USB host to know if the MCP2221A
        /// still has to read from a slave device. Value 0, 1 or 2.
        pub i2c_read_pending_value: u8,
        /// MCP2221A hardware revision (major, minor).
        ///
        /// Bytes 46 & 47.
        pub hardware_revision: (char, char),
        /// MCP2221A firmware revision (major, minor)
        ///
        /// Bytes 48 & 49.
        pub firmware_revision: (char, char),
        /// ADC Data (16-bit) values.
        ///
        /// 3x 16-bit ADC channel values (CH0, CH1, CH2).
        ///
        /// Bytes 50..=55.
        pub adc_values: (u16, u16, u16),
    }

    impl Status {
        pub(crate) fn from_buffer(buf: &[u8; 64]) -> Self {
            Self {
                i2c_transfer_requested_length: u16::from_le_bytes([buf[9], buf[10]]),
                i2c_transfer_completed_length: u16::from_le_bytes([buf[11], buf[12]]),
                i2c_internal_data_buffer_counter: buf[13],
                i2c_communication_speed_divider: buf[14],
                i2c_timeout_value: buf[15],
                i2c_address_being_used: u16::from_le_bytes([buf[16], buf[17]]),
                i2c_scl_line_high: buf[22] == 0x01,
                i2c_sda_line_high: buf[23] == 0x01,
                interrupt_edge_detector_state: buf[24],
                i2c_read_pending_value: buf[25],
                hardware_revision: (buf[46] as char, buf[47] as char),
                firmware_revision: (buf[48] as char, buf[49] as char),
                adc_values: (
                    u16::from_le_bytes([buf[50], buf[51]]),
                    u16::from_le_bytes([buf[52], buf[53]]),
                    u16::from_le_bytes([buf[54], buf[55]]),
                ),
            }
        }
    }
}

pub mod flash_data {
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
            eprintln!("Serial :: {:?}", &buf[4..(4 + length)]);
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
}
