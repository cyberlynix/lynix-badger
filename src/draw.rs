// Graphics
use embedded_graphics::{
    image::Image,
    mono_font::{MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics::mono_font::MonoFont;
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

pub fn draw_image<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    data: &[u8],
    x: i32,
    y: i32
) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    let tga: Bmp<BinaryColor> = Bmp::from_slice(data).unwrap();
    let _ = Image::new(&tga, Point::new(x, y)).draw(display);
}

pub fn draw_textbox<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    text: &str,
    font: MonoFont,
    foreground: BinaryColor,
    align: HorizontalAlignment,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    let character_style = MonoTextStyle::new(&font, foreground);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(align)
        //.vertical_alignment(embedded_text::alignment::VerticalAlignment::Top)
        .paragraph_spacing(6)
        .build();

    let bounds = Rectangle::new(Point::new(x, y), Size::new(width, height));
    bounds
        .into_styled(PrimitiveStyle::with_fill(foreground))
        .draw(display)
        .unwrap();

    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
    text_box.draw(display).unwrap();
}

pub fn draw_text<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    text: &str,
    foreground: BinaryColor,
    x: i32,
    y: i32

) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    let text_style = MonoTextStyle::new(&PROFONT_24_POINT, foreground);
    Text::new(text, Point::new(x, y), text_style).draw(display).unwrap();
}