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

import android.os.Handler;

import com.microchip.android.microchipusb.MCP2221;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;

/**
 * MCP2221 Communication Library.
 * 
 */
public class Mcp2221Comm {
    private static final int VREF_4096MV = 0x7;
    private static final int VREF_2048MV = 0x5;
    private static final int VREF_1024MV = 0X3;
    private static final int VREF_VDD = 0;
    private static final int BITMASK_INTERRUPT_BOTH_EDGES = 0x1E;
    private static final int BITMASK_INTERRUPT_FALLING_EDGE = 0x16;
    private static final int BITMASK_INTERRUPT_RISING_EDGE = 0x1c;
    private static final int INTERRUPT_BOTH_EDGES = 3;
    private static final int INTERRUPT_FALLING_EDGE = 2;
    private static final int INTERRUPT_RISING_EDGE = 1;
    private static final int INTERRUPT_OFF = 0;
    /** maximum number of data bytes that can be included in a USB packet. */
    private static final int MAX_USB_BYTES = 60;
    /** Maximum number of bytes that can be written/read from the MCP2221. */
    private static final int I2C_MAX_BYTES = 65535;
    /** Maximum size for the USB packet. */
    private static final int I2C_BRIDGE_MAX_DATA_PACKET = 64;
    /** Values for calculating the CRC on SMBus transactions. */
    private static final byte[] CRC_TABLE = { 0x00, 0x07, 0x0E, 0x09, 0x1C, 0x1B, 0x12, 0x15, 0x38,
            0x3F, 0x36, 0x31, 0x24, 0x23, 0x2A, 0x2D };
    /** Used for setSramSettings to enable changing settings. MSB states if a new value will be used */
    private static final int BITMASK_LOAD_NEW_VALUE = 0x80;
    private static final int CURRENT_SETTINGS_ONLY = 0;
    private static final int PWRUP_DEFAULTS_ONLY = 1;
    private static final int BOTH = 2;
    /** MCP2221 object used to send/receive data over the USB interface. */
    private final MCP2221 mcp2221;
    /** Byte sequence that will transmitted to the MCP2221. */
    private final ByteBuffer mTxData = ByteBuffer.allocate(I2C_BRIDGE_MAX_DATA_PACKET);
    /** Buffer to store the reply received from the MCP2221. */
    private ByteBuffer mRxData = ByteBuffer.allocate(I2C_BRIDGE_MAX_DATA_PACKET);
    /** Contains all the configuration data for the MCP2221. */
    private Mcp2221Config mMcp2221Config = new Mcp2221Config();
    /** used to make sure the rx and tx bytebuffers are reset to all 0's. */
    private byte[] clearBuffer = new byte[64];

    /**
     * Creates a new MCP2221.
     * 
     * @param superMcp2221
     *            (MCP2221) -A reference to an MCP2221 object created in the app<br>
     *            to which a connection was successfully opened.
     */
    public Mcp2221Comm(final MCP2221 superMcp2221) {
        super();
        this.mcp2221 = superMcp2221;
    }

    /**
     * Read data from the I2C device at the specified address.
     * 
     * @param i2cAddress
     *            (byte) - The I2C slave address of the device from which we wish to receive the I2C
     *            data
     * @param i2cDataReceived
     *            (ByteBuffer) - The data that was read from the I2C Slave chip
     * @param numberOfBytesToRead
     *            (int) - The number of bytes we want to read from the I2C Slave chip
     * @param i2cBusSpeed
     *            (int) - The I2C communication speed
     * @return (int) - If successful, returns 0. A value less than 0 indicates an error.
     */
    public int readI2cData(final byte i2cAddress, final ByteBuffer i2cDataReceived,
            final int numberOfBytesToRead, final int i2cBusSpeed) {

        int uiLclTimeToSleep;
        int uiLclRetryCount;
        int uiTempSleep;
        byte ucLclBitRateDivider;
        int iLclTimeout;
        int uiLclRxedLength;
        byte forceCurrentI2cTransfer = 1;

        if (numberOfBytesToRead > I2C_MAX_BYTES) {
            // the requested transfer length is larger than the maximum allowed by the chip itself
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        // now send the speed and the I2C address we want to communicate with
        // try to send the settings to the chip
        // the settings are sent as a STATUS cmd
        // compute the real value for bit-rate divider
        ucLclBitRateDivider = (byte) (48000000 / (4 * i2cBusSpeed) - 1 - 2);
        // 27 for 400 KHz

        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // no use
        mTxData.put(1, (byte) 0);
        // CANCEL XFER sub-cmd
        mTxData.put(2, (byte) 0);
        // SET XFER Speed sub-cmd
        mTxData.put(3, Mcp2221Constants.SUBCMD_I2C_SET_TRANSFER_SPEED);
        // send the bit-rate divider value
        mTxData.put(4, ucLclBitRateDivider);
        // SET ADDRESS sub-cmd - not used anymore
        mTxData.put(5, (byte) 0);
        // LSB of the I2C Slave address - not used anymore
        mTxData.put(6, (byte) 0);
        // MSB of the I2C Slave address - not used anymore
        mTxData.put(7, (byte) 0);

        // send the above data and check the response we get back
        // send the data report
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData == null) {
            // Check for error
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // STATUS command returned an error
            return Mcp2221Constants.ERROR_I2C_STATUS;
        }

        if (mRxData.get(3) != Mcp2221Constants.USB_RESP_SETXFERSPEED_OK) {
            // the requested xfer speed cannot be set - the I2C module might be in a timeout
            // situation and in this case, the PC host has to decide what to do if the
            // "forceCurrentI2CXfer" is set, then we will have to clear this state in the MCP2221
            // and prepare the chip for the current requested transfer
            if (forceCurrentI2cTransfer != 0) {
                // force a STOP condition into the SCL/SDA lines
                mTxData.put(2, Mcp2221Constants.SUBCMD_CANCEL_TRANSFER);

                // send the data and wait for the response
                mRxData = mcp2221.sendData(mTxData);
                if (mRxData == null) {
                    // Check for error
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // STATUS command returned an error
                    return Mcp2221Constants.ERROR_I2C_STATUS;
                }
            } else {
                return Mcp2221Constants.ERROR_I2C_SETSPEED;
            }
        }

        // if we got here - then the settings in the chip went fine

        // Ensure that array is valid
        if (i2cDataReceived == null) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_2;
        }

        if (numberOfBytesToRead > I2C_MAX_BYTES) {
            // the user wants to read more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_DATA_LEN;
        }

        // prepare the data to be sent over
        mTxData.put(0, (byte) Mcp2221Constants.USB_CMD_I2CM_RDDATA7);
        // LSB - transfer length
        mTxData.put(1, (byte) (numberOfBytesToRead & 0xFF));
        // MSB - transfer length
        mTxData.put(2, (byte) ((numberOfBytesToRead & 0xFF00) >> 8));
        // the I2C Slave address goes in here
        mTxData.put(3, i2cAddress);

        // send the data report
        mRxData = mcp2221.sendData(mTxData);
        // Check for error
        if (mRxData == null) {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // xfer wasn't possible
            return Mcp2221Constants.ERROR_I2C_READ001;
        }

        // for the short buffers (<60) we add 1 unit to be able to retrieve the data

        // multiply by 3 the number of retrieve data attempts per 60 bytes buffer
        iLclTimeout = (numberOfBytesToRead / MAX_USB_BYTES + 1) * 3;

        // how much data we got so far
        uiLclRxedLength = 0;
        // load the local retry counter
        uiLclRetryCount = Mcp2221Constants.MAX_RETRY_COUNT;
        // from measurements MCP2220(read 60 bytes@400KHz) < 3.3ms
        // 60 bytes@400KHz < 3.3ms; 60bytes@guiCommSpeed ?msec
        // 1/400,000 ... 3.3msec
        // 1/guiCommSpeed ... y msec
        // y = (1/x * 3.3)/(1/400000) = (3.3 * 400000)/guiCommSpeed
        uiLclTimeToSleep = (int) (3.3 * 400000 / i2cBusSpeed);
        uiTempSleep = uiLclTimeToSleep;

        mTxData.put(0, (byte) Mcp2221Constants.CMD_I2C_READDATA);
        // not really needed - but just in case
        mTxData.put(1, (byte) 0);
        // not really needed - but just in case
        mTxData.put(2, (byte) 0);
        // not really needed - but just in case
        mTxData.put(3, (byte) 0);

        // enter the while when we issue a normal read with atleast 1 user byte or in the case
        // when we just want to read without any user data - e.g. scan for I2c devices
        while (uiLclRxedLength < numberOfBytesToRead || numberOfBytesToRead == 0 && iLclTimeout > 0) {

            // Sleep for a while to give time to the MCP2220 to get its data
            // from I2C Slaves
            try {
                Thread.sleep(uiTempSleep);
            } catch (final InterruptedException e) {
                // //e.printStackTrace();
            } // x is computed from the amount of data to be read and the comm
              // speed
              // now, let's try to read whatever data the chip has
              // send the data report

            mRxData = mcp2221.sendData(mTxData);
            if (mRxData == null) {
                return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
            }

            if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                return Mcp2221Constants.ERROR_I2C_READ002;
            }

            // check if we got back a length of 0 - e.g. used in bus scan
            if (mRxData.get(3) == 0x00) {
                // break the loop since we've finished this command
                break;
            }

            if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP) {
                // the I2C Slave didn't replied to the I2C Slave address sent to it I2C Addr NACK
                return Mcp2221Constants.ERROR_I2C_ADDRNACK;
            }

            // now check if the returned small buffer length is out of range
            if (mRxData.get(3) > MAX_USB_BYTES) {
                // this means the read command is not completed
                if (uiLclRetryCount > 0) {
                    // we need to give the chip another chance change the sleep to the minimum
                    // amount (1ms)
                    uiTempSleep = uiLclTimeToSleep / 2;
                    uiLclRetryCount--;
                    continue;
                } else {
                    // we gave the chip a few chances to retrieve the data
                    return Mcp2221Constants.ERROR_I2C_READ003;
                }
            }

            if (uiLclRxedLength + mRxData.get(3) > numberOfBytesToRead) {
                // we have more data then the user wants to read
                return Mcp2221Constants.ERROR_I2C_READ004;
            }

            final byte readByteCount = mRxData.get(3);

            // now copy up to 60 bytes of user data
            for (int i = 0; i < mRxData.get(3); i++) {
                // copy the user data
                i2cDataReceived.put(i, mRxData.get(4 + i));
            }

            // if we got here it means we were able to read the data from the
            // chip
            uiLclRxedLength += readByteCount;
            iLclTimeout--;
            // re-load the local retry counter
            uiLclRetryCount = Mcp2221Constants.MAX_RETRY_COUNT;
            // restore the sleep time as calculated
            uiTempSleep = uiLclTimeToSleep;
        }

        // command completed succesfully
        return Mcp2221Constants.ERROR_SUCCESSFUL;
    }

    /**
     * Send the SMB read block command and get the data.
     * 
     * @param smbAddress
     *            (byte) - the I2C/SMB address of the slave we want to read data from
     * @param smbDataToRead
     *            (ByteBuffer) - data to read from the I2C/SMB device
     * @param numberOfBytesToRead
     *            (int) - Number of bytes to read
     * @param smbSpeed
     *            (int) - the communication speed used
     * @param usesPEC
     *            (byte) - use PEC or not
     * @param readRegIndex
     *            (int) - the register index (as per SMB specs) we will use to read data from
     * @return (int) - If successful, returns 0. A value less than 0 indicates an error (read
     *         failed).
     */
    public int smbReadBlock(final byte smbAddress, final ByteBuffer smbDataToRead,
            final int numberOfBytesToRead, final int smbSpeed, final byte usesPEC,
            final byte readRegIndex) {

        int uiLclTimeout;
        int uiLclDataIndex;
        int uiLclTxedLength;
        int uiTxBufferLen;
        int uiLclTimeToSleep;
        int uiLclRetryCount;
        int uiTempSleep;
        int iLclTimeout;
        int uiLclRxedLength;
        int uiLclCounter;
        int uiLclXferLen;
        byte ucOldCrc;
        byte ucCrc = 0;
        int ucRxedPEC = 0;
        byte ucLclBitRateDivider;
        boolean forceCurrentTransfer = true;

        if (numberOfBytesToRead > I2C_MAX_BYTES) {
            // the user wants to write more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        if (numberOfBytesToRead > 65534 && usesPEC == 1) {
            // the user wants to read more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        // try to send the settings to the chip
        // the settings are sent as a STATUS cmd
        // compute the real value for bit-rate divider
        // 27 for 400KHz
        ucLclBitRateDivider = (byte) (48000000 / (4 * smbSpeed) - 1 - 2);

        // check the status here and wait till all the data is sent
        mTxData.clear();
        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // no use
        mTxData.put(1, (byte) 0);
        // CANCEL XFER sub-cmd - not used
        mTxData.put(2, (byte) 0);
        // SET XFER Speed sub-cmd - anything other than 0
        mTxData.put(3, Mcp2221Constants.SUBCMD_I2C_SET_TRANSFER_SPEED);
        // send the bit-rate divider value
        mTxData.put(4, ucLclBitRateDivider);
        // SET ADDRESS sub-cmd - not used
        mTxData.put(5, (byte) 0);
        // LSB ADDRESS - not used
        mTxData.put(6, (byte) 0);
        // MSB ADDRESS - not used
        mTxData.put(7, (byte) 0);

        // send the above data and check the response we get back
        mRxData.clear();
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData == null) {
            // Check for error
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // STATUS command returned an error
            return Mcp2221Constants.ERROR_I2C_STATUS;
        }

        if (mRxData.get(3) != Mcp2221Constants.USB_RESP_SETXFERSPEED_OK) {
            // the requested xfer speed cannot be set
            // force a STOP condition into the SCL/SDA lines
            if (forceCurrentTransfer) {
                // CANCEL XFER sub-cmd
                mTxData.put(2, Mcp2221Constants.SUBCMD_CANCEL_TRANSFER);

                // send the above data and check the response we get back
                // send the data report
                mRxData.clear();
                mRxData = mcp2221.sendData(mTxData);
                if (mRxData == null) {
                    // Check for error
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // STATUS command returned an error
                    return Mcp2221Constants.ERROR_I2C_STATUS;
                }
            } else {
                return Mcp2221Constants.ERROR_I2C_SETSPEED;
            }
        }

        // if we got here - then everything went fine

        // prepare the control variables
        uiLclDataIndex = 0;
        // for the short buffers (<60) we add 1 unit to be able to retrieve the
        // data
        uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;
        // how much data we need to write first
        // we need to send only 1 user byte - the read index
        uiLclTxedLength = 1;
        uiLclDataIndex = 0;
        // load the local retry counter
        uiLclRetryCount = Mcp2221Constants.MAX_RETRY_COUNT;
        // compute the optimal Sleep value in msec
        uiLclTimeToSleep = (int) (3.3 * 400000 / smbSpeed);
        uiTempSleep = uiLclTimeToSleep;

        // prepare the data to be sent over - no STOP condition at the end of
        // transfer
        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.CMD_I2C_WRDATA7_NOSTOP);
        // LSB - transfer length
        mTxData.put(1, (byte) 1);
        // MSB - transfer length
        mTxData.put(2, (byte) 0);
        // the I2C slave address to use
        mTxData.put(3, smbAddress);

        // we need to send only 1 user byte
        uiLclTxedLength = 1;
        // copy the only user data byte we need to send to the device
        mTxData.put(4, readRegIndex);

        while (uiLclTxedLength > 0) {
            // we have data to send

            // 60bytes is the maximum amount the chip can accept for sending
            if (uiLclTxedLength > MAX_USB_BYTES) {
                // we have more than 60 bytes to send - take 60 bytes and take some
                // more at the next iteration
                uiTxBufferLen = MAX_USB_BYTES;
            } else {
                // we have less or equal to 60 bytes to send
                uiTxBufferLen = uiLclTxedLength;
            }

            // no user data bytes to copy from
            // we're just sending the address and the read index register
            while (uiLclTimeout > 0) {
                // send the above data and check the response we get back
                // send the data report
                mRxData.clear();
                mRxData = mcp2221.sendData(mTxData);
                if (mRxData == null) {
                    // Check for error
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                // check the chip's reply
                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // xfer wasn't possible check if we have a timeout situation
                    if (mRxData.get(2) != Mcp2221Constants.I2CM_SM_START_TOUT
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_READDATA_TOUT
                            || mRxData.get(2) != Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                        // we have a timeout situation
                        return Mcp2221Constants.ERROR_TIMEOUT;
                    }

                    uiLclTimeout--;
                    try {
                        Thread.sleep(uiTempSleep / 2);
                    } catch (final InterruptedException e) {
                        // e.printStackTrace();
                    }
                } else {
                    // the command was completed successfully
                    // break the loop
                    break;
                }
            }

            if (uiLclTimeout == 0) {
                // the data could not be sent
                return Mcp2221Constants.ERROR_I2C_SEND_ERR;
            }

            // data was sent - update the variables
            uiLclTxedLength = uiLclTxedLength - uiTxBufferLen;
            uiLclDataIndex = uiLclDataIndex + uiTxBufferLen;
            // re-load the retry counter
            uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;

            // Sleep for the computed amount of time - to allow the chip to send
            // its data buffer
            try {
                Thread.sleep(uiTempSleep);
            } catch (final InterruptedException e) {
                // e.printStackTrace();
            }
        }

        // check the status here and wait till all the data is sent
        mTxData.clear();
        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // no use
        mTxData.put(1, (byte) 0);
        // CANCEL XFER sub-cmd - not used
        mTxData.put(2, (byte) 0);
        // SET XFER Speed sub-cmd - not used
        mTxData.put(3, (byte) 0);
        // send the bit-rate divider value
        mTxData.put(4, (byte) 0);
        // SET ADDRESS sub-cmd - not used
        mTxData.put(5, (byte) 0);
        // LSB ADDRESS - not used
        mTxData.put(6, (byte) 0);
        // MSB ADDRESS - not used
        mTxData.put(7, (byte) 0);

        uiLclRetryCount = 0;
        while (uiLclTimeout > 0) {
            // send the above data and check the response we get back
            // send the data report
            mRxData.clear();
            mRxData = mcp2221.sendData(mTxData);
            if (mRxData == null) {
                // Check for error
                return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
            }
            // check the chip's reply
            if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                // xfer wasn't possible
                uiLclTimeout--;
                try {
                    Thread.sleep(uiTempSleep / 2);
                } catch (final InterruptedException e) {
                    // e.printStackTrace();
                }
            }

            // check if we need to break the loop
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRITEDATA_END_NOSTOP) {
                // the SM is finished sending dta with no STOP - break the loop
                break;
            }

            // check if we have a timeout situation
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_START_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_WAITSEND
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                // we have a timeout situation
                return Mcp2221Constants.ERROR_TIMEOUT;
            }

            // check if we need to break the loop
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_IDLE) {
                // the chip replied to the address but it hasn't ACK'ed the data
                return Mcp2221Constants.ERROR_I2C_SLAVE_DATA_NACK;
            }

            uiLclRetryCount++;
        }

        if (uiLclTimeout == 0) {
            // the data could not be sent
            return Mcp2221Constants.ERROR_I2C_SEND_ERR;
        }

        // =====================
        // now comes the read portion

        if (usesPEC == 1) {
            // we need to read one more byte - the PEC at the end of the command
            uiLclXferLen = numberOfBytesToRead + 1;
        } else {
            // no PEC involved
            uiLclXferLen = numberOfBytesToRead;
        }
        // prepare the data to be sent over
        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.CMD_I2C_RESTARTREAD_7BITS);
        // LSB - transfer length
        mTxData.put(1, (byte) (uiLclXferLen & 0xFF));
        // MSB - transfer length
        mTxData.put(2, (byte) ((uiLclXferLen & 0xFF00) >> 8));
        // the I2C/SMB slave address we want to read from
        mTxData.put(3, (byte) (smbAddress | 0x01));

        // send the above data and check the response we get back
        // send the data report
        mRxData.clear();
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData == null) {
            // Check for error
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // xfer wasn't possible
            return Mcp2221Constants.ERROR_I2C_READ001;
        }

        // for the short buffers (<60) we add 1 unit to be able to retrieve the
        // data
        iLclTimeout = (uiLclXferLen / MAX_USB_BYTES + 1) * 3;
        // multiply by 3 the number of retrieve data attempts per 60 bytes buffer how much data we
        // got so far
        uiLclRxedLength = 0;
        // load the local retry counter
        uiLclRetryCount = Mcp2221Constants.MAX_RETRY_COUNT;
        // from measurements MCP2220(read 60 bytes@400KHz) < 3.3ms
        // 60 bytes@400KHz < 3.3ms; 60bytes@guiCommSpeed ?msec
        // 1/400,000 ... 3.3msec
        // 1/guiCommSpeed ... y msec
        // y = (1/x * 3.3)/(1/400000) = (3.3 * 400000)/guiCommSpeed
        // x is computed from the amount of data to be read and the comm speed
        uiLclTimeToSleep = (int) (3.3 * 400000 / smbSpeed);
        uiTempSleep = uiLclTimeToSleep;

        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.CMD_I2C_READDATA);
        // not really needed - but just in case
        mTxData.put(1, (byte) 0);
        mTxData.put(2, (byte) 0);
        mTxData.put(3, (byte) 0);

        while ((uiLclRxedLength < uiLclXferLen || uiLclXferLen == 0) && iLclTimeout > 0) {
            // enter the while when we issue a normal read with at least 1 user byte or
            // in the case when we just want to read without any user data
            // e.g. scan for I2c devices

            // Sleep for a while to give time to the MCP2220 to get its data
            // from I2C Slaves
            try {
                Thread.sleep(uiTempSleep);
            } catch (final InterruptedException e) {
                // e.printStackTrace();
            }
            // now, let's try to read whatever data the chip has
            // send the above data and check the response we get back
            // send the data report
            mRxData.clear();
            mRxData = mcp2221.sendData(mTxData);
            if (mRxData == null) {
                // Check for error
                return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
            }

            if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                return Mcp2221Constants.ERROR_I2C_READ002;
            }

            // check if we got back a length of 0 - e.g. used in bus scan
            if (mRxData.get(3) == 0x00) {
                // break the loop since we've finished this command
                break;
            }

            if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP) {
                // the I2C Slave didn't reply to the I2C Slave address sent to it and it
                // I2C Addr NACK
                return Mcp2221Constants.ERROR_I2C_ADDRNACK;
            }

            // now check if the returned small buffer length is out of range
            if (mRxData.get(3) > MAX_USB_BYTES) {
                // this means the read command is not completed
                if (uiLclRetryCount > 0) {
                    // we need to give the chip another change
                    // change the sleep to the minimum amount (1ms)
                    uiTempSleep = uiLclTimeToSleep / 2;
                    uiLclRetryCount--;
                    continue;
                } else {
                    // we gave the chip a few chances to retrieve the data
                    return Mcp2221Constants.ERROR_I2C_READ003;
                }
            }

            if (uiLclRxedLength + mRxData.get(3) > uiLclXferLen) {
                // we have more data than the user wants to read
                return Mcp2221Constants.ERROR_I2C_READ004;
            }

            // now copy up to 60 bytes of user data
            for (uiLclCounter = 0; uiLclCounter < mRxData.get(3); uiLclCounter++) {
                if (usesPEC == 1 && uiLclRxedLength + uiLclCounter == uiLclXferLen - 1) {
                    // this is the moment to skip copying the PEC and copy it into a variable
                    ucRxedPEC = mRxData.get(4 + uiLclCounter);
                } else {
                    // copy the user data
                    smbDataToRead
                            .put(uiLclRxedLength + uiLclCounter, mRxData.get(4 + uiLclCounter));
                }
            }

            // if we got here it means we were able to read the data from the chip
            uiLclRxedLength += mRxData.get(3);
            iLclTimeout--;
            // re-load the local retry counter
            uiLclRetryCount = Mcp2221Constants.MAX_RETRY_COUNT;
            // restore the sleep time as calculated
            uiTempSleep = uiLclTimeToSleep;
        }

        // now, check if the received PEC matches the one computed for the
        // message
        if (usesPEC == 1) {
            // seed value
            ucOldCrc = 0x00;

            // all bytes on the buss need to be
            // CRC'd, including addresses
            ucCrc = crc8(smbAddress, ucOldCrc);
            ucOldCrc = ucCrc;
            ucCrc = crc8(readRegIndex, ucOldCrc);
            ucOldCrc = ucCrc;
            ucCrc = crc8((byte) (smbAddress | 0x01), ucOldCrc);

            ucOldCrc = ucCrc;
            for (uiLclCounter = 0; uiLclCounter < uiLclRxedLength - 1; uiLclCounter++) {
                ucCrc = crc8(smbDataToRead.get(uiLclCounter), ucOldCrc);
                ucOldCrc = ucCrc;
            }

            if (ucCrc != ucRxedPEC) {
                // wrong PEC - return the error code
                return Mcp2221Constants.ERROR_WRONG_PEC;
            }
        }

        // command completed succesfully
        return Mcp2221Constants.ERROR_SUCCESSFUL;
    }

    /**
     * Send the SMB write block command and the given user data.
     * 
     * @param smbAddress
     *            (byte) - the I2C/SMB address of the slave we want to write data to
     * @param smbDataToSend
     *            (ByteBuffer) - data to send to the I2C/SMB device
     * @param numberOfBytesToWrite
     *            (int) - data transfer length
     * @param smbSpeed
     *            (int) - the communication speed used
     * @param usesPEC
     *            (byte) - use PEC or not
     * @return (int) - If successful, returns 0. A value less than 0 indicates an error (write
     *         failed).
     */
    public int smbWriteBlock(final byte smbAddress, final ByteBuffer smbDataToSend,
            final int numberOfBytesToWrite, final int smbSpeed, final byte usesPEC) {

        int uiLclTimeout;
        int uiLclDataIndex;
        int uiLclCounter;
        int uiLclXferLen;
        int uiLclTxedLength;
        int uiTxBufferLen;
        int uiLclTimeToSleep;
        int uiTempSleep;
        byte ucOldCrc;
        byte ucCrc = 0;
        byte ucLclBitRateDivider;
        boolean forceCurrentTransfer = true;

        if (numberOfBytesToWrite > I2C_MAX_BYTES) {
            // the user wants to write more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        if (numberOfBytesToWrite > 65534 && usesPEC == 1) {
            // the user wants to write more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        // try to send the settings to the chip
        // the settings are sent as a STATUS cmd
        // compute the real value for bit-rate divider
        // 27 for 400KHz
        ucLclBitRateDivider = (byte) (48000000 / (4 * smbSpeed) - 1 - 2);

        // check the status here and wait till all the data is sent
        mTxData.clear();
        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // no use
        mTxData.put(1, (byte) 0);
        // CANCEL XFER sub-cmd - not used
        mTxData.put(2, (byte) 0);
        // SET XFER Speed sub-cmd - anything other than 0
        mTxData.put(3, Mcp2221Constants.SUBCMD_I2C_SET_TRANSFER_SPEED);
        // send the bit-rate divider value
        mTxData.put(4, ucLclBitRateDivider);
        // SET ADDRESS sub-cmd - not used
        mTxData.put(5, (byte) 0);
        // LSB ADDRESS - not used
        mTxData.put(6, (byte) 0);
        // MSB ADDRESS - not used
        mTxData.put(7, (byte) 0);

        // send the above data and check the response we get back
        mRxData = mcp2221.sendData(mTxData);

        // Check to see if the operation was successful
        if (mRxData == null) {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // STATUS command returned an error
            return Mcp2221Constants.ERROR_I2C_STATUS;
        }
        // if the requested xfer speed cannot be set
        if (mRxData.get(3) != Mcp2221Constants.USB_RESP_SETXFERSPEED_OK) {
            // force a STOP condition into
            // the SCL/SDA lines
            if (forceCurrentTransfer) {
                // CANCEL XFER sub-cmd
                mTxData.put(2, Mcp2221Constants.SUBCMD_CANCEL_TRANSFER);

                // send the above data and check the response we get back
                mRxData = mcp2221.sendData(mTxData);

                // Check to see if the operation was successful
                if (mRxData == null) {
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // STATUS command returned an error
                    return Mcp2221Constants.ERROR_I2C_STATUS;
                }
            } else {
                return Mcp2221Constants.ERROR_I2C_SETSPEED;
            }
        }

        // if we got here - then everything went fine

        // prepare the control variables
        uiLclDataIndex = 0;
        // for the short buffers (<60) we add 1 unit to be able to retrieve the
        // data
        uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;
        // how much data we got so far
        if (usesPEC == 1) {
            uiLclXferLen = numberOfBytesToWrite + 1;
            uiLclTxedLength = uiLclXferLen;
            // update the PEC value for the given user data block
            // address needs to be crc'd as well
            // seed value
            ucOldCrc = 0x00;
            ucCrc = crc8(smbAddress, ucOldCrc);
            ucOldCrc = ucCrc;
            for (uiLclCounter = 0; uiLclCounter < uiLclXferLen - 1; uiLclCounter++) {
                ucCrc = crc8(smbDataToSend.get(uiLclCounter), ucOldCrc);
                ucOldCrc = ucCrc;
            }
        } else {
            uiLclXferLen = numberOfBytesToWrite;
            uiLclTxedLength = uiLclXferLen;
        }

        uiLclDataIndex = 0;
        // compute the optimal Sleep value in msec
        uiLclTimeToSleep = (int) (3.3 * 400000 / smbSpeed);
        uiTempSleep = uiLclTimeToSleep;

        // prepare the data to be sent over
        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.USB_CMD_I2CM_WRDATA7);
        // LSB - transfer length
        mTxData.put(1, (byte) (uiLclXferLen & 0xFF));
        // MSB - transfer length
        mTxData.put(2, (byte) ((uiLclXferLen & 0xFF00) >> 8));
        // the I2C/SMB slave address to use
        mTxData.put(3, smbAddress);

        if (uiLclTxedLength == 0) {
            // no user data to be sent - can be used for scanning the devices
            uiTxBufferLen = uiLclTxedLength;

            // now copy up to 60 bytes of user data
            for (uiLclCounter = 0; uiLclCounter < uiTxBufferLen; uiLclCounter++) {
                if (usesPEC == 1 && uiLclDataIndex + uiLclCounter == uiLclTxedLength - 1) {
                    // this is the moment to copy the computed PEC value
                    mTxData.put(4 + uiLclCounter, ucCrc);

                } else {
                    // copy the user data
                    mTxData.put(4 + uiLclCounter, smbDataToSend.get(uiLclDataIndex + uiLclCounter));
                }
            }

            while (uiLclTimeout > 0) {
                // send the above data and check the response we get back
                // send the data report
                mRxData.clear();
                mRxData = mcp2221.sendData(mTxData);
                // Check to see if the operation was successful
                if (mRxData == null) {
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                // check the chip's reply
                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // xfer wasn't possible. Check if we have a timeout situation
                    if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_START_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                        // we have a timeout situation
                        return Mcp2221Constants.ERROR_TIMEOUT;
                    }

                    uiLclTimeout--;
                    try {
                        Thread.sleep(uiTempSleep / 2);
                    } catch (final InterruptedException e) {
                        // e.printStackTrace();
                    }
                } else {
                    // the command was completed successfully
                    // break the loop
                    break;
                }
            }

            if (uiLclTimeout == 0) {
                // the data could not be sent
                return Mcp2221Constants.ERROR_I2C_SEND_ERR;
            }

            // data was sent - update the variables
            uiLclTxedLength = uiLclTxedLength - uiTxBufferLen;
            uiLclDataIndex = uiLclDataIndex + uiTxBufferLen;
            // re-load the retry counter
            uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;

            // Sleep for the computed amount of time - to allow the chip to send
            // its data buffer
            try {
                Thread.sleep(uiTempSleep);
            } catch (final InterruptedException e) {
                // e.printStackTrace();
            }
        } else {
            // in case we have more than 0 user bytes to send
            while (uiLclTxedLength > 0) {
                // we have data to send

                // 60bytes is the maximum amount the chip can accept for sending
                if (uiLclTxedLength > MAX_USB_BYTES) {
                    // we have more than 60 bytes to send - take 60 bytes
                    // and take
                    // some more at the next iteration
                    uiTxBufferLen = MAX_USB_BYTES;
                } else {
                    // we have less or equal to 60 bytes to send
                    uiTxBufferLen = uiLclTxedLength;
                }

                // now copy up to 60 bytes of user data
                for (uiLclCounter = 0; uiLclCounter < uiTxBufferLen; uiLclCounter++) {
                    if (usesPEC == 1 && uiLclDataIndex + uiLclCounter == uiLclXferLen - 1) {
                        // this is he moment to copy the computed PEC value
                        mTxData.put(4 + uiLclCounter, ucCrc);
                    } else {
                        // copy the user data
                        mTxData.put(4 + uiLclCounter,
                                smbDataToSend.get(uiLclDataIndex + uiLclCounter));
                    }
                }

                while (uiLclTimeout > 0) {
                    // send the above data and check the response we get back
                    mRxData.clear();
                    mRxData = mcp2221.sendData(mTxData);

                    if (mRxData == null) {
                        return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                    }

                    // check the chip's reply
                    if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                        // xfer wasn't possible. Check if we have a timeout situation
                        if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_START_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                            // we have a timeout situation
                            return Mcp2221Constants.ERROR_TIMEOUT;
                        }

                        uiLclTimeout--;
                        try {
                            Thread.sleep(uiTempSleep / 2);
                        } catch (final InterruptedException e) {
                            // e.printStackTrace();
                        }
                    } else {
                        // the command was completed successfully
                        // break the loop
                        break;
                    }
                }

                if (uiLclTimeout == 0) {
                    // the data could not be sent
                    return Mcp2221Constants.ERROR_I2C_SEND_ERR;
                }

                // data was sent - update the variables
                uiLclTxedLength = uiLclTxedLength - uiTxBufferLen;
                uiLclDataIndex = uiLclDataIndex + uiTxBufferLen;
                // re-load the retry counter
                uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;

                // Sleep for the computed amount of time - to allow the chip to
                // send its data buffer
                try {
                    Thread.sleep(uiTempSleep);
                } catch (final InterruptedException e) {
                    // e.printStackTrace();
                }
            }
        }

        // check the status here and wait till all the data is sent
        mTxData.clear();
        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // clear everything, just in case
        mTxData.put(1, (byte) 0);
        mTxData.put(2, (byte) 0);
        mTxData.put(3, (byte) 0);
        mTxData.put(4, (byte) 0);
        mTxData.put(5, (byte) 0);
        mTxData.put(6, (byte) 0);
        mTxData.put(7, (byte) 0);

        while (uiLclTimeout > 0) {
            // send the above data and check the response we get back
            mRxData.clear();
            mRxData = mcp2221.sendData(mTxData);

            if (mRxData == null) {
                return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
            }

            // check the chip's reply
            if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                // xfer wasn't possible
                uiLclTimeout--;
                try {
                    Thread.sleep(uiTempSleep / 2);
                } catch (final InterruptedException e) {
                    // e.printStackTrace();
                }
            }

            // check if we need to break the loop
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_IDLE) {
                // the SM is in IDLE - break the loop
                break;
            }

            // check if we have a timeout situation
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_START_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_WAITSEND
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                return Mcp2221Constants.ERROR_TIMEOUT;
            }
        }

        if (uiLclTimeout == 0) {
            // the data could not be sent
            return Mcp2221Constants.ERROR_I2C_SEND_ERR;
        }

        // command completed succesfully
        return Mcp2221Constants.ERROR_SUCCESSFUL;
    }

    /**
     * Write I2C data to the specified address.
     * 
     * @param i2cAddress
     *            (byte) - The I2C slave address of the device to which we wish to send the I2C data
     * @param dataToSend
     *            (ByteBuffer) - Array of bytes holding data to send
     * @param transferLength
     *            (int) - The length of dataToSend array
     * @param i2cBusSpeed
     *            (int) - The I2C communication speed
     * @return (int) - If successful, returns 0. A value less than 0 indicates an error.
     */
    public int writeI2cdata(final byte i2cAddress, final ByteBuffer dataToSend,
            final int transferLength, final int i2cBusSpeed) {
        final byte[] lclTxData = new byte[MAX_USB_BYTES];
        int uiLclTimeout;
        int uiLclDataIndex;
        int uiLclTxedLength;
        int uiTxBufferLen;
        int uiLclTimeToSleep;
        int uiTempSleep;
        byte ucLclBitRateDivider;
        byte forceCurrentI2cTransfer = 1;

        if (transferLength > I2C_MAX_BYTES) {
            // the requested transfer length is larger than the maximum allowed by the chip itself
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_3;
        }

        // now send the speed and the I2C address we want to communicate with
        // try to send the settings to the chip
        // the settings are sent as a STATUS cmd
        // compute the real value for bit-rate divider
        // 27 for 400KHz
        ucLclBitRateDivider = (byte) (48000000 / (4 * i2cBusSpeed) - 1 - 2);

        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        mTxData.put(1, (byte) 0);
        // CANCEL XFER sub-cmd
        mTxData.put(2, (byte) 0);
        // SET transfer Speed sub-cmd
        mTxData.put(3, Mcp2221Constants.SUBCMD_I2C_SET_TRANSFER_SPEED);
        // send the bit-rate divider value
        mTxData.put(4, ucLclBitRateDivider);
        // SET ADDRESS sub-cmd - not used anymore
        mTxData.put(5, (byte) 0);
        // LSB of the I2C Slave address - not used anymore
        mTxData.put(6, (byte) 0);
        // MSB of the I2C Slave address - not used anymore
        mTxData.put(7, (byte) 0);

        // send the above data and check the response we get back
        // send the data report
        mRxData = mcp2221.sendData(mTxData);

        // Check to see if the operation was successful
        if (mRxData == null) {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
            // STATUS command returned an error
            return Mcp2221Constants.ERROR_I2C_STATUS;
        }

        if (mRxData.get(3) != Mcp2221Constants.USB_RESP_SETXFERSPEED_OK) {
            // the requested xfer speed cannot be set - the I2C module might be
            // in a timeout situation
            // and in this case, the PC host has to decide what to do
            // if the "forceCurrentI2CXfer" is set, then we will have to clear
            // this state in the MCP2221
            // and prepare the chip for the current requested transfer
            if (forceCurrentI2cTransfer != 0) {
                // force a STOP condition into the SCL/SDA lines
                mTxData.put(2, Mcp2221Constants.SUBCMD_CANCEL_TRANSFER);

                // clear any previous replies
                mRxData.clear();
                // send the data and wait for the response
                mRxData = mcp2221.sendData(mTxData);

                // Check for error
                if (mRxData == null) {
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // STATUS command returned an error
                    return Mcp2221Constants.ERROR_I2C_STATUS;
                }
            } else {
                return Mcp2221Constants.ERROR_I2C_SETSPEED;
            }
        }

        // if we got here - then the settings in the chip went fine

        // Ensure that array is valid
        if (dataToSend == null) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_2;
        }

        if (transferLength > I2C_MAX_BYTES) {
            // the user wants to write more data than the chip can accomodate
            return Mcp2221Constants.ERROR_INVALID_DATA_LEN;
        }

        // prepare the control variables
        uiLclDataIndex = 0;
        // for the short buffers (<60) we add 1 unit to be able to retrieve the
        // data
        uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;
        // how much data we got so far
        uiLclTxedLength = transferLength;
        uiLclDataIndex = 0;
        // compute the optimal Sleep value in msec
        uiLclTimeToSleep = (int) (3.3 * 400000 / i2cBusSpeed);
        uiTempSleep = uiLclTimeToSleep;

        // prepare the data to be sent over
        mTxData.put(0, (byte) Mcp2221Constants.USB_CMD_I2CM_WRDATA7);
        // LSB - transfer length
        mTxData.put(1, (byte) (transferLength & 0xFF));
        // MSB - transfer length
        mTxData.put(2, (byte) ((transferLength & 0xFF00) >> 8));
        // the I2C Slave address goes in here
        mTxData.put(3, i2cAddress);

        if (uiLclTxedLength == 0) {
            // no user data to send - this part is used for scan operations
            uiTxBufferLen = uiLclTxedLength;

            // now copy up to 60 bytes of user data
            while (uiLclTimeout > 0) {
                // send the data report
                dataToSend.get(lclTxData, 0, MAX_USB_BYTES);
                mTxData.put(lclTxData, 4, MAX_USB_BYTES);
                mRxData.clear();
                mRxData = mcp2221.sendData(mTxData);

                // Check for error
                if (mRxData == null) {
                    return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                }

                // check the chip's reply
                if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                    // xfer wasn't possible check if we have a timeout situation
                    if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_START_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                            || mRxData.get(2) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                        // we have a timeout situation
                        return Mcp2221Constants.ERROR_TIMEOUT;
                    }

                    uiLclTimeout--;
                    try {
                        Thread.sleep(uiTempSleep / 2);
                    } catch (final InterruptedException e) {
                        // //e.printStackTrace();
                    }
                } else {
                    // the command was completed successfully
                    // break the loop
                    break;

                }
            }

            if (uiLclTimeout == 0) {
                // the data could not be sent
                return Mcp2221Constants.ERROR_I2C_SEND_ERR;
            }

            // data was sent - update the variables
            uiLclTxedLength = uiLclTxedLength - uiTxBufferLen;
            uiLclDataIndex = uiLclDataIndex + uiTxBufferLen;
            // re-load the retry counter
            uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;

            // Sleep for the computed amount of time - to allow the chip to send
            // its data buffer
            try {
                Thread.sleep(uiTempSleep);
            } catch (final InterruptedException e) {
                // //e.printStackTrace();
            }
        } else {
            // in case we have more than 0 user bytes to send
            while (uiLclTxedLength > 0) {
                // we have data to send
                // 60bytes is the maximum amount the chip can accept for sending
                if (uiLclTxedLength > MAX_USB_BYTES) {
                    // we have more than 60 bytes to send - take 60 bytes
                    // and take
                    // some more at the next iteration
                    uiTxBufferLen = MAX_USB_BYTES;
                } else {
                    // we have less or equal to 60 bytes to send
                    uiTxBufferLen = uiLclTxedLength;
                }

                // now copy up to 60 bytes of user data
                dataToSend.position(uiLclDataIndex);
                dataToSend.get(lclTxData, 0, uiTxBufferLen);
                // place data from position 4 onwards
                mTxData.position(4);
                mTxData.put(lclTxData, 0, uiTxBufferLen);

                while (uiLclTimeout > 0) {
                    // send the data report
                    mRxData.clear();
                    mRxData = mcp2221.sendData(mTxData);
                    if (mRxData == null) {
                        // Check for error
                        return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
                    }

                    // check the chip's reply
                    if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                        // xfer wasn't possible check if we have a timeout situation
                        if (mRxData.get(2) == Mcp2221Constants.I2CM_SM_START_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                                || mRxData.get(2) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                            // we have a timeout situation
                            return Mcp2221Constants.ERROR_TIMEOUT;
                        }

                        uiLclTimeout--;
                        try {
                            Thread.sleep(uiTempSleep / 2);
                        } catch (final InterruptedException e) {
                            // //e.printStackTrace();
                        }
                    } else {
                        // the command was completed successfully
                        // break the loop
                        break;
                    }
                }

                if (uiLclTimeout == 0) {
                    // the data could not be sent
                    return Mcp2221Constants.ERROR_I2C_SEND_ERR;
                }

                // data was sent - update the variables
                uiLclTxedLength = uiLclTxedLength - uiTxBufferLen;
                uiLclDataIndex = uiLclDataIndex + uiTxBufferLen;
                // re-load the retry counter
                uiLclTimeout = Mcp2221Constants.MAX_RETRY_COUNT;

                // Sleep for the computed amount of time - to allow the chip to
                // send its data buffer
                try {
                    Thread.sleep(uiTempSleep);
                } catch (final InterruptedException e) {
                    // //e.printStackTrace();
                }
            }
        }

        // check the status here and wait till all the data is sent
        mTxData.put(0, Mcp2221Constants.USB_CMD_STATUS);
        // clear everything
        mTxData.put(1, (byte) 0);
        mTxData.put(2, (byte) 0);
        mTxData.put(3, (byte) 0);
        mTxData.put(4, (byte) 0);
        mTxData.put(5, (byte) 0);
        mTxData.put(6, (byte) 0);
        mTxData.put(7, (byte) 0);

        while (uiLclTimeout > 0) {
            // send the data report
            mRxData.clear();
            mRxData = mcp2221.sendData(mTxData);
            if (mRxData == null) {
                // Check for error
                return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
            }

            // check the chip's reply
            if (mRxData.get(1) != Mcp2221Constants.USB_RESP_SUCCESS) {
                // xfer wasn't possible
                uiLclTimeout--;
                try {
                    Thread.sleep(uiTempSleep / 2);
                } catch (final InterruptedException e) {
                    // //e.printStackTrace();
                }
            }

            // check if we need to break the loop

            // if the I2C Slave address wasn't ACK-ed
            if ((mRxData.get(20) & 0x40) == 0x40) {
                // the SM is in IDLE - but we've had no response
                // from the I2C slave
                return Mcp2221Constants.ERROR_I2C_ADDRNACK;
            }

            // check if we need to break the loop
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_IDLE) {
                // the SM is in IDLE - break the loop
                break;
            }

            // check if we have a timeout situation
            if (mRxData.get(8) == Mcp2221Constants.I2CM_SM_START_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_REPSTART_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_WAITSEND
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRADDRL_NACK_STOP
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_WRITEDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_READDATA_TOUT
                    || mRxData.get(8) == Mcp2221Constants.I2CM_SM_STOP_TOUT) {
                // we have a timeout situation
                return Mcp2221Constants.ERROR_TIMEOUT;
            }
        }

        if (uiLclTimeout == 0) {
            // the data could not be sent
            return Mcp2221Constants.ERROR_I2C_SEND_ERR;
        }
        // command completed succesfully
        return Mcp2221Constants.ERROR_SUCCESSFUL;
    }

    /**
     * Calculate the CRC. Used for PEC calculations
     * 
     * @param newByte
     *            (byte) - byte that will be added to the CRC value
     * @param oldCrc
     *            (byte) - previous CRC value
     * @return (byte) - new CRC value
     */
    private byte crc8(final byte newByte, final byte oldCrc) {
        byte crc = oldCrc;
        crc = (byte) (crc << 4 ^ CRC_TABLE[(crc ^ newByte) >> 4 & 0x0f]);
        crc = (byte) (crc << 4 ^ CRC_TABLE[(crc >> 4 ^ newByte) & 0x0F]);
        return crc;
    }

    /******************************************************************
     * SRAM settings functions
     ******************************************************************/

    /**
     * Get the current SRAM settings of the device.
     * 
     * @param mcp2221Config
     *            (Mcp2221Config) - object that will contain the settings read from the device
     * @return (int) If successful, returns 0. A value less than 0 indicates an error.
     */
    public int getSRamSettings(Mcp2221Config mcp2221Config) {

        // Setup a buffer with command
        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.CMD_SRAM_READ);

        mRxData.clear();
        // send the command
        mRxData = mcp2221.sendData(mTxData);

        if (mRxData == null) {
            // Check for error
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }

        // check the chip's reply
        if (mRxData.get(1) == Mcp2221Constants.USB_RESP_SUCCESS) {
            mcp2221Config.setCdcSerialNumberEnumeration(getBits(mRxData.get(4), 1, 7));
            mcp2221Config.setSuspendPinValue(getBits(mRxData.get(4), 1, 3));
            mcp2221Config.setUsbConfigPinValue(getBits(mRxData.get(4), 1, 2));
            mcp2221Config.setAccessControlState(getBits(mRxData.get(4), 2, 0));
            mcp2221Config.setGpClockDividerValue(getBits(mRxData.get(5), 3, 0));
            mcp2221Config.setGpClockDutyCycle(getBits(mRxData.get(5), 2, 3));
            mcp2221Config.setDacValue(getBits(mRxData.get(6), 5, 0));
            mcp2221Config.setDacVoltageReference(getBits(mRxData.get(6), 3, 5));
            mcp2221Config.setInterruptPinMode(getBits(mRxData.get(7), 2, 5));
            mcp2221Config.setAdcVoltageReference(getBits(mRxData.get(7), 3, 2));
            mcp2221Config.setVid((mRxData.get(9) << 8) + (mRxData.get(8) & 0xff));
            mcp2221Config.setPid((mRxData.get(11) << 8) + (mRxData.get(10) & 0xff));
            mcp2221Config.setPowerAttribute(mRxData.get(12));
            mcp2221Config.setCurrentRequested(mRxData.get(13) * 2);

            ByteBuffer pinValues = ByteBuffer.allocate(4);
            ByteBuffer pinDirections = ByteBuffer.allocate(4);
            ByteBuffer pinDesignations = ByteBuffer.allocate(4);
            for (int i = 0; i < 4; i++) {
                pinValues.put(i, getBits(mRxData.get(22 + i), 1, 4));
                pinDirections.put(i, getBits(mRxData.get(22 + i), 1, 3));
                pinDesignations.put(i, getBits(mRxData.get(22 + i), 3, 0));
            }

            mcp2221Config.setGpPinValues(pinValues.array());
            mcp2221Config.setGpPinDirections(pinDirections.array());
            mcp2221Config.setGpPinDesignations(pinDesignations.array());

        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }
        return mRxData.get(1);
    }

    /**
     * Set the SRAM settings on the MPC2221.
     * 
     * @param mcp2221Config
     *            - object containing the settings for the MCP2221
     * @param setClockConfiguration
     *            (boolean) - true to configure the clock
     * @param setDacVRef
     *            (boolean) - true to set the DAC voltage reference
     * @param setDacValue
     *            (boolean) - true to configure the DAC value
     * @param setAdcVRef
     *            (boolean) - true to configure the ADC voltage reference
     * @param clearInterruptFlag
     *            (boolean) - true to clear the interrupt flag
     * @param setInterruptMode
     *            (boolean) - true to configure the interrupt mode
     * @param setGpConfiguration
     *            (boolean) - true to configure the GPIO pins
     * @return (int) - 0 if successful. A value less than 0 indicates an error.
     */
    public int setSRamSettings(Mcp2221Config mcp2221Config, boolean setClockConfiguration,
            boolean setDacVRef, boolean setDacValue, boolean setAdcVRef,
            boolean clearInterruptFlag, boolean setInterruptMode, boolean setGpConfiguration) {

        // Setup a buffer with command
        mTxData.clear();
        mTxData.put(0, (byte) Mcp2221Constants.CMD_SRAM_WRITE);
        // Reserved. Don't care
        mTxData.put(1, (byte) 0);

        if (setClockConfiguration) {
            mTxData.put(2, (byte) (BITMASK_LOAD_NEW_VALUE
                    | (mcp2221Config.getGpClockDutyCycle() << 3) | mcp2221Config
                    .getGpClockDividerValue()));
        } else {
            mTxData.put(2, (byte) 0);
        }

        if (setDacVRef) {
            mTxData.put(3, (byte) (BITMASK_LOAD_NEW_VALUE | mcp2221Config.getDacVoltageReference()));
        } else {
            mTxData.put(3, (byte) 0);
        }

        if (setDacValue) {
            mTxData.put(4, (byte) (BITMASK_LOAD_NEW_VALUE | mcp2221Config.getDacValue()));
        } else {
            mTxData.put(4, (byte) 0);
        }

        if (setAdcVRef) {
            mTxData.put(5, (byte) (BITMASK_LOAD_NEW_VALUE | mcp2221Config.getAdcVoltageReference()));
        } else {
            mTxData.put(5, (byte) 0);
        }

        if (setInterruptMode) {
            switch (mcp2221Config.getInterruptPinMode()) {
                case INTERRUPT_OFF:
                    mTxData.put(6, (byte) (BITMASK_LOAD_NEW_VALUE | 0));
                    break;
                case INTERRUPT_RISING_EDGE:
                    mTxData.put(6, (byte) (BITMASK_LOAD_NEW_VALUE | BITMASK_INTERRUPT_RISING_EDGE));
                    break;
                case INTERRUPT_FALLING_EDGE:
                    mTxData.put(6, (byte) (BITMASK_LOAD_NEW_VALUE | BITMASK_INTERRUPT_FALLING_EDGE));
                    break;
                case INTERRUPT_BOTH_EDGES:
                    mTxData.put(6, (byte) (BITMASK_LOAD_NEW_VALUE | BITMASK_INTERRUPT_BOTH_EDGES));
                    break;

                default:
                    break;
            }

        } else {
            mTxData.put(6, (byte) 0);
        }

        if (clearInterruptFlag) {
            mTxData.put(6, (byte) (mTxData.get(6) | BITMASK_LOAD_NEW_VALUE | 0x1));
        }

        if (setGpConfiguration) {
            mTxData.put(7, (byte) (BITMASK_LOAD_NEW_VALUE));

            byte[] pinDesignations = mcp2221Config.getGpPinDesignations();
            byte[] pinDirections = mcp2221Config.getGpPinDirections();
            byte[] pinValues = mcp2221Config.getGpPinValues();

            mTxData.put(8,
                    (byte) (pinDesignations[0] + (pinDirections[0] << 3) + (pinValues[0] << 4)));
            mTxData.put(9,
                    (byte) (pinDesignations[1] + (pinDirections[1] << 3) + (pinValues[1] << 4)));
            mTxData.put(10,
                    (byte) (pinDesignations[2] + (pinDirections[2] << 3) + (pinValues[2] << 4)));
            mTxData.put(11,
                    (byte) (pinDesignations[3] + (pinDirections[3] << 3) + (pinValues[3] << 4)));

        } else {
            mTxData.put(7, (byte) (0));
        }

        // Write the command to the device
        mRxData.clear();
        // send the command
        mRxData = mcp2221.sendData(mTxData);

        if (mRxData == null) {
            // Check for error
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        } else {
            return mRxData.get(1);
        }
    }

    /******************************************************************
     * GPIO functions
     ******************************************************************/

    /**
     * Get the current direction of the specified pin.
     * 
     * @param pinNumber
     *            (byte) - The pin number of the pin to get the value of.
     * @return (int) - If successful, returns pin direction (0 = output, 1 = input or 0xEF if the
     *         pin isn't configured for GPIO operation). -201 indicates an error. Notes: The
     *         direction (input/output) only matters if the pin is designated as a GPIO.
     */
    public int getGpPinDirection(byte pinNumber) {

        if (pinNumber < 0 || pinNumber > 3) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        // Setup a buffer with command
        mTxData.put(0, (byte) Mcp2221Constants.CMD_GET_GP);
        mRxData = mcp2221.sendData(mTxData);

        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {

            ByteBuffer pinDirections = ByteBuffer.allocate(4);

            pinDirections.put(0, mRxData.get(3));
            pinDirections.put(1, mRxData.get(5));
            pinDirections.put(2, mRxData.get(7));
            pinDirections.put(3, mRxData.get(9));

            return pinDirections.get(pinNumber) & 0xff;
        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }

    }

    /**
     * Set the current direction of the specified pin.
     * 
     * @param pinNumber
     *            (byte) - The pin number of the pin to set the value of (0, 1, 2, or 3 are valid
     *            values)
     * @param directionToSet
     *            (byte) - The pin direction to set on the specified pin (0 = output, 1 = input).
     * @return (int) If successful, returns 0. A value less than 0 indicates an error. Notes: The
     *         direction (input/output) only matters if the pin is designated as a GPIO.
     */
    public int setGpPinDirection(byte pinNumber, byte directionToSet) {

        // clear the tx buffer
        mTxData.clear();
        mTxData.put(clearBuffer, 0, 64);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_SET_GP);

        switch (pinNumber) {
            case 0:
                // The next byte (index 5) will be the value used to set GP0’s pin direction
                mTxData.put(4, (byte) 1);
                mTxData.put(5, directionToSet);
                break;
            case 1:
                // The next byte (index 9) will be the value used to set GP1’s pin direction
                mTxData.put(8, (byte) 1);
                mTxData.put(9, directionToSet);
                break;
            case 2:
                // The next byte (index 13) will be the value used to set GP2’s pin direction
                mTxData.put(12, (byte) 1);
                mTxData.put(13, directionToSet);
                break;
            case 3:
                // The next byte (index 17) will be the value used to set GP3’s pin direction
                mTxData.put(16, (byte) 1);
                mTxData.put(17, directionToSet);
                break;

            default:
                return Mcp2221Constants.ERROR_INVALID_PARAMETER_2;

        }

        // clear the RX buffer
        mRxData.clear();
        mRxData.put(clearBuffer, 0, 64);

        // send the command
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return Mcp2221Constants.ERROR_SUCCESSFUL;
        } else {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }
    }

    /**
     * Read the current value of the specified pin.
     * 
     * @param pinNumber
     *            (byte) - The pin number of the pin to get the value of.
     * @return (int) - If successful, returns pin value (0 = logic low, 1 = logic high or 0xEE if
     *         the pin isn't configured for GPIO operation). -201 indicates an error.
     */
    public int getGpPinValue(byte pinNumber) {

        if (pinNumber < 0 || pinNumber > 3) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        // Setup a buffer with command
        mTxData.clear();
        mTxData.put(clearBuffer);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_GET_GP);

        mRxData.clear();
        mRxData.put(clearBuffer);
        mRxData = mcp2221.sendData(mTxData);

        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {

            ByteBuffer pinValues = ByteBuffer.allocate(4);

            pinValues.put(0, mRxData.get(2));
            pinValues.put(1, mRxData.get(4));
            pinValues.put(2, mRxData.get(6));
            pinValues.put(3, mRxData.get(8));

            return pinValues.get(pinNumber) & 0xff;
        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }

    }

    /**
     * Read the current values of the four GPIO pins.
     * 
     * @param pinValues
     *            (ByteBuffer) - The buffer will contain the pin values (0 = logic low, 1 = logic
     *            high or 0xEE if the pin isn't configured for GPIO operation). The buffer's limit
     *            should be at least 4.
     * @return (int) - 0 If successful. Negative value indicates an error.
     */
    public int getGpPinValue(ByteBuffer pinValues) {

        if (pinValues == null || pinValues.limit() < 4) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        // Setup a buffer with command
        mTxData.clear();
        mTxData.put(clearBuffer);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_GET_GP);

        mRxData.clear();
        mRxData.put(clearBuffer);
        mRxData = mcp2221.sendData(mTxData);

        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {

            pinValues.put(0, (byte) (mRxData.get(2) & 0xff));
            pinValues.put(1, (byte) (mRxData.get(4) & 0xff));
            pinValues.put(2, (byte) (mRxData.get(6) & 0xff));
            pinValues.put(3, (byte) (mRxData.get(8) & 0xff));

            return Mcp2221Constants.ERROR_SUCCESSFUL;
        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }

    }

    /**
     * Set the current value of the specified pin.
     * 
     * @param pinNumber
     *            (byte) - The pin number of the pin to set the value of (0, 1, 2, or 3 are valid
     *            values)
     * @param valueToSet
     *            (byte) - The output value to set on the specified pin (0 = logic low, 1 = logic
     *            high).
     * @return (int) If successful, returns 0. A value less than 0 indicates an error.
     */
    public int setGpPinValue(byte pinNumber, byte valueToSet) {

        // clear the tx buffer
        mTxData.clear();
        mTxData.put(clearBuffer);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_SET_GP);

        switch (pinNumber) {
            case 0:
                // The next byte (index 3) will be the value used to set GP0 output (only if GP0 is
                // set for GPIO output).
                mTxData.put(2, (byte) 1);
                mTxData.put(3, valueToSet);
                break;
            case 1:
                // The next byte (index 7) will be the value used to set GP1 output
                mTxData.put(6, (byte) 1);
                mTxData.put(7, valueToSet);
                break;
            case 2:
                // The next byte (index 11) will be the value used to set GP2 output
                mTxData.put(10, (byte) 1);
                mTxData.put(11, valueToSet);
                break;
            case 3:
                // The next byte (index 15) will be the value used to set GP3’s pin value
                mTxData.put(14, (byte) 1);
                mTxData.put(15, valueToSet);
                break;

            default:
                return Mcp2221Constants.ERROR_INVALID_PARAMETER_2;

        }

        // clear the RX buffer
        mRxData.clear();
        mRxData.put(clearBuffer);

        // send the command
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return Mcp2221Constants.ERROR_SUCCESSFUL;
        } else {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;
        }
    }

    /**
     * Configure all four GPIO outputs with the specified values.
     * 
     * @param valuesToSet
     *            (ByteBuffer) - buffer containing the values that will be set to the outputs
     *            (0=logic low, 1 = logic high). valuesToSet[x] will configure the GPx pin. Only the
     *            first 4 buffer entries will be used.
     * @return (int) - 0 if successful, negative number if there was an error.
     */
    public int setGpPinValue(ByteBuffer valuesToSet) {

        if (valuesToSet == null || valuesToSet.limit() < 4) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        // clear the tx buffer
        mTxData.clear();
        mTxData.put(clearBuffer);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_SET_GP);

        // The next byte (index 3) will be the value used to set GP0 output (only if GP0 is
        // set for GPIO output).
        mTxData.put(2, (byte) 1);
        mTxData.put(3, valuesToSet.get(0));
        // The next byte (index 7) will be the value used to set GP1 output
        mTxData.put(6, (byte) 1);
        mTxData.put(7, valuesToSet.get(1));
        // The next byte (index 11) will be the value used to set GP2 output
        mTxData.put(10, (byte) 1);
        mTxData.put(11, valuesToSet.get(2));
        // The next byte (index 15) will be the value used to set GP3’s pin value
        mTxData.put(14, (byte) 1);
        mTxData.put(15, valuesToSet.get(3));

        // clear the RX buffer
        mRxData.clear();
        mRxData.put(clearBuffer);

        // send the command
        mRxData = mcp2221.sendData(mTxData);
        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return Mcp2221Constants.ERROR_SUCCESSFUL;
        } else {
            return Mcp2221Constants.ERROR_DEV_WRITE_FAILED;

        }

    }

    /******************************************************************
     * ADC functions
     ******************************************************************/

    /**
     * Possible values for the ADC and DAC voltage reference.
     */
    public enum VRefValue {
        /** Vdd used as the voltage reference. */
        VREF_VDD,
        /** Vref = 1.024 V. */
        VREF_1024MV,
        /** Vref = 2.048 V. */
        VREF_2048MV,
        /** Vref = 4.096 V if Vdd > 4.096 V . */
        VREF_4096MV,
        /** Invalid Voltage Reference. */
        INVALID_VREF

    }

    /**
     * Sets the voltage reference value for the ADC.
     * 
     * @param vRefValue
     *            (VRefValue) - the value which will be set
     * @return (int) - 0 if successful, negative value if there was an error
     */
    public final int setAdcVoltageReference(final VRefValue vRefValue) {

        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        switch (vRefValue) {
            case VREF_VDD:
                mMcp2221Config.setAdcVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_VDD));
                break;
            case VREF_1024MV:
                mMcp2221Config
                        .setAdcVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_1024MV));
                break;
            case VREF_2048MV:
                mMcp2221Config
                        .setAdcVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_2048MV));
                break;
            case VREF_4096MV:
                mMcp2221Config
                        .setAdcVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_4096MV));
                break;
            default:
                return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;

        }
        // update the ADC vref value
        int result =
                setSRamSettings(mMcp2221Config, false, false, false, true, false, false, false);
        return result;
    }

    /**
     * Get the Vref value for the ADC.
     * 
     * @return (VRefValue) - the currently set Vref value for the ADC. If the value cannot be read
     *         or it's not one of the predefined values, then "INVALID_VREF" will be returned.
     */
    public final VRefValue getAdcVoltageReference() {
        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        if (getSRamSettings(mMcp2221Config) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            switch (mMcp2221Config.getAdcVoltageReference()) {
                case VREF_VDD:
                    return VRefValue.VREF_VDD;
                case VREF_1024MV:
                    return VRefValue.VREF_1024MV;
                case VREF_2048MV:
                    return VRefValue.VREF_2048MV;
                case VREF_4096MV:
                    return VRefValue.VREF_4096MV;
                default:
                    return VRefValue.INVALID_VREF;
            }
        } else {
            return VRefValue.INVALID_VREF;
        }

    }

    /**
     * Get the ADC data for all 3 channels.
     * 
     * @param adcValues
     *            (IntBuffer) - The ADC pin values. adcValues[x] will contain the value for pin
     *            ADCx. The buffer limit must be at least 3.
     * @return (int) - 0 if successful. Negative value if there was an error.
     */
    public final int getAdcData(final IntBuffer adcValues) {
        if (adcValues.limit() < 3) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        // clear the TX buffer
        mTxData.clear();
        mTxData.put(clearBuffer);

        mTxData.put(0, (byte) Mcp2221Constants.CMD_I2C_SET_CONFIGURATION_GET_STATUS);

        mRxData.clear();
        mRxData.put(clearBuffer);

        mRxData = mcp2221.sendData(mTxData);

        if (mRxData.get(0) == mTxData.get(0) && mRxData.get(1) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            adcValues.put(0, ((mRxData.get(51) << 8) + (mRxData.get(50) & 0xff)) & 0x3ff);
            adcValues.put(1, ((mRxData.get(53) << 8) + (mRxData.get(52) & 0xff)) & 0x3ff);
            adcValues.put(2, ((mRxData.get(55) << 8) + (mRxData.get(54) & 0xff)) & 0x3ff);
            return Mcp2221Constants.ERROR_SUCCESSFUL;

        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }
    }

    /******************************************************************
     * DAC functions
     ******************************************************************/

    /**
     * Sets the voltage reference value for the DAC.
     * 
     * @param vRefValue
     *            (VRefValue) - the value which will be set
     * @return (int) - 0 if successful, negative value if there was an error
     */
    public final int setDacVoltageReference(final VRefValue vRefValue) {

        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        switch (vRefValue) {
            case VREF_VDD:
                mMcp2221Config.setDacVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_VDD));
                break;
            case VREF_1024MV:
                mMcp2221Config
                        .setDacVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_1024MV));
                break;
            case VREF_2048MV:
                mMcp2221Config
                        .setDacVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_2048MV));
                break;
            case VREF_4096MV:
                mMcp2221Config
                        .setDacVoltageReference((byte) (BITMASK_LOAD_NEW_VALUE | VREF_4096MV));
                break;
            default:
                return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;

        }
        // update the DAC vref value
        int result =
                setSRamSettings(mMcp2221Config, false, true, false, false, false, false, false);
        return result;
    }

    /**
     * Get the Vref value for the DAC.
     * 
     * @return (VRefValue) - the currently set Vref value for the DAC. If the value cannot be read
     *         or it's not one of the predefined values, then "INVALID_VREF" will be returned.
     */
    public final VRefValue getDacVoltageReference() {
        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        if (getSRamSettings(mMcp2221Config) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            switch (mMcp2221Config.getDacVoltageReference()) {
                case VREF_VDD:
                    return VRefValue.VREF_VDD;
                case VREF_1024MV:
                    return VRefValue.VREF_1024MV;
                case VREF_2048MV:
                    return VRefValue.VREF_2048MV;
                case VREF_4096MV:
                    return VRefValue.VREF_4096MV;
                default:
                    return VRefValue.INVALID_VREF;
            }
        } else {
            return VRefValue.INVALID_VREF;
        }

    }

    /**
     * Get the DAC value.
     * 
     * @return (int) - DAC value (between 0 and 31). Negative value if an error was encountered.
     */
    public final int getDacValue() {
        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }
        int res = getSRamSettings(mMcp2221Config);
        if (res == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return mMcp2221Config.getDacValue();
        } else {
            return res;
        }
    }

    /**
     * Set the DAC value.
     * 
     * @param dacValue
     *            (byte) - DAC value (between 0 and 31)
     * @return (int) - 0 if successfull. Negative value if an error was encountered.
     */
    public final int setDacValue(byte dacValue) {
        if (dacValue < 0 || dacValue > 31) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }

        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        mMcp2221Config.setDacValue((byte) dacValue);

        // update the DAC value
        int result =
                setSRamSettings(mMcp2221Config, false, false, true, false, false, false, false);
        return result;
    }

    /******************************************************************
     * Clock functions
     ******************************************************************/

    /**
     * Get the clock pin divider value, which ranges between 1 and 7.
     * 
     * @return (int) - If successful, returns value of clock divider (ranges from 1 - 7). A value
     *         less than 0 indicates an error. This value is the exponent in the clock divider
     *         calculation: (2^n) (Ex. Receiving 2 means clock divider is 4, 3 means it is 8, etc).
     *         See device datasheet for more details.
     */
    public final int getClockPinDividerValue() {
        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        // get the current configuration
        if (getSRamSettings(mMcp2221Config) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return mMcp2221Config.getGpClockDividerValue();
        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }
    }

    /**
     * Get the clock pin duty cycle.
     * 
     * @return (int) - If successful, returns value of clock duty cycle (ranges from 0 - 3). A value
     *         less than 0 indicates an error. Notes: 3 = 75%, 2 = 50%, 1 = 25%, 0 = 0%
     */
    public int getClockPinDutyCycle() {
        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        // get the current configuration
        if (getSRamSettings(mMcp2221Config) == Mcp2221Constants.ERROR_SUCCESSFUL) {
            return mMcp2221Config.getGpClockDutyCycle();
        } else {
            return Mcp2221Constants.ERROR_DEV_READ_FAILED;
        }
    }

    /**
     * Set the clock pin divider value and duty cycle.
     * 
     * @param clockDividerValueToSet
     *            (byte) - Value to use for the clock divider.Possible value for clock divider
     *            ranges from 1 to 7 and represent powers of 2. (ex: 3 = 2^3 divider).
     * @param dutyCycleToSet
     *            (byte) - Value indicating the clock duty cycle: 0% = 0, 25% = 1, 50% = 2, 75% = 3
     * @return (int) - 0 if successful. Negative number indicates an error.
     */
    public final int setClockPinConfiguration(final byte clockDividerValueToSet,
            final byte dutyCycleToSet) {

        int minimumValueAccepted = 1;

        if (clockDividerValueToSet < minimumValueAccepted || clockDividerValueToSet > 7) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_1;
        }
        if (dutyCycleToSet < 0 || dutyCycleToSet > 3) {
            return Mcp2221Constants.ERROR_INVALID_PARAMETER_2;
        }

        if (mMcp2221Config == null) {
            mMcp2221Config = new Mcp2221Config();
        }

        mMcp2221Config.setGpClockDividerValue(clockDividerValueToSet);
        mMcp2221Config.setGpClockDutyCycle(dutyCycleToSet);

        return setSRamSettings(mMcp2221Config, true, false, false, false, false, false, false);
    }

    /******************************************************************
     * COM related functions
     ******************************************************************/
    /**
     * Set the COM baud rate.
     * 
     * @param baudRate
     *            (int) - the desired baud rate. Supported values are between 300 and 115200
     * @return (boolean) - true if the baud rate was set, false if an error occured.
     */
    public boolean setBaudRate(int baudRate) {
        return mcp2221.setBaudRate(baudRate);
    }

    /**
     * Get the COM baud rate.
     * 
     * @return (int) - the baud rate currently configured on the mcp2221, negative value indicates
     *         an error.
     */
    public int getBaudRate() {
        return mcp2221.getBaudRate();
    }

    /**
     * Returns true if the receiver has already been started and still runs code (hasn't died yet).
     * Returns false either if the receiver hasn't been started yet or if it has already started and
     * run to completion and died.
     * 
     * @return (boolean) -a boolean indicating the liveness of the Thread
     */
    public boolean isComOpen() {
        return mcp2221.isComOpen();
    }

    /**
     * Starts the thread that handles the COM communication.
     * 
     * @return (boolean) - true if the COM thread was successfully started.
     */
    public boolean openCOM() {
        return mcp2221.openCOM();
    }

    /**
     * Close the COM port.
     */
    public void closeCOM() {
        mcp2221.closeCOM();
    }

    /**
     * Set the handler which will send/receive the CDC messages.
     * 
     * @param handler
     *            (Handler) - Handler to pass CDC messages from the COM thread to the calling
     *            activity
     */
    public void setHandler(Handler handler) {
        mcp2221.setHandler(handler);
    }

    /**
     * Sends data to the MCP2221 Serial Port (COM).
     * 
     * @param data
     *            (String) - the data that will be sent.
     */
    public void sendCdcData(String data) {
        mcp2221.sendCdcData(data);
    }

    /******************************************************************
     * Various functions
     ******************************************************************/

    /**
     * Returns the value of certain bits in a given input byte. ex: to read bits 2-4 from 0x74 -
     * getBits(0x74,3,2) will return 0x5
     * 
     * @param inputByte
     *            (byte) - the byte we want to extract bits from
     * @param numberOfBitsToRead
     *            (int) - the number of bits that will be extracted from the inputByte
     * @param positionToReadFrom
     *            (int) - starting position from which the bits will be read (MSB = 7, LSB = 0)
     * @return (byte) the value of the selected bits from the inputByte
     */
    private byte getBits(byte inputByte, int numberOfBitsToRead, int positionToReadFrom) {
        return (byte) ((inputByte >> positionToReadFrom) & (byte) (Math.pow(2, numberOfBitsToRead) - 1));
    }

}
