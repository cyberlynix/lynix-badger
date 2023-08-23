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
pub fn draw_ccnb_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_image(display, include_bytes!("../../assets/anthony2.bmp"), 0, 0);
    draw::draw_textbox(display, "Anthony", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 140, 0, (WIDTH - 130), 0);
    draw::draw_textbox(display, "Programme: Cybersécurité\nBonne Rentrée!", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 140, 32, (WIDTH - 130), 0);
    draw::draw_image(display, include_bytes!("../../assets/lock.bmp"), 140, 100);
    draw::draw_image(display, include_bytes!("../../assets/isc.bmp"), 174, 100);
    draw::draw_image(display, include_bytes!("../../assets/dcf.bmp"), 235, 100);
    let _ = display.update();
}