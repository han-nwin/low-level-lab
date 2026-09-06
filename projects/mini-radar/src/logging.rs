//! Reusable USB CDC logger for RP2350 labs.
//!
//! Call [`init`] once near the start of `main`, then call [`poll`] frequently
//! (at least every few milliseconds). Use `log!` and `logln!` like `print!`
//! and `println!`.
//!

//NOTE: screen /dev/cu.usbmodemA74D15D81

use rp_usb_serial::RpUsbConsole;
use rp235x_hal as hal;

/// Crystal frequency on a Raspberry Pi Pico 2.
const XTAL_FREQ_HZ: u32 = 12_000_000;

/// Configure the clocks and expose the RP2350 USB peripheral as a CDC serial
/// port. This intentionally uses the HAL so the labs can stay register-level.
pub fn init(
    usb: hal::pac::USB,
    usb_dpram: hal::pac::USB_DPRAM,
    resets: &mut hal::pac::RESETS,
    usb_clock: hal::clocks::UsbClock,
) {
    // Initialize USB serial console
    // After this call, the USB controller can present itself to
    // the computer as a CDC serial device.
    //
    // Initialization alone is insufficient because this is a
    // polled USB implementation. The program repeatedly calls:
    //
    // logging::poll();
    //
    // at src/main.rs:296. That advances USB communication and
    // sends buffered log output.
    RpUsbConsole::init(usb, usb_dpram, resets, usb_clock);
}

//            logging::poll()
//                  │
//        ┌─────────┴─────────┐
//        ▼                   ▼
//   Mac → Pico           Pico → Mac
//        │                   │
//        ▼                   ▼
//    RX queue            TX queue
//        │                   ▲
//        ▼                   │
// logging::read()          logln!()

/// Advance USB enumeration and flush buffered log messages.
#[inline]
pub fn poll() {
    RpUsbConsole::poll();
}

/// Read bytes sent by the host. `poll()` must be called regularly first.
/// Use to read bytes comming into the MCU
#[allow(dead_code)]
pub fn read(buffer: &mut [u8]) -> usize {
    RpUsbConsole::read(buffer)
}

//  - $arg is a metavariable.
// - :tt means one token tree.
// - $( ... )* means repeat the enclosed pattern zero or more times.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        rp_usb_serial::usb_print!($($arg)*)
    };
}

#[macro_export]
macro_rules! logln {
    ($($arg:tt)*) => {
        rp_usb_serial::usb_println!($($arg)*)
    };
}
