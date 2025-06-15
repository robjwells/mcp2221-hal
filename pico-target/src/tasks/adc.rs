use embassy_rp::adc::{self, Adc, Channel};

use crate::signals::ADC_SIGNAL;

/// Analog input voltage monitor.
///
/// This task monitors the voltage on the given pin and updates the static signal
/// with the current value every 500ms.
#[embassy_executor::task]
pub(crate) async fn monitor(mut adc: Adc<'static, adc::Async>, mut pin: Channel<'static>) -> ! {
    const ADC_DIVIDER: f32 = 4_095.0 / 3.3;
    loop {
        embassy_time::Timer::after_millis(500).await;
        match adc.read(&mut pin).await {
            Ok(0) => {}
            Ok(reading) => {
                let read_voltage = f32::from(reading) / ADC_DIVIDER;
                ADC_SIGNAL.signal(read_voltage);
            }
            Err(e) => {
                defmt::error!("Error reading from ADC: {:?}", e);
            }
        }
    }
}
