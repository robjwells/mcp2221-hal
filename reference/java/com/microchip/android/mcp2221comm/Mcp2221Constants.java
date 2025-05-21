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

public final class Mcp2221Constants {

    // Miscellaneous
    public static final int DEFAULT_VID = 0x04D8;
    public static final int DEFAULT_PID = 0x00DD;

    // ====================================================
    // SMBUS
    public static final int I2CM_SM_IDLE = 0x00;
    public static final int I2CM_SM_START = 0x10;
    public static final int I2CM_SM_START_ACK = 0x11;
    public static final int I2CM_SM_START_TOUT = 0x12;
    public static final int I2CM_SM_REPSTART = 0x15;
    public static final int I2CM_SM_REPSTART_ACK = 0x16;
    public static final int I2CM_SM_REPSTART_TOUT = 0x17;
    public static final int I2CM_SM_WRADDRL = 0x20;
    public static final int I2CM_SM_WRADDRL_WAITSEND = 0x21;
    public static final int I2CM_SM_WRADDRL_ACK = 0x22;
    public static final int I2CM_SM_WRADDRL_TOUT = 0x23;
    public static final int I2CM_SM_WRADDRL_NACK_STOP_PEND = 0x24;
    public static final int I2CM_SM_WRADDRL_NACK_STOP = 0x25;
    public static final int I2CM_SM_WRADDRH = 0x30;
    public static final int I2CM_SM_WRADDRH_WAITSEND = 0x31;
    public static final int I2CM_SM_WRADDRH_ACK = 0x32;
    public static final int I2CM_SM_WRADDRH_TOUT = 0x33;
    public static final int I2CM_SM_WRITEDATA = 0x40;
    public static final int I2CM_SM_WRITEDATA_WAITSEND = 0x41;
    public static final int I2CM_SM_WRITEDATA_ACK = 0x42;
    public static final int I2CM_SM_WRITEDATA_WAIT = 0x43;
    public static final int I2CM_SM_WRITEDATA_TOUT = 0x44;
    public static final int I2CM_SM_WRITEDATA_END_NOSTOP = 0x45;
    public static final int I2CM_SM_READDATA = 0x50;
    public static final int I2CM_SM_READDATA_RCEN = 0x51;
    public static final int I2CM_SM_READDATA_TOUT = 0x52;
    public static final int I2CM_SM_READDATA_ACK = 0x53;
    public static final int I2CM_SM_READDATA_WAIT = 0x54;
    public static final int I2CM_SM_READDATA_WAITGET = 0x55;
    public static final int I2CM_SM_STOP = 0x60;
    public static final int I2CM_SM_STOP_WAIT = 0x61;
    public static final int I2CM_SM_STOP_TOUT = 0x62;

    // USB command codes
    public static final byte USB_CMD_STATUS = 0x10;
    // all the sub-commands below are transported using the STATUS command

    // ====================================================
    // Sub-commands;
    public static final int SUBCMD_RESET_B1 = 0xAB;
    public static final int SUBCMD_RESET_B2 = 0xCD;
    public static final byte SUBCMD_CANCEL_TRANSFER = 0x10;
    public static final byte SUBCMD_I2C_SET_TRANSFER_SPEED = 0x20;
    public static final byte SUBCMD_I2C_SET_ADDRESS = 0x30;

    // the below command ends without sending STOP condition
    // this is useful for supporting SMB Block reads/writes
    public static final int CMD_I2C_WRITEDATA_7BITS_NOSTOP = 0x94;
    public static final int I2C_CMD_WRDATA10 = 0xA0;
    public static final int I2C_CMD_RDDATA10 = 0xA1;
    public static final int I2C_CMD_RSTART_WRDATA10 = 0xA2;
    public static final int I2C_CMD_RSTART_RDDATA10 = 0xA3;
    // the below command ends without sending STOP condition;
    // this is useful for supporting SMB Block reads/writes;
    public static final int I2C_CMD_WRDATA10_NOSTOP = 0xA4;

    // ====================================================
    // Commands;;
    public static final int CMD_IDLE = I2CM_SM_IDLE;
    public static final int CMD_SRAM_READ = 0x61;
    public static final int CMD_SRAM_WRITE = 0x60;
    public static final int CMD_GET_GP = 0x51;
    public static final int CMD_SET_GP = 0x50;
    public static final int CMD_GET_FLASH = 0xB0;
    public static final int CMD_SET_FLASH = 0xB1;
    public static final int CMD_I2C_TRANSER_DATA = 0;
    public static final int CMD_USB_SET_KEY_PARAM = 0x30;
    public static final int CMD_INT_EVNT_CNT_GET = 0x12;
    public static final int CMD_ENTER_ACCESS_PASSWORD = 0xB2;
    public static final int CMD_RESET_SEQUENCE_VALUE_1 = 0x70;
    public static final int CMD_RESET_SEQUENCE_VALUE_2 = 0xAB;
    public static final int CMD_RESET_SEQUENCE_VALUE_3 = 0xCD;
    public static final int CMD_RESET_SEQUENCE_VALUE_4 = 0xEF;
    // I2C
    public static final int CMD_I2C_SET_CONFIGURATION_GET_STATUS = 0x10;
    public static final int CMD_I2C_FORCESTOP = 0x80;
    public static final int CMD_I2C_WRITEDATA_7BITS = 0x90;
    public static final int CMD_I2C_READDATA_7BITS = 0x91;
    public static final int CMD_I2C_RESTARTWRITE_7BITS = 0x92;
    public static final int CMD_I2C_RESTARTREAD_7BITS = 0x93;
    public static final int CMD_I2C_WRDATA7_NOSTOP = 0x94;
    public static final int CMD_I2C_READDATA = 0x40;
    public static final int USB_CMD_I2CM_WRDATA7 = CMD_I2C_WRITEDATA_7BITS;
    public static final int USB_CMD_I2CM_RDDATA7 = CMD_I2C_READDATA_7BITS;
    public static final int USB_CMD_I2CM_WRDATA10 = I2C_CMD_WRDATA10;
    public static final int USB_CMD_I2CM_RDDATA10 = I2C_CMD_RDDATA10;

    // ====================================================
    // USB responses
    public static final int USB_RESP_SUCCESS = 0x00;
    public static final int USB_ERROR_BUSY = 0x01;
    public static final int USB_RESP_CANCEL_XFER_OK = SUBCMD_CANCEL_TRANSFER;
    public static final int USB_RESP_CANCEL_XFER_ERR = SUBCMD_CANCEL_TRANSFER + 1;
    public static final int USB_RESP_SETXFERSPEED_OK = SUBCMD_I2C_SET_TRANSFER_SPEED;
    public static final int USB_RESP_SETXFERSPEED_ERR = SUBCMD_I2C_SET_TRANSFER_SPEED + 1;
    public static final int USB_RESP_SETADDRESS_OK = SUBCMD_I2C_SET_ADDRESS;
    public static final int USB_RESP_SETADDRESS_ERR = SUBCMD_I2C_SET_ADDRESS + 1;
    public static final int USB_RESP_I2CM_GET_READDATA_ERR = 0x41;
    public static final int MAX_RETRY_COUNT = 50;

    // ====================================================
    // Error codes;
    public static final int ERROR_SUCCESSFUL = 0;
    public static final int ERROR_DEV_WRITE_FAILED = -4;
    public static final int ERROR_DEV_READ_FAILED = -5;
    public static final int ERROR_I2C_SLAVE_DATA_NACK = -11;
    public static final int ERROR_WRONG_PEC = -12;
    public static final int ERROR_INVALID_DATA_LEN = -16;
    public static final int ERROR_TIMEOUT = -18;
    public static final int ERROR_I2C_SEND_ERR = -19;
    public static final int ERROR_I2C_SETSPEED = -21;
    public static final int ERROR_I2C_STATUS = -22;
    public static final int ERROR_I2C_ADDRNACK = -23;
    public static final int ERROR_I2C_READ001 = -24;
    public static final int ERROR_I2C_READ002 = -25;
    public static final int ERROR_I2C_READ003 = -26;
    public static final int ERROR_I2C_READ004 = -27;

    public static final int I2C_STATE_IDLE = 0x11;
    public static final int I2C_STATE_TRANSFER_MARKED_FOR_CANCELLATION = 0x10;

    // DLL Parameter errors - Start at -200;
    public static final int ERROR_INVALID_PARAMETER_1 = -201;
    public static final int ERROR_INVALID_PARAMETER_2 = -202;
    public static final int ERROR_INVALID_PARAMETER_3 = -203;

    /**
     * private constructor. Nothing is initialized here.
     */
    private Mcp2221Constants() {

    }

}
