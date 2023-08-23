// Graphics
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle},
};
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_hal::blocking::spi::Write as SpiWrite;
use core::fmt::Write as FmtWrite;
use embedded_text::{
    alignment::HorizontalAlignment,
};

use profont::*;
use uc8151::{HEIGHT, Uc8151, WIDTH};


// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;
use heapless::String;
use crate::{draw, ProgramState};

pub fn draw_menu<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    items: [&str; 6],
    selected_item: usize,
    page: usize
) where
    SPI: SpiWrite<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    draw::draw_image(display, include_bytes!("../../assets/options.bmp"), 0, 0);
    draw::draw_textbox(display, "Programs", PROFONT_24_POINT, BinaryColor::Off, HorizontalAlignment::Left, 42, 3, (WIDTH - 42), 0);

    let items_per_page = 4;
    let start_index = page * items_per_page;

    let mut page_text: String<32> = String::from("[");

    write!(page_text, "{}/{}]", page + 1, (items.len() / items_per_page) + 1).unwrap();
    // Now draw the text with the formatted page number
    draw::draw_text(display, &page_text, BinaryColor::Off, (WIDTH - 50) as i32, (HEIGHT - 5) as i32);

    for (index, item) in items.iter().enumerate().skip(start_index).take(items_per_page) {
        let y_position = 30 + (14 + (index as i32 - start_index as i32) * 20); // Adjust positioning as needed.

        // Highlight the selected item.
        if index == selected_item {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::Off)
                .stroke_width(3)
                .fill_color(BinaryColor::Off)
                .build();

            Rectangle::new(Point::new(10, y_position - 9), Size::new(10, 10))
                .into_styled(style)
                .draw(display).unwrap();
        } else {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::Off)
                .stroke_width(3)
                .fill_color(BinaryColor::On)
                .build();

            Rectangle::new(Point::new(10, y_position - 9), Size::new(10, 10))
                .into_styled(style)
                .draw(display).unwrap();
        }

        draw::draw_text(display, item, BinaryColor::Off, 27, y_position);
    }
}

pub fn handle_menu_program<SPI, CS, DC, BUSY, RESET>(
    display: &mut Uc8151<SPI, CS, DC, BUSY, RESET>,
    items: [&str; 6],
    current_item: &mut usize,
    btn_up_pressed: bool,
    btn_down_pressed: bool,
) where
    SPI: SpiWrite<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    RESET: OutputPin,
{
    // Handle navigation logic
    if btn_up_pressed {
        if *current_item > 0 {
            *current_item -= 1;
        }
    }

    if btn_down_pressed {
        // Replace 4 with the length of your menu items list
        if *current_item < items.len() - 1 {
            *current_item += 1;
        }
    }

    let current_page = *current_item / 4;

    // Clear display before updating
    let _ = display.clear(BinaryColor::On);

    // Draw the menu for the current page
    draw_menu(display, items, *current_item, current_page);

    // Update the display if any button was pressed
    if btn_up_pressed || btn_down_pressed {
        let _ = display.update();
    }
}

pub fn launch_selected_program(
    current_item: usize,
    btn_a_pressed: bool,
) -> Option<ProgramState> {
    if btn_a_pressed {
        // let items = ["Lynix Badge", "CCNB", "Socials + QR", "Device Info", "Blinky", "Settings"];

        match current_item {
            // Handle each case and return the corresponding program state
            0 => Some(ProgramState::Lynix),
            1 => Some(ProgramState::Ccnb),
            2 => Some(ProgramState::Socials),
            3 => Some(ProgramState::Info),
            4 => Some(ProgramState::Blinky),
            _ => Some(ProgramState::NotFound)
        }
    } else {
        None // Return None if no program is launched
    }
}