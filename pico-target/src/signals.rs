use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub(crate) static ADC_SIGNAL: Signal<CriticalSectionRawMutex, f32> = Signal::new();
pub(crate) static PIN_INPUT_SIGNAL: Signal<CriticalSectionRawMutex, Level> = Signal::new();
pub(crate) static I2C_SIGNAL: Signal<CriticalSectionRawMutex, embassy_rp::i2c_slave::Command> =
    Signal::new();
