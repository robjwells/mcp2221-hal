#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{self, Adc};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Output, Pull};
use embassy_rp::i2c;
use embassy_rp::i2c_slave::{self, I2cSlave};
use embassy_rp::peripherals::{I2C0, UART0};
use embassy_rp::uart;
use {defmt_rtt as _, panic_probe as _};

mod explorer;
mod signals;
mod tasks;

use explorer::screen::create_pico_explorer_base_display;

bind_interrupts!(struct UartInterupts {
    UART0_IRQ => uart::InterruptHandler<UART0>;
});

bind_interrupts!(struct AdcInterrupts {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

bind_interrupts!(struct I2cInterrupts {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // I2C target initialisation.
    let t_sda = p.PIN_20;
    let t_scl = p.PIN_21;
    let mut target_config = i2c_slave::Config::default();
    target_config.general_call = true;
    target_config.addr = 0x26;
    let target_driver = I2cSlave::new(p.I2C0, t_scl, t_sda, I2cInterrupts, target_config);

    // UART initialisation.
    let uart = uart::Uart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        UartInterupts,
        p.DMA_CH1,
        p.DMA_CH2,
        Default::default(),
    );

    // Pin 2 monitor initialisation.
    let gp2_input = Input::new(p.PIN_2, Pull::None);
    let led = Output::new(p.PIN_25, gp2_input.get_level());

    // ADC monitor initialisation.
    let adc = Adc::new(p.ADC, AdcInterrupts, Default::default());
    let analog_pin = adc::Channel::new_pin(p.PIN_26, Pull::Down);

    let display = create_pico_explorer_base_display(
        p.SPI0, p.PIN_19, p.PIN_18, p.PIN_17, p.PIN_16, p.DMA_CH0,
    );

    defmt::info!("Spawning I2C target task.");
    _spawner
        .spawn(tasks::i2c::target(target_driver))
        .expect("Failed to spawn I2C target task.");

    defmt::info!("Spawning UART task.");
    _spawner
        .spawn(tasks::uart::echo(uart))
        .expect("Failed to spawn UART task");

    defmt::info!("Spawning pin 2-to-LED task.");
    _spawner
        .spawn(tasks::pin::input_pin_to_led(gp2_input, led))
        .expect("Failed to spawn pin 2 task");

    defmt::info!("Spawning ADC pin monitor task.");
    _spawner
        .spawn(tasks::adc::monitor(adc, analog_pin))
        .expect("Failed to spawn ADC pin monitor task.");

    defmt::info!("Spawning display task.");
    _spawner
        .spawn(tasks::display::status(display))
        .expect("Failed to spawn display task.");
}
