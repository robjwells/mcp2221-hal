//! Analog input (ADC) and output (DAC) types.
//!
//! ## Analog input (ADC)
//!
//! The MCP2221 has a 10-bit ADC with three channels exposed on pins GP1, GP2 and GP3,
//! which can each be separately (or not configured) as analog inputs, and their
//! readings are separate, though they do share a single voltage reference.
//!
//! Analog readings can be taken by calling [`MCP2221::analog_read`], which returns an
//! [`AdcReading`] containing potentially three analog readings. The pin fields of that
//! struct will be `Some(u16)` if the pin is configured as an analog input, and `None`
//! otherwise. Since it is a 10-bit ADC, the readings will be in the range `0..=1023`.
//! The voltage reference at the time of reading is included in that struct so that
//! you may convert the raw reading into a voltage with the formula:
//!
//! ```plain
//! voltage_read = adc_reading / 1023 * voltage_reference
//! ```
//!
//! [`MCP2221::analog_read`]: crate::MCP2221::analog_read
//!
//! While pins that are not configured as analog inputs do not have values in the
//! [`AdcReading`] struct, the readings of all three ADC channels can be seen in
//! the [`Status`] output whether or not the pins are so configured. The readings
//! appear to be what you would expect, but formally this behaviour is undefined and
//! unsupported.
//!
//! [`Status`]: crate::status::Status
//!
//! ## Analog output (DAC)
//!
//! The MCP2221 has a 5-bit DAC, with a single output channel shared between pins GP2
//! and GP3. These pins can be configured separately as analog outputs, but if both
//! are analog outputs they will have the same output level.
//!
//! Analog writes can be performed by calling [`MCP2221::analog_write`], with a 5-bit
//! value (`0..=31`). This will set the output voltage on any pins configured as
//! analog outputs, with the voltage range determined by the DAC’s voltage reference.
//!
//! [`MCP2221::analog_write`]: crate::MCP2221::analog_write
//!
//! ## Voltage references
//!
//! The ADC and DAC each have a voltage reference module, referred to in the datasheet
//! as Vrm. By setting the voltage reference, you set the range for analog input readings
//! and analog output voltage levels.
//!
//! For example, if you set the ADC to a voltage reference of 2.048V (achieved by
//! configuring it with [`VoltageReference`]`(`[`VrmVoltage::V2_048`]`)`), a reading of
//! 1023 indicates that the voltage on the pin is at or above 2V, and a reading of 512
//! that the voltage is about 1V.
//!
//! If we name the supply voltage Vdd, the possible voltage references are 1.024V,
//! 2.048V, 4.096V (if Vdd is greater than this), and Vdd itself.
//!
//! <div class="warning">
//!
//! ### Vrm bugs
//!
//! There appear to be two bugs with the Vrm voltage reference source.
//!
//! #### Vrm set to "off" causes 0V DAC output
//!
//! The design of the MCP2221 configuration commands (and their implementation in this
//! driver) allows the user to configure the DAC to reference Vrm, but to set
//! Vrm to "[off]". This results in a voltage output barely above 0V at all output
//! values. The datasheet says this configuration should result in a reference level
//! equivalent to Vdd (which _is_ how it works for the ADC).
//!
//! [off]: VrmVoltage::Off
//!
//! #### Vrm power-up setting always treated as "off"
//!
//! The MCP2221 [`ChipSettings`] stored in flash memory determine the behaviour of the
//! device on power-up, as they are copied into SRAM and are the initial settings.
//!
//! If you set the DAC to use Vrm as its voltage reference, its output on power-up
//! will be barely above 0V (identical to the above), no matter the power-up DAC
//! output value. This is the case for all Vrm voltage levels, not just "off".
//!
//! Likewise, the ADC will behave as if configured with a Vrm level of "off", and
//! will give readings with Vdd as the reference voltage, no matter the level set
//! in flash memory.
//!
//! Re-configuring the voltage references for the DAC and ADC in SRAM will cause them
//! to behave as expected from that point on.
//!
//! [`ChipSettings`]: crate::settings::ChipSettings
//!
//! </div>
//!
//! ## DAC output levels
//!
//! In testing, the MCP2221 DAC appears to have a non-linear output, and which does not
//! get close to the reference voltage. The testing was performed by wiring GP3 on an
//! [Adafruit MCP2221A breakout board][adafruit], to an ADC pin of a Raspberry Pi Pico
//! (which has a 10-bit ADC). The supply voltage (Vdd) was 3.3V, and the measured
//! resistance between the pins was 95Ω. [See here for a chart of the measured DAC
//! output][chart].
//!
//! Please note that these readings are rough (taken by eye from the Pico's output, with
//! a few checked with a multimeter), and that I am a relative novice at electronics. If
//! you can think of a reason why I may have been "holding it wrong", please do [open an
//! issue].
//!
//! [adafruit]: https://www.adafruit.com/product/4471
//! [open an issue]: https://github.com/robjwells/mcp2221-hal/issues
//! [chart]: https://github.com/robjwells/mcp2221-hal/blob/main/assets/dac-output-chart.png
//!
//!
//! <details>
//!     <summary><h4>Table of DAC output voltage readings</h4></summary>
//!
//! | Value | Vdd @ 3.3V | Vrm @ 2V | Vrm @ 1V |
//! | ----: | ---------: | -------: | -------: |
//! |     0 |       0.03 |     0.03 |     0.03 |
//! |     1 |       0.11 |     0.08 |     0.06 |
//! |     2 |       0.16 |     0.11 |     0.08 |
//! |     3 |       0.20 |     0.14 |     0.09 |
//! |     4 |       0.24 |     0.16 |     0.10 |
//! |     5 |       0.27 |     0.18 |     0.11 |
//! |     6 |       0.30 |     0.20 |     0.12 |
//! |     7 |       0.32 |     0.22 |     0.13 |
//! |     8 |       0.35 |     0.23 |     0.14 |
//! |     9 |       0.38 |     0.25 |     0.15 |
//! |    10 |       0.40 |     0.26 |     0.15 |
//! |    11 |       0.42 |     0.28 |     0.16 |
//! |    12 |       0.44 |     0.29 |     0.17 |
//! |    13 |       0.47 |     0.31 |     0.18 |
//! |    14 |       0.50 |     0.33 |     0.19 |
//! |    15 |       0.54 |     0.35 |     0.19 |
//! |    16 |       0.57 |     0.37 |     0.20 |
//! |    17 |       0.60 |     0.39 |     0.21 |
//! |    18 |       0.65 |     0.41 |     0.23 |
//! |    19 |       0.69 |     0.43 |     0.24 |
//! |    20 |       0.75 |     0.46 |     0.25 |
//! |    21 |       0.80 |     0.49 |     0.27 |
//! |    22 |       0.87 |     0.53 |     0.28 |
//! |    23 |       0.94 |     0.57 |     0.30 |
//! |    24 |       1.03 |     0.62 |     0.33 |
//! |    25 |       1.14 |     0.68 |     0.35 |
//! |    26 |       1.27 |     0.75 |     0.39 |
//! |    27 |       1.42 |     0.84 |     0.42 |
//! |    28 |       1.60 |     0.94 |     0.46 |
//! |    29 |       1.83 |     1.07 |     0.52 |
//! |    30 |       2.12 |     1.23 |     0.60 |
//! |    31 |       2.47 |     1.43 |     0.70 |
//!
//! </details>
//!
//! ## Datasheet
//!
//! See section 1.8 of the [MCP2221 datasheet] for information about the ADC, DAC, and
//! Vrm. The bit patterns representing voltage reference source and the Vrm voltage
//! level can be found in register 1-3 and 1-4 (ChipSetting2 and ChipSetting3).
//!
//! Note that in the datasheet HID command sections related to setting the voltage
//! reference source, [there appear to be a couple of mistakes][mistakes] in whether Vrm
//! or Vdd is selected by setting the appropriate bit to 1. This library always
//! interprets 1 as meaning that Vrm is selected, and 0 that Vdd is selected, and the
//! measured output shows this to be correct.
//!
//! [MCP2221 datasheet]: https://ww1.microchip.com/downloads/aemDocuments/documents/APID/ProductDocuments/DataSheets/MCP2221A-Data-Sheet-20005565E.pdf
//! [mistakes]: https://forum.microchip.com/s/topic/a5CV40000003RuvMAE/t400836

/// Three-channel reading of the 10-bit ADC.
///
/// Each channel reading is optional as their values are not defined if the
/// corresponding pin is not configured for ADC operation. Readings are in the range
/// `0..=1023`, with 1023 being the level of the voltage reference.
///
/// The channels are named here to match their GP pin (1-3) as given in table 1-1
/// of the datasheet (where channel 1 is read from GP1). Note that in table 3-2
/// the channels are named 0-2.
#[derive(Debug, Clone, Copy)]
pub struct AdcReading {
    /// ADC voltage reference setting in SRAM at the time of the reading.
    pub vref: VoltageReference,
    /// Analog reading from GP1.
    pub gp1: Option<u16>,
    /// Analog reading from GP2.
    pub gp2: Option<u16>,
    /// Analog reading from GP3.
    pub gp3: Option<u16>,
}

/// Voltage level for the internal Vrm voltage reference module.
///
/// ## Datasheet
///
/// See section 1.8.1.1 for details about the Vrm voltage levels (with the caveat
/// listed for [`VrmVoltage::Off`]).
#[derive(Debug, Clone, Copy)]
pub enum VrmVoltage {
    /// 4.096V
    ///
    /// Only available if Vdd is above this voltage.
    V4_096,
    /// 2.048V
    V2_048,
    /// 1.024V
    V1_024,
    /// Reference voltage is off and the supply voltage (Vdd) is used.
    ///
    /// <div class="warning">
    ///
    /// Configuring the DAC to use this voltage reference (Vrm in “off” mode) results
    /// in an output voltage barely above 0V at all output values.
    ///
    /// </div>
    Off,
}

/// Convert a raw 2-bit value into the corresponding Vrm voltage level.
///
/// ## Datasheet
///
/// See the `DACVRM` and `ADCVRM` descriptions in the ChipSetting2 and ChipSetting3
/// registers (register 1-3 and 1-4), as well as the appropriate parts of the settings
/// commands (eg, table 3-5, bytes 6 and 7).
#[doc(hidden)]
impl From<u8> for VrmVoltage {
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

/// Convert a Vrm voltage level into its corresponding 2-bit representation.
///
/// ## Datasheet
///
/// See the `DACVRM` and `ADCVRM` descriptions in the ChipSetting2 and ChipSetting3
/// registers (register 1-3 and 1-4), as well as the appropriate parts of the settings
/// commands (eg, table 3-5, bytes 6 and 7).
#[doc(hidden)]
impl From<VrmVoltage> for u8 {
    fn from(value: VrmVoltage) -> Self {
        match value {
            VrmVoltage::V4_096 => 0b11,
            VrmVoltage::V2_048 => 0b10,
            VrmVoltage::V1_024 => 0b01,
            VrmVoltage::Off => 0b00,
        }
    }
}

/// Analog voltage reference source.
///
/// Used to set the reference for analog input readings (ADC) and analog output (DAC).
/// The ADC and DAC have separate reference modules, with identical options.
///
/// <div class="warning">
///
/// Setting the voltage reference in flash for either the DAC or ADC to Vrm (at any
/// level) will cause them to behave on power-up as if they had been set to Vrm with
/// a level of "off". See the [`analog`] module-level documentation for more
/// information.
///
/// </div>
///
/// [`analog`]: crate::analog
///
/// ## Datasheet
///
/// See section 1.8 for information about the Vrm modules, and general information
/// about the ADC and DAC.
#[derive(Debug, Clone, Copy)]
pub enum VoltageReference {
    /// Internal voltage reference module with a configurable voltage level.
    Vrm(VrmVoltage),
    /// MCP2221 supply voltage.
    Vdd,
}

/// Construct a voltage reference from a 1-bit source and 2-bit Vrm level.
///
/// See [`VrmVoltage`] for the meaning of the 2-bit level.
///
/// The source is represented as a `bool` because this is the type returned by the
/// [`bit_field`] crate when reading a single bit.
///
/// ## Datasheet
///
/// See `DACREF` and `ADCREF` in registers ChipSetting2 and ChipSetting3 for the meaning
/// of 1 = Vrm and 0 = Vdd (registers 1-3 and 1-4). Note that the _descriptions_ of the
/// source selector bit in a couple of the HID commands have mistakes that reverse this.
/// See this [forum post] for more details about those mistakes.
///
/// [forum post]: https://forum.microchip.com/s/topic/a5CV40000003RuvMAE/t400836
#[doc(hidden)]
impl From<(bool, u8)> for VoltageReference {
    fn from((source_bit, vrm_level): (bool, u8)) -> Self {
        // 1 = Vrm, 0 = Vdd, despite the inconsistency of the datasheet.
        match source_bit {
            true => Self::Vrm(VrmVoltage::from(vrm_level)),
            false => Self::Vdd,
        }
    }
}

/// Convert into a 1-bit voltage source selector and 2-bit Vrm level selector.
///
/// See [`VrmVoltage`] for the meaning of the 2-bit level.
///
/// The source is represented as a `bool` because this is the type used by the
/// [`bit_field`] crate to set a individual bit.
///
/// ## Datasheet
///
/// See `DACREF` and `ADCREF` in registers ChipSetting2 and ChipSetting3 for the meaning
/// of 1 = Vrm and 0 = Vdd (registers 1-3 and 1-4). Note that the _descriptions_ of the
/// source selector bit in a couple of the HID commands have mistakes that reverse this.
/// See this [forum post] for more details about those mistakes.
///
/// [forum post]: https://forum.microchip.com/s/topic/a5CV40000003RuvMAE/t400836
#[doc(hidden)]
impl From<VoltageReference> for (bool, u8) {
    fn from(value: VoltageReference) -> Self {
        match value {
            VoltageReference::Vrm(vrm_level) => (true, vrm_level.into()),
            VoltageReference::Vdd => (false, VrmVoltage::Off.into()),
        }
    }
}
