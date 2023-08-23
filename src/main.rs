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
mod screens;
mod libs;
mod menu;
mod programs;

use cortex_m::delay::Delay;
// The macro for our start-up function
use rp_pico::entry;

// The macro for marking our interrupt functions
use rp_pico::hal::pac::interrupt;

// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

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
use defmt::export::char;

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
use uc8151::{Uc8151, WIDTH};
use fugit::RateExtU32;
use fugit::ExtU32;

// Graphics
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::primitives::{Circle, PrimitiveStyleBuilder};
use embedded_graphics::text::{Text, TextStyle, TextStyleBuilder};
use embedded_hal::blocking::spi::Write;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use generic_array::GenericArray;
use profont::*;
use rp2040_hal::gpio::Pin;

use tinybmp::Bmp;
use crate::menu::draw_menu;
use crate::programs::blinky::{draw_blinky_screen, handle_blinky_program};
use crate::screens::ccnb::draw_ccnb_screen;
use crate::screens::info::draw_info_screen;
use crate::screens::main::draw_main_screen;
use crate::screens::socials::draw_socials_screen;

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

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(usb_bus);
    }

    // Grab a reference to the USB Bus allocator. We are promising to the
    // compiler not to take mutable access to this global variable whilst this
    // reference exists!
    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    // Set up the USB Communications Class Device driver
    let serial = SerialPort::new(bus_ref);
    unsafe {
        USB_SERIAL = Some(serial);
    }

    // Create a USB device with a fake VID and PID
    let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x11dd))
        .manufacturer("Lynix Security")
        .product("Lynix E-Ink Badge")
        .serial_number("FREAK-4921.8222023")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();
    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_DEVICE = Some(usb_dev);
    }

    // Enable the USB interrupt
    unsafe {
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    };

    // No more USB code after this point in main! We can do anything we want in
    // here since USB is handled in the interrupt - let's blink an LED!

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
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
    let mut dc = pins.gpio20.into_push_pull_output();
    let mut cs = pins.gpio17.into_push_pull_output();
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
    let mut btn_c = pins.gpio14.into_pull_down_input();

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

    let items = ["Lynix Badge", "CCNB", "Socials + QR", "Device Info", "Blinky", "Settings"];

    // Draw menu items.
    let mut selected_item = 1;

    draw_menu(&mut display, items, selected_item, 0);

    let _ = display.update();

    let mut serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    let mut is_in_program = false;

    // Initialize variables for paging
    let mut current_page = 0;
    let items_per_page = 4;

    loop {
        // Read button states
        let btn_up_pressed = btn_up.is_high().unwrap();
        let btn_down_pressed = btn_down.is_high().unwrap();
        let btn_a_pressed = btn_a.is_high().unwrap();
        let btn_b_pressed = btn_b.is_high().unwrap();

        // If not in program mode
        if !is_in_program {
            if btn_up_pressed {
                if selected_item > 0 {
                    selected_item -= 1;
                }
            }

            if btn_down_pressed {
                if selected_item < items.len() - 1 {
                    selected_item += 1;
                }
            }

            // Calculate the current page based on the selected item
            current_page = selected_item / items_per_page;

            // Clear display before updating
            let _ = display.clear(BinaryColor::On);

            // Draw the menu for the current page
            draw_menu(&mut display, items, selected_item, current_page);

            if btn_a_pressed {
                // Clear display before launching a program
                let _ = display.clear(BinaryColor::On);

                // Set program mode
                is_in_program = true;

                match selected_item {
                    //0 => draw_main_screen(&mut display),
                    //1 => draw_socials_screen(&mut display),
                    // Handle other program launches based on the selected item
                    4 => draw_blinky_screen(&mut display),
                    _ => {}
                }
            }

            if btn_b_pressed {
                // Clear display before launching program menu
                let _ = display.clear(BinaryColor::On);

                // Clear Program Mode
                is_in_program = false;

                selected_item = current_page * items_per_page;
                draw_menu(&mut display, items, selected_item, current_page);
            }

            // Update the display if any button was pressed
            if btn_up_pressed || btn_down_pressed || btn_a_pressed || btn_b_pressed {
                let _ = display.update();
            }
        }

        // Your program mode handling here...
        if is_in_program {
            match selected_item {
                4 => handle_blinky_program(&mut led_pin, &mut delay),
                _ => {}
            }
        }
    }
}