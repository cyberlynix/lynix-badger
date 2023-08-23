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

pub fn draw_main_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_image(display, include_bytes!("../../assets/lynix.bmp"), 0, 0);
    draw::draw_textbox(display, "Lynix", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 140, 0, (WIDTH - 130), 0);
    draw::draw_textbox(display, "Cybersecurity Student\nCanadian\n[lynix.ca]", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 140, 32, (WIDTH - 130), 0);


    let _ = display.update();
}