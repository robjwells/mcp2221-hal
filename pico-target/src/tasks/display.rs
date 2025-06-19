use core::fmt::Write;

use embassy_rp::gpio::Level;
use embassy_time::Timer;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, Drawable, Point, RgbColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_text::TextBox;
use heapless::String;

use pico_explorer_base::screen::ExplorerDisplay;
use crate::signals::{ADC_SIGNAL, I2C_SIGNAL, PIN_INPUT_SIGNAL};

/// Show Pico target status on the Explorer Base display
///
/// This task updates the display with the current values of:
///
/// - Analog input voltage
/// - Digital input level
/// - Most recent I2C transfer
#[embassy_executor::task]
pub(crate) async fn status(mut display: ExplorerDisplay) -> ! {
    let black_fill = PrimitiveStyle::with_fill(Rgb565::BLACK);
    let style = MonoTextStyleBuilder::new()
        .background_color(Rgb565::BLACK)
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();

    let adc_label_pos = Point::new(20, 30);
    let pin_label_pos = Point::new(20, 50);

    let i2c_label_pos = Point::new(20, 80);
    let i2c_box_bounds = Rectangle::with_corners(Point::new(20, 100), Point::new(220, 220));

    let adc_label = Text::new("ADC V: ", adc_label_pos, style);
    let adc_value_pos = Point::new(
        adc_label.bounding_box().bottom_right().unwrap().x,
        adc_label_pos.y,
    );
    adc_label.draw(&mut display).unwrap();

    let pin_label = Text::new("Pin in: ", pin_label_pos, style);
    let pin_value_pos = Point::new(
        pin_label.bounding_box().bottom_right().unwrap().x,
        pin_label_pos.y,
    );
    pin_label.draw(&mut display).unwrap();

    Text::new("I2C last xfer:", i2c_label_pos, style)
        .draw(&mut display)
        .unwrap();

    let mut adc_string: String<8> = String::new();
    let mut i2c_string: String<128> = String::new();

    loop {
        Timer::after_millis(50).await;
        if !(ADC_SIGNAL.signaled() || PIN_INPUT_SIGNAL.signaled()) {
            continue;
        }

        if let Some(reading) = ADC_SIGNAL.try_take() {
            adc_string.clear();
            write!(&mut adc_string, "{:.03}", reading)
                .expect("Failed to write ADC voltage to adc_string.");
            Text::new(&adc_string, adc_value_pos, style)
                .draw(&mut display)
                .unwrap();
        }

        if let Some(new_level) = PIN_INPUT_SIGNAL.try_take() {
            match new_level {
                Level::Low => Text::new("LOW ", pin_value_pos, style),
                Level::High => Text::new("HIGH", pin_value_pos, style),
            }
            .draw(&mut display)
            .unwrap();
        }

        if let Some(new_command) = I2C_SIGNAL.try_take() {
            i2c_string.clear();
            write!(&mut i2c_string, "{:#?}", new_command)
                .expect("Failed to format I2C command string");
            i2c_box_bounds
                .draw_styled(&black_fill, &mut display)
                .unwrap();
            TextBox::new(&i2c_string, i2c_box_bounds, style)
                .draw(&mut display)
                .unwrap();
        }
    }
}
