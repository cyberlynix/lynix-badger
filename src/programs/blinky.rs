use cortex_m::delay::Delay;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};

// Graphics
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::blocking::delay::DelayMs;
use embedded_text::{
    alignment::HorizontalAlignment,
};
use profont::*;
use rp2040_hal::gpio::{Pin, PushPullOutput};
use rp2040_hal::gpio::bank0::{Gpio25};
use uc8151::{WIDTH};

use crate::draw;

pub fn draw_blinky_screen<SPI, CS, DC, BUSY, RESET>(display: &mut uc8151::Uc8151<SPI, CS, DC, BUSY, RESET>) where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Draw Initial Screen
    draw::draw_image(display, include_bytes!("../../assets/options.bmp"), 0, 0);
    draw::draw_textbox(display, "Blinky Test", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 42, 3, (WIDTH - 42), 0);
    draw::draw_textbox(display, "led go blink.", PROFONT_14_POINT, BinaryColor::Off, HorizontalAlignment::Left, 0, 38, (WIDTH), 0);
    let _ = display.update();
}

pub fn handle_blinky_program(mut led_pin: &mut Pin<Gpio25, PushPullOutput>, mut delay: &mut Delay, home: bool) {
    led_pin.set_high().unwrap();
    delay.delay_ms(250);

    led_pin.set_low().unwrap();
    delay.delay_ms(250);
}