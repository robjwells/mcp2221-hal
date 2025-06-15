use embassy_rp::{
    peripherals::UART0,
    uart::{self, Uart},
};

/// UART character echo
///
/// This task just echoes every character received on the UART.
#[embassy_executor::task]
pub(crate) async fn echo(mut driver: Uart<'static, UART0, uart::Async>) -> ! {
    defmt::info!("Writing hello message to Uart...");
    driver
        .write(b"Hello from the UART driver! Entering echo mode.\r\n")
        .await
        .expect("Failed to write to the UART.");

    // Single-character echo.
    let mut buf = [0u8; 1];
    loop {
        if let Err(e) = driver.read(&mut buf).await {
            defmt::error!("Error on read: {:?}", e);
            continue;
        };
        if let Err(e) = driver.write(&buf).await {
            defmt::error!("Error on write: {:?}", e);
            continue;
        }
        // Write NL if character was just a CR.
        if &buf == b"\r" {
            if let Err(e) = driver.write(b"\n").await {
                defmt::error!("Error on write: {:?}", e);
            }
        }
    }
}
