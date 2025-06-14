use embassy_rp::gpio::{Input, Output};

use crate::signals::PIN_INPUT_SIGNAL;

#[embassy_executor::task]
pub(crate) async fn input_pin_to_led(mut input_pin: Input<'static>, mut led: Output<'static>) -> ! {
    loop {
        let level = input_pin.get_level();
        PIN_INPUT_SIGNAL.signal(level);
        led.set_level(level);
        input_pin.wait_for_any_edge().await;
    }
}
