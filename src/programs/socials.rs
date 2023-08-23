use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};

// Graphics
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_text::{
    alignment::HorizontalAlignment,
};
use profont::*;
use uc8151::WIDTH;

use crate::draw;

pub fn draw_socials_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_image(display, include_bytes!("../../assets/qr.bmp"), 0, 0);
    draw::draw_textbox(display, "Socials", PROFONT_18_POINT, BinaryColor::Off, HorizontalAlignment::Left, 130, 5, (WIDTH - 130), 0);
    draw::draw_textbox(display, "Discord: @lynix.ca\nTelegram: @cyberlynix", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 130, 37, (WIDTH - 130), 0);


    let _ = display.update();
}