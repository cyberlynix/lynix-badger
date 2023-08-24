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

// TODO: Used for the first day of college (REMOVE AFTER FIRST WEEk)
pub fn draw_info_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_image(display, include_bytes!("../../assets/options.bmp"), 0, 0);
    draw::draw_textbox(display, "Device Info", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 42, 3, (WIDTH - 42), 0);
    draw::draw_textbox(display, "FW Version: v2.0.7\nSerial #: FREAK-4921.8222023\nLynix E-Ink Badge", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 0, 38, (WIDTH), 0);
    let _ = display.update();
}