use embassy_rp::i2c_slave::{self, I2cSlave};
use embassy_rp::peripherals::I2C0;

use crate::signals::I2C_SIGNAL;

static OUR_DATA: [u8; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
    155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
    193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
    212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
    231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
    250, 251, 252, 253, 254, 255,
];

#[embassy_executor::task]
pub(crate) async fn target(mut driver: I2cSlave<'static, I2C0>) -> ! {
    defmt::info!("I2C target task started");
    let mut receive_buffer = [0_u8; 512];
    loop {
        let res = driver.listen(&mut receive_buffer).await;
        if let Err(e) = res {
            defmt::error!("Error from I2C target driver: {:?}", e);
        } else if let Ok(cmd) = res {
            I2C_SIGNAL.signal(cmd);
            match cmd {
                i2c_slave::Command::GeneralCall(n) => general_call(n, &receive_buffer),
                i2c_slave::Command::Read => read(&mut driver).await,
                i2c_slave::Command::WriteRead(n) => {
                    write_read(&mut driver, n, &receive_buffer).await
                }
                i2c_slave::Command::Write(n) => write(n, &receive_buffer),
            }
        };
    }
}

fn general_call(n: usize, receive_buffer: &[u8]) {
    defmt::info!(
        "General call: Received {} bytes\t{=[u8]:#X}",
        n,
        receive_buffer[..n]
    );
}

async fn read(driver: &mut I2cSlave<'static, I2C0>) {
    defmt::info!("Read: Responding with the large data buffer repeatedly.");
    use i2c_slave::ReadStatus::*;
    loop {
        // Send from our big data buffer.
        match driver.respond_to_read(&OUR_DATA).await {
            // Loop and send again.
            Ok(NeedMoreBytes) => continue,
            // Done, either exactly or with bytes left in the transmit buffer.
            // Note the number of leftover bytes is _not_ the number of bytes not sent
            // from your own buffer; seems to be some internal embassy buffer.
            Ok(Done) | Ok(LeftoverBytes(_)) => break,
            Err(e) => {
                defmt::error!("I2C Read error: {:?}", e);
                break;
            }
        }
    }
    defmt::info!("Read: done");
}

async fn write_read(driver: &mut I2cSlave<'static, I2C0>, n: usize, receive_buffer: &[u8]) {
    defmt::info!(
        "Write-Read: Received {:?} bytes\t{=[u8]:#X}",
        n,
        receive_buffer[..n]
    );
    let start = receive_buffer[0] as usize;
    let length = (receive_buffer[1] as usize).clamp(0, 256 - start);
    defmt::info!(
        "Responding with {} bytes of our data starting at {}: {:?}",
        length,
        start,
        &OUR_DATA[start..start + length],
    );
    if let Err(e) = driver
        .respond_to_read(&OUR_DATA[start..start + length])
        .await
    {
        defmt::error!("Error responding to Read: {:?}", e);
    }
}

fn write(n: usize, receive_buffer: &[u8]) {
    defmt::info!(
        "Write: Received {:?} bytes\t{=[u8]:#X}",
        n,
        receive_buffer[..n]
    )
}
