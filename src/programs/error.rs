use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_text::{
    alignment::HorizontalAlignment,
};
use profont::*;
use uc8151::{WIDTH};

use crate::draw;

pub fn draw_error_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_textbox(display, "Program Not Installed.", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 0, 0, (WIDTH), 0);
    let _ = display.update();
}