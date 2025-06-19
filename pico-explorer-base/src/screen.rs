use embassy_rp::{
    Peripheral,
    dma::Channel,
    gpio::{Level, Output},
    peripherals::{PIN_16, PIN_17, PIN_18, PIN_19, SPI0},
    spi::{Async, Config as SpiConfig, Spi},
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Drawable, Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use mipidsi::{Display, NoResetPin, interface::SpiInterface, models::ST7789};
use static_cell::ConstStaticCell;

/// Buffer for the screen pixel data, used (only!) by the ST7789 driver.
/// Creating it as a static allows us to keep the per-task arena size low.
static IMAGE_BUFFER: ConstStaticCell<[u8; 8_192]> = ConstStaticCell::new([0_u8; 8_192]);

pub type ExplorerDisplay = Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    ST7789,
    NoResetPin,
>;

/// Set up the display driver for the ST7789 on the Pico Explorer Base.
///
/// Note that the LCD `DC` pin is listed as `SPI MISO` on the back of the PCB.
pub fn create_display(
    spi_peripheral: SPI0,
    mosi: PIN_19,
    clk: PIN_18,
    cs: PIN_17,
    dc: PIN_16,
    dma: impl Peripheral<P = impl Channel> + 'static,
) -> ExplorerDisplay {
    use mipidsi::options::ColorInversion::Inverted;

    let lcd_cs = Output::new(cs, Level::High);
    let lcd_dc = Output::new(dc, Level::Low);

    let mut spi_config: SpiConfig = Default::default();
    spi_config.frequency = 62_500_000; // 62.5 MHz (taken from Pimoroni driver)
    let spi_bus = Spi::new_txonly(spi_peripheral, clk, mosi, dma, spi_config);
    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, lcd_cs).unwrap();

    let display_interface = SpiInterface::new(spi_device, lcd_dc, IMAGE_BUFFER.take());
    let mut display = mipidsi::Builder::new(ST7789, display_interface)
        .invert_colors(Inverted)
        .display_size(240, 240)
        .init(&mut embassy_time::Delay)
        .unwrap();
    blank_screen(&mut display);
    display
}

pub fn blank_screen(display: &mut ExplorerDisplay) {
    Rectangle::new(Point::new(0, 0), Size::new(240, 240))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLACK)
                .build(),
        )
        .draw(display)
        .unwrap();
}
