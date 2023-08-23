// Graphics
use embedded_graphics::{
    image::Image,
    mono_font::{MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::text::{Text};
use embedded_hal::blocking::spi::Write;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

use profont::*;
use tinybmp::Bmp;
use uc8151::{Uc8151, WIDTH};


// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;
use crate::draw;

pub fn draw_menu<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    items: [&str; 3],
    selected_item: usize
) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    draw::draw_image(display, include_bytes!("../assets/options.bmp"), 0, 0);
    draw::draw_textbox(display, "Programs", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 42, 3, (WIDTH - 42), 0);

    for (index, item) in items.iter().enumerate() {
        let y_position = 30 + (14 + (index as i32) * 20); // Adjust positioning as needed.

        // Highlight the selected item.
        if index == selected_item {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::Off)
                .stroke_width(3)
                .fill_color(BinaryColor::Off)
                .build();

            Rectangle::new(Point::new(0, y_position - 9), Size::new(10, 10))
                .into_styled(style)
                .draw(display).unwrap();
        } else {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::Off)
                .stroke_width(3)
                .fill_color(BinaryColor::On)
                .build();

            Rectangle::new(Point::new(0, y_position - 9), Size::new(10, 10))
                .into_styled(style)
                .draw(display).unwrap();
        }

        draw::draw_text(display, item, BinaryColor::Off, 17, y_position);
    }
}