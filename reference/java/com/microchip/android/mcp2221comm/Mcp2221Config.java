/*
 * Copyright (C) 2014 Microchip Technology Inc. and its subsidiaries. You may use this software and
 * any derivatives exclusively with Microchip products.
 * 
 * THIS SOFTWARE IS SUPPLIED BY MICROCHIP "AS IS". NO WARRANTIES, WHETHER EXPRESS, IMPLIED OR
 * STATUTORY, APPLY TO THIS SOFTWARE, INCLUDING ANY IMPLIED WARRANTIES OF NON-INFRINGEMENT,
 * MERCHANTABILITY, AND FITNESS FOR A PARTICULAR PURPOSE, OR ITS INTERACTION WITH MICROCHIP
 * PRODUCTS, COMBINATION WITH ANY OTHER PRODUCTS, OR USE IN ANY APPLICATION.
 * 
 * IN NO EVENT WILL MICROCHIP BE LIABLE FOR ANY INDIRECT, SPECIAL, PUNITIVE, INCIDENTAL OR
 * CONSEQUENTIAL LOSS, DAMAGE, COST OR EXPENSE OF ANY KIND WHATSOEVER RELATED TO THE SOFTWARE,
 * HOWEVER CAUSED, EVEN IF MICROCHIP HAS BEEN ADVISED OF THE POSSIBILITY OR THE DAMAGES ARE
 * FORESEEABLE. TO THE FULLEST EXTENT ALLOWED BY LAW, MICROCHIP'S TOTAL LIABILITY ON ALL CLAIMS IN
 * ANY WAY RELATED TO THIS SOFTWARE WILL NOT EXCEED THE AMOUNT OF FEES, IF ANY, THAT YOU HAVE PAID
 * DIRECTLY TO MICROCHIP FOR THIS SOFTWARE.
 * 
 * MICROCHIP PROVIDES THIS SOFTWARE CONDITIONALLY UPON YOUR ACCEPTANCE OF THESE TERMS.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
 * in compliance with the License. You may obtain a copy of the License at
 * 
 * http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software distributed under the License
 * is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
 * or implied. See the License for the specific language governing permissions and limitations under
 * the License.
 */

package com.microchip.android.mcp2221comm;

/**
 * Contains the configuration settings for a MCP2221 object. Used in the getSRAMSettings and
 * setSramSettings methods.
 */
public final class Mcp2221Config {

    // ******************************
    // Constants
    // *******************************

    /** VDD voltage reference value for ADC and DAC Vref. */
    public static final byte VREF_VDD = 0;
    /** 1.024V voltage reference value for ADC and DAC Vref. */
    public static final byte VREF_1024 = 3;
    /** 2.048V voltage reference value for ADC and DAC Vref. */
    public static final byte VREF_2048 = 5;
    /** 4.096V voltage reference value for ADC and DAC Vref. */
    public static final byte VREF_4096 = 7;

    // ================================================================================
    // Properties
    // ================================================================================

    /**
     * Number of GPIO pins available on the MCP2221.
     */
    private static final int GP_PINS = 4;
    /**
     * Password protection status. 0 = unsecured, 1 = password protected, 2 = permanently locked
     */
    private byte mAccessControlState;
    /**
     * Vendor ID.
     */
    private int mVid;
    /**
     * Product ID.
     */
    private int mPid;
    /**
     * Requested USB current. The requested mA value during the USB enumeration will represent the
     * value at this index multiplied by 2
     */
    private int currentRequested;

    /**
     * This value will be used by the MCP2221’s USB Configuration Descriptor (power attributes
     * value) during the USB enumeration.
     */
    private byte powerAttribute;

    /**
     * Initial value for USBCFG pin option.This value represents the logic level signaled when the
     * device is not USB configured. When the device will be USB configured, the USBCFG pin (if
     * enabled) will take the negated value of this bit.
     */
    private byte usbConfigPinValue;

    /**
     * Initial value for SSPND pin option.This value represents the logic level signaled when the
     * device is not in Suspend mode. Upon entering Suspend mode, the SSPND pin (if enabled) will
     * take the negated value of this bit.
     */
    private byte suspendPinValue;

    /**
     * CDC Serial Number Enumeration Enable. 0 = No serial number descriptor will be presented
     * during the USB enumeration. 1 = The USB serial number will be used during the USB enumeration
     * of the CDC interface
     */
    private byte cdcSerialNumberEnumeration;
    /**
     * Initial value for LEDUARTRX pin option. This value represents the logic level signaled when
     * no UART RX activity takes places. When the UART RX (of the MCP2221) is receiving data, the
     * LEDUARTRX pin will take the negated value of this bit.
     */
    private byte ledUartRxInitialValue;
    /**
     * Initial value for LEDUARTRX pin option. This value represents the logic level signaled when
     * no UART RX activity takes places. When the UART RX (of the MCP2221) is receiving data, the
     * LEDUARTRX pin will take the negated value of this bit.
     */
    private byte ledUartTxInitialValue;
    /**
     * Initial value for LEDI2C pin option. This value represents the logic level signaled when no
     * I2C traffic occurs. When I2C traffic is active, the LEDI2C pin (if enabled) will take the
     * negated value of this bit.
     */
    private byte ledI2cInitialValue;
    /**
     * Pin designations for the gpio pins.
     */
    private byte[] gpPinDesignations = new byte[GP_PINS];
    /**
     * GPIO pin directions. 0 = output, 1 = input.
     */
    private byte[] gpPinDirections = new byte[GP_PINS];
    /**
     * GPIO pin values.
     */
    private byte[] gpPinValues = new byte[GP_PINS];
    /**
     * Configuration for the interrupt on change pin. 1 = rising edges, 2 = falling edges, 3 = both
     */
    private byte interruptPinMode;
    /**
     * Duty cycle value.
     */
    private byte gpClockDutyCycle;
    /**
     * This divider will determine the frequency set for the clock output on the GP pin.
     */
    private byte gpClockDividerValue;
    /**
     * DAC voltage reference. b'11' - Reference voltage is 4.096V (only if VDD is above this
     * voltage) b'10' Reference voltage is 2.048V b'01' Reference voltage is 1.024V b'00' Reference
     * voltage is off (this is useful for the case in which the DAC uses other reference than VRM
     * DAC; e.g., VDD)
     */
    private byte dacVoltageReference;
    /**
     * Power-Up DAC value.
     */
    private byte dacValue;
    /**
     * ADC Reference Voltage. b'11' Reference voltage is 4.096V (only if VDD is above this voltage)
     * b'10' Reference voltage is 2.048V b'01' Reference voltage is 1.024V b'00' Reference voltage
     * is off (this is useful for the case in which the ADC uses other reference than VRM DAC; e.g.,
     * VDD)
     */
    private byte adcVoltageReference;

    // ================================================================================
    // Constructors
    // ================================================================================

    /**
     * Create a new MCP2221Config object.
     */
    public Mcp2221Config() {

    }

    // ================================================================================
    // Accessors
    // ================================================================================

    /**
     * Gets the password protection status. <br>
     * 0 = unsecured, 1 = password protected, 2 = permanently locked
     * 
     * @return the accessControlState
     */
    public byte getAccessControlState() {
        return mAccessControlState;
    }

    /**
     * Sets the password protection status. Only used when reading the settings via the
     * GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param accessControlState
     *            the accessControlState to set
     */
    public void setAccessControlState(byte accessControlState) {
        this.mAccessControlState = accessControlState;
    }

    /**
     * Get the Vendor ID.
     * 
     * @return the vid
     */
    public int getVid() {
        return mVid;
    }

    /**
     * Set the Vendor ID. Only used when reading the settings via the GetSramSettings. Altering this
     * parameter does not affect the MCP2221's settings.
     * 
     * @param vid
     *            the vid to set
     */
    public void setVid(int vid) {
        this.mVid = vid;
    }

    /**
     * Get the Product ID.
     * 
     * @return the pid
     */
    public int getPid() {
        return mPid;
    }

    /**
     * Set the Product ID. Only used when reading the settings via the GetSramSettings. Altering
     * this parameter does not affect the MCP2221's settings.
     * 
     * @param pid
     *            the pid to set
     */
    public void setPid(int pid) {
        this.mPid = pid;
    }

    /**
     * Gets the requested USB current. The requested mA value during the USB enumeration will
     * represent the value at this index multiplied by 2
     * 
     * @return the currentRequested
     */
    public int getCurrentRequested() {
        return currentRequested;
    }

    /**
     * Sets the requested USB current. Only used when reading the settings via the GetSramSettings.
     * Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mCurrentRequested
     *            the currentRequested to set
     */
    public void setCurrentRequested(int mCurrentRequested) {
        this.currentRequested = mCurrentRequested;
    }

    /**
     * Get the power attribute value that will be used by the MCP2221’s USB Configuration Descriptor
     * during the USB enumeration.
     * 
     * @return the powerAttribute
     */
    public byte getPowerAttribute() {
        return powerAttribute;
    }

    /**
     * Set the power attribute value. Only used when reading the settings via the GetSramSettings.
     * Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mPowerAttribute
     *            the powerAttribute to set
     */
    public void setPowerAttribute(byte mPowerAttribute) {
        this.powerAttribute = mPowerAttribute;
    }

    /**
     * Gets the initial value for USBCFG pin option.This value represents the logic level signaled
     * when the device is not USB configured. When the device will be USB configured, the USBCFG pin
     * (if enabled) will take the negated value of this bit.
     * 
     * @return the usbConfigPinValue
     */
    public byte getUsbConfigPinValue() {
        return usbConfigPinValue;
    }

    /**
     * Sets the initial value for the USBCFG pin. Only used when reading the settings via the
     * GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mUsbConfigPinValue
     *            the usbConfigPinValue to set
     */
    public void setUsbConfigPinValue(byte mUsbConfigPinValue) {
        this.usbConfigPinValue = mUsbConfigPinValue;
    }

    /**
     * Gets the initial value for SSPND pin option.This value represents the logic level signaled
     * when the device is not in Suspend mode. Upon entering Suspend mode, the SSPND pin (if
     * enabled) will take the negated value of this bit.
     * 
     * @return the suspendPinValue
     */
    public byte getSuspendPinValue() {
        return suspendPinValue;
    }

    /**
     * Sets the SSPND pin value. Only used when reading the settings via the GetSramSettings.
     * Altering this parameter does not affect the MCP2221's settings. ram mSuspendPinValue the
     * suspendPinValue to set
     * 
     * @param mSuspendPinValue
     *            the suspendPinValue to set
     */
    public void setSuspendPinValue(byte mSuspendPinValue) {
        this.suspendPinValue = mSuspendPinValue;
    }

    /**
     * Gets the CDC Serial Number Enumeration Enable value.
     * 
     * @return the cdcSerialNumberEnumeration : 0 = No serial number descriptor will be presented
     *         during the USB enumeration. 1 = The USB serial number will be used during the USB
     *         enumeration of the CDC interface
     */
    public byte isCdcSerialNumberEnumeration() {
        return cdcSerialNumberEnumeration;
    }

    /**
     * Sets the CDC Serial Enumeration Enable value. Only used when reading the settings via the
     * GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mCdcSerialNumberEnumeration
     *            the cdcSerialNumberEnumeration to set
     */
    public void setCdcSerialNumberEnumeration(byte mCdcSerialNumberEnumeration) {
        this.cdcSerialNumberEnumeration = mCdcSerialNumberEnumeration;
    }

    /**
     * Gets the initial value for the LEDUARTRX pin option. This value represents the logic level
     * signaled when no UART RX activity takes places. When the UART RX (of the MCP2221) is
     * receiving data, the LEDUARTRX pin will take the negated value of this bit.
     * 
     * @return the ledUartRxInitialValue
     */
    public byte getLedUartRxInitialValue() {
        return ledUartRxInitialValue;
    }

    /**
     * Sets the initial value for the LEDUARTRX pin option. Only used when reading the settings via
     * the GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mLedUartRxInitialValue
     *            the ledUartRxInitialValue to set
     */
    public void setLedUartRxInitialValue(byte mLedUartRxInitialValue) {
        this.ledUartRxInitialValue = mLedUartRxInitialValue;
    }

    /**
     * Gets the initial value for LEDUARTTX pin option. This value represents the logic level
     * signaled when no UART RX activity takes places. When the UART TX (of the MCP2221) is sending
     * data, the LEDUARTTX pin will take the negated value of this bit.
     * 
     * @return the ledUartTxInitialValue
     */
    public byte getLedUartTxInitialValue() {
        return ledUartTxInitialValue;
    }

    /**
     * Sets the initial value for the LEDUARTTX Only used when reading the settings via the
     * GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mLedUartTxInitialValue
     *            the ledUartTxInitialValue to set
     */
    public void setLedUartTxInitialValue(byte mLedUartTxInitialValue) {
        this.ledUartTxInitialValue = mLedUartTxInitialValue;
    }

    /**
     * Gets the initial value for LEDI2C pin option. This value represents the logic level signaled
     * when no I2C traffic occurs. When I2C traffic is active, the LEDI2C pin (if enabled) will take
     * the negated value of this bit.
     * 
     * @return the ledI2cInitialValue
     */
    public byte getLedI2cInitialValue() {
        return ledI2cInitialValue;
    }

    /**
     * Sets the initial value for LEDI2C pin option. Only used when reading the settings via the
     * GetSramSettings. Altering this parameter does not affect the MCP2221's settings.
     * 
     * @param mLedI2cInitialValue
     *            the ledI2cInitialValue to set
     */
    public void setLedI2cInitialValue(byte mLedI2cInitialValue) {
        this.ledI2cInitialValue = mLedI2cInitialValue;
    }

    /**
     * Get the GP pin designations.
     * 
     * @return (byte[]) - the gpPinDesignations. Possible values:
     *         <p>
     *         GP0: 2 = Alternate function 0 (LED UART RX), 1 = Dedicated function operation
     *         (SSPND), 0 = GPIO operation
     *         <p>
     *         GP1: 4 = Alternate function 2 (Interrupt Detection), 3 = Alternate function 1 (LED
     *         UART TX), 2 = Alternate function 0 (ADC1), 1 = Dedicated function operation (Clock
     *         Output), 0 = GPIO operation
     *         <p>
     *         GP2: 3 = Alternate function 1 (DAC1), 2 = Alternate function 0 (ADC2), 1 = Dedicated
     *         function operation (USBCFG), 0 = GPIO operation
     *         <p>
     *         GP3: 3 - Alternate function 1 (DAC2), 2 = Alternate function 0 (ADC3), 1 = Dedicated
     *         function operation (LED I2C), 0 = GPIO operation
     */
    public byte[] getGpPinDesignations() {
        return gpPinDesignations;
    }

    /**
     * Set the GP pin designations.
     * 
     * @param mGpPinDesignations
     *            (byte[]) - the gpPinDesignations. Possible values:
     *            <p>
     *            GP0: 2 = Alternate function 0 (LED UART RX), 1 = Dedicated function operation
     *            (SSPND), 0 = GPIO operation
     *            <p>
     *            GP1: 4 = Alternate function 2 (Interrupt Detection), 3 = Alternate function 1 (LED
     *            UART TX), 2 = Alternate function 0 (ADC1), 1 = Dedicated function operation (Clock
     *            Output), 0 = GPIO operation
     *            <p>
     *            GP2: 3 = Alternate function 1 (DAC1), 2 = Alternate function 0 (ADC2), 1 =
     *            Dedicated function operation (USBCFG), 0 = GPIO operation
     *            <p>
     *            GP3: 3 - Alternate function 1 (DAC2), 2 = Alternate function 0 (ADC3), 1 =
     *            Dedicated function operation (LED I2C), 0 = GPIO operation
     * 
     */
    public void setGpPinDesignations(byte[] mGpPinDesignations) {
        this.gpPinDesignations = mGpPinDesignations;
    }

    /**
     * Gets the GP pin directions.
     * 
     * @return the gpPinDirections (byte[]) - the GP pin directions. 0 = output, 1 = input or 0xEF
     *         if the pin isn't configured for GPIO operation
     */
    public byte[] getGpPinDirections() {
        return gpPinDirections;
    }

    /**
     * Sets the GP pin directions.
     * 
     * @param mGpPinDirections
     *            (byte[]) - 0 = output, 1 = input
     * 
     */
    public void setGpPinDirections(byte[] mGpPinDirections) {
        this.gpPinDirections = mGpPinDirections;
    }

    /**
     * Gets the GP pin values.
     * 
     * @return the gpPinValues (byte[]) - 0 = logic low, 1 = logic high or 0xEE if the pin isn't
     *         configured for GPIO operation
     */
    public byte[] getGpPinValues() {
        return gpPinValues;
    }

    /**
     * Sets the GP pin values.
     * 
     * @param mGpPinValues
     *            (byte[]) - the pin values to set. <br>
     *            0 = logic low, 1 = logic high
     */
    public void setGpPinValues(byte[] mGpPinValues) {
        this.gpPinValues = mGpPinValues;
    }

    /**
     * Gets the configuration for the interrupt on change pin.
     * 
     * @return the interruptPinMode (byte) - 1 = rising edges, 2 = falling edges, 3 = both
     */
    public byte getInterruptPinMode() {
        return interruptPinMode;
    }

    /**
     * Sets the configuration for the interrupt on change pin.
     * 
     * @param mInterruptPinMode
     *            (byte) the interruptPinMode to set - 1 = rising edges, 2 = falling edges, 3 = both
     */
    public void setInterruptPinMode(byte mInterruptPinMode) {
        this.interruptPinMode = mInterruptPinMode;
    }

    /**
     * Gets the duty cycle value for the clock out pin.
     * 
     * @return (byte) the gpClockDutyCycle <br>
     *         0 = 0 % <br>
     *         1 = 25 % <br>
     *         2 = 50 % <br>
     *         3 = 75 %
     */
    public byte getGpClockDutyCycle() {
        return gpClockDutyCycle;
    }

    /**
     * Sets the duty cycle value for the clock out pin.
     * 
     * @param mGpClockDutyCycle
     *            (byte) - the gpClockDutyCycle to set <br>
     *            0 = 0 % <br>
     *            1 = 25 % <br>
     *            2 = 50 % <br>
     *            3 = 75 %
     */
    public void setGpClockDutyCycle(byte mGpClockDutyCycle) {
        this.gpClockDutyCycle = mGpClockDutyCycle;
    }

    /**
     * Gets the clock divider value used for the clock out pin.
     * 
     * @return (byte) the gpClockDividerValue <br>
     *         1 = 24 MHz <br>
     *         2 = 12 MHz (default) <br>
     *         3 = 6 MHz <br>
     *         4 = 3 MHz <br>
     *         5 = 1.5 MHz <br>
     *         6 = 750 kHz <br>
     *         7 = 375 kHz
     */
    public byte getGpClockDividerValue() {
        return gpClockDividerValue;
    }

    /**
     * Sets the clock divider value used for the clock out pin.
     * 
     * @param mGpClockDividerValue
     *            (byte) - the gpClockDividerValue to set <br>
     *            1 = 24 MHz <br>
     *            2 = 12 MHz (default) <br>
     *            3 = 6 MHz <br>
     *            4 = 3 MHz <br>
     *            5 = 1.5 MHz <br>
     *            6 = 750 kHz <br>
     *            7 = 375 kHz
     */
    public void setGpClockDividerValue(byte mGpClockDividerValue) {
        this.gpClockDividerValue = mGpClockDividerValue;
    }

    /**
     * Gets the DAC voltage reference.
     * 
     * @return (byte) the dacVoltageReference <br>
     *         0 = Vdd <br>
     *         3 = 1.024V <br>
     *         5 = 2.048V <br>
     *         7 = 4.096V
     */
    public byte getDacVoltageReference() {
        // if bit 0 is 0 then vref = Vdd
        if ((dacVoltageReference & 0x1) == 0) {
            return 0;
        } else {
            return dacVoltageReference;
        }

    }

    /**
     * Sets the DAC voltage reference.
     * 
     * @param mDacVoltageReference
     *            (byte) - the dacVoltageReference to set <br>
     *            0 = Vdd <br>
     *            3 = 1.024V <br>
     *            5 = 2.048V <br>
     *            7 = 4.096V
     */
    public void setDacVoltageReference(byte mDacVoltageReference) {
        this.dacVoltageReference = mDacVoltageReference;
    }

    /**
     * Gets the DAC value.
     * 
     * @return the dacValue (byte) - value between 0 and 31.
     */
    public byte getDacValue() {
        return dacValue;
    }

    /**
     * Sets the DAC value.
     * 
     * @param mDacValue
     *            (byte) the dacValue to set. Value must be between 0 and 31.
     */
    public void setDacValue(byte mDacValue) {
        this.dacValue = mDacValue;
    }

    /**
     * Gets the ADC voltage reference.
     * 
     * @return (byte) the adcVoltageReference <br>
     *         0 = Vdd <br>
     *         3 = 1.024V <br>
     *         5 = 2.048V <br>
     *         7 = 4.096V
     */
    public byte getAdcVoltageReference() {
        // if bit 0 is 0 then vref = Vdd
        if ((adcVoltageReference & 0x1) == 0) {
            return 0;
        } else {
            return adcVoltageReference;
        }
    }

    /**
     * Sets the ADC voltage reference.
     * 
     * @param mAdcVoltageReference
     *            (byte) the adcVoltageReference to set <br>
     *            0 = Vdd <br>
     *            3 = 1.024V <br>
     *            5 = 2.048V <br>
     *            7 = 4.096V
     */
    public void setAdcVoltageReference(byte mAdcVoltageReference) {
        this.adcVoltageReference = mAdcVoltageReference;
    }

}
