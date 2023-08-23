use rp2040_hal::pac::interrupt;
use crate::ProgramState;

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    use core::sync::atomic::{AtomicBool, Ordering};
    static INIT_TEXT: AtomicBool = AtomicBool::new(false);

    // Grab the global objects. This is OK as we only access them under interrupt.
    let usb_dev = crate::USB_DEVICE.as_mut().unwrap();
     let serial = crate::USB_SERIAL.as_mut().unwrap();

    if !INIT_TEXT.load(Ordering::Relaxed) {
        INIT_TEXT.store(true, Ordering::Relaxed);
        let _ = serial.write(b"LYNIXFW READY\r\n");
    }

// Poll the USB driver with all of our supported USB Classes
    if usb_dev.poll(&mut [serial]) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_e) => {
                // Do nothing or handle error
            }
            Ok(0) => {
                // Do nothing or handle empty data
            }
            Ok(cmd) => {
                // Handle received data
                let mut wr_ptr = &buf[..cmd];
                serial.write(wr_ptr).unwrap();

                if let Ok(received_text) = core::str::from_utf8(wr_ptr) {
                    if received_text == "test\n" {
                        serial.write("lynix is cute.".as_bytes()).unwrap();
                    }
                }
            }
        }
    }
}