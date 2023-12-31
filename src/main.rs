//! # Pico USB Serial (with Interrupts) Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the USB interrupt.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to upercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

mod draw;
mod usb;
mod libs;
mod programs;

// The macro for our start-up function
use rp_pico::entry;

// The macro for marking our interrupt functions

// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;

// Pull in any important traits
use rp_pico::hal::prelude::*;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;
use hal::timer::Timer;
use cortex_m::prelude::_embedded_hal_timer_CountDown;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Serial Device Driver (shared with the interrupt).
static mut USB_SERIAL: Option<SerialPort<hal::usb::UsbBus>> = None;

// Display
use fugit::RateExtU32;
use fugit::ExtU32;

// Graphics
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::blocking::spi::Write;

// Programs
use crate::programs::blinky::{draw_blinky_screen, handle_blinky_program};
use crate::programs::ccnb::draw_ccnb_screen;
use crate::programs::error::draw_error_screen;
use crate::programs::info::draw_info_screen;
use crate::programs::main::draw_main_screen;
use crate::programs::menu;
use crate::programs::socials::draw_socials_screen;

use panic_halt as _;

pub enum ProgramState {
    Menu,
    Lynix,
    Ccnb,
    Socials,
    Info,
    Blinky,
    NotFound,
}

enum ButtonState {
    Pressed,
    Released,
}

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then blinks the LED in an
/// infinite loop.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    // TODO: Run at a lower clock speed to save battery.
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the pins for the e-ink display
    let _spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();
    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);
    let dc = pins.gpio20.into_push_pull_output();
    let cs = pins.gpio17.into_push_pull_output();
    let busy = pins.gpio26.into_pull_up_input();
    let reset = pins.gpio21.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        RateExtU32::MHz(1),
        &embedded_hal::spi::MODE_0,
    );

    // Setup Buttons
    let mut btn_up = pins.gpio15.into_pull_down_input();
    let mut btn_down = pins.gpio11.into_pull_down_input();

    let mut btn_a = pins.gpio12.into_pull_down_input();
    let mut btn_b = pins.gpio13.into_pull_down_input();
    //let mut btn_c = pins.gpio14.into_pull_down_input();

    // Get all the basic peripherals, and init clocks/timers
    // Enable 3.3V power or you won't see anything
    let mut power = pins.gpio10.into_push_pull_output();
    power.set_high().unwrap();

    // Set the LED to be an output
    let mut led_pin = pins.led.into_push_pull_output();

    // Create new Display object
    let mut display = uc8151::Uc8151::new(spi, cs, dc, busy, reset);
    let mut count_down = timer.count_down();

    // Reset the display
    display.disable();

    count_down.start(10u32.millis());
    let _ = nb::block!(count_down.wait());
    display.enable();
    count_down.start(10u32.millis());
    let _ = nb::block!(count_down.wait());
    // Wait for the screen to finish reset
    while display.is_busy() {}

    let _ = display.setup(&mut delay, uc8151::LUT::Fast);

    let _ = display.clear(BinaryColor::On);

    //draw_main_screen(&mut display);

    let _ = display.update();

    let items = ["Lynix Badge", "CCNB", "Socials + QR", "Device Info", "Blinky", "DEFCON Furs", "Cryptography", "Settings"];

    // Draw menu items.
    let mut selected_item = 0;

    //crate::programs::menu::draw_menu(&mut display, items, selected_item, 0);

    //let _ = display.update();

    // Current Program
    let mut initial_screen_drawn = false;
    let mut current_program = ProgramState::Lynix;

    // Countdown
    let mut counter = 0;

    loop {
        count_down.start(1u32.secs());

        // Read Buttons
        let btn_up_pressed = btn_up.is_high().unwrap();
        let btn_down_pressed = btn_down.is_high().unwrap();
        let btn_a_pressed = btn_a.is_high().unwrap();
        let btn_b_pressed = btn_b.is_high().unwrap();
        //let btn_c_pressed = btn_b.is_high().unwrap();

        if btn_b_pressed {
            initial_screen_drawn = false;
            current_program = ProgramState::Menu;
            let _ = display.clear(BinaryColor::On);
        }

        match current_program {
            ProgramState::Menu => {
                count_down.start(1u32.millis());

                // Draw Screen
                if !initial_screen_drawn {
                    let _ = display.setup(&mut delay, uc8151::LUT::Ultrafast);

                    // Make sure screen is cleared
                    let _ = display.clear(BinaryColor::On);
                    let _ = display.update();
                    let _ = display.update();

                    menu::draw_menu(&mut display, items, selected_item, 0);
                    let _ = display.update();
                    let _ = display.update();
                    initial_screen_drawn = true;
                }

                menu::handle_menu_program(
                    &mut display,
                    items,
                    &mut selected_item,
                    btn_up_pressed,
                    btn_down_pressed,
                );

                let new_program = menu::launch_selected_program(
                    selected_item,
                    btn_a_pressed,
                );

                if let Some(program_state) = new_program {
                    let _ = display.setup(&mut delay, uc8151::LUT::Fast);
                    current_program = program_state;
                    let _ = display.clear(BinaryColor::On);
                    initial_screen_drawn = false;
                }
            }
            ProgramState::Blinky => {
                // Draw Screen
                if !initial_screen_drawn {
                    draw_blinky_screen(&mut display);
                    initial_screen_drawn = true;
                }

                // Handle Blinky program logic
                handle_blinky_program(&mut led_pin, &mut delay, false);
            }
            ProgramState::Lynix => {
                // Draw Screen
                if !initial_screen_drawn {
                    draw_main_screen(&mut display);
                    initial_screen_drawn = true;
                }
                counter += 1;

                if counter == 10 {
                    let _ = display.clear(BinaryColor::On);
                    draw_socials_screen(&mut display);
                    led_pin.set_high().unwrap();
                }

                if counter == 20 {
                    counter = 0;
                    initial_screen_drawn = false;
                    let _ = display.clear(BinaryColor::On);
                    led_pin.set_low().unwrap();
                }
            }
            ProgramState::Ccnb => {
                // Draw Screen
                if !initial_screen_drawn {
                    draw_ccnb_screen(&mut display);
                    initial_screen_drawn = true;
                }

                counter += 1;

                if counter == 10 {
                    let _ = display.clear(BinaryColor::On);
                    draw_socials_screen(&mut display);
                    led_pin.set_high().unwrap();
                }

                if counter == 20 {
                    counter = 0;
                    initial_screen_drawn = false;
                    let _ = display.clear(BinaryColor::On);
                    led_pin.set_low().unwrap();
                }
            }
            ProgramState::Socials => {
                // Draw Screen
                if !initial_screen_drawn {
                    draw_socials_screen(&mut display);
                    initial_screen_drawn = true;
                }
            }
            ProgramState::Info => {
                // Draw Screen
                if !initial_screen_drawn {
                    draw_info_screen(&mut display);
                    initial_screen_drawn = true;
                }
            }
            ProgramState::NotFound => {
                if !initial_screen_drawn {
                    draw_error_screen(&mut display);
                    initial_screen_drawn = true;
                }

                delay.delay_ms(2000);

                current_program = ProgramState::Menu;
                let _ = display.clear(BinaryColor::On);
                initial_screen_drawn = false;
            }
            // Handle programs that are not found
        }
        let _ = nb::block!(count_down.wait());
    }
}