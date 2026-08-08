//! Reusable USB CDC logger for RP2350 labs.
//!
//! Call [`init`] once near the start of `main`, then call [`poll`] frequently
//! (at least every few milliseconds). Use `log!` and `logln!` like `print!`
//! and `println!`.

use rp_usb_serial::RpUsbConsole;
use rp235x_hal as hal;

/// Crystal frequency on a Raspberry Pi Pico 2.
const XTAL_FREQ_HZ: u32 = 12_000_000;

/// Configure the clocks and expose the RP2350 USB peripheral as a CDC serial
/// port. This intentionally uses the HAL so the labs can stay register-level.
pub fn init() {
    let mut peripherals = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(peripherals.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .expect("clock initialization failed");

    RpUsbConsole::init(
        peripherals.USB,
        peripherals.USB_DPRAM,
        &mut peripherals.RESETS,
        clocks.usb_clock,
    );
}

/// Advance USB enumeration and flush buffered log messages.
#[inline]
pub fn poll() {
    RpUsbConsole::poll();
}

/// Read bytes sent by the host. `poll()` must be called regularly first.
#[allow(dead_code)]
pub fn read(buffer: &mut [u8]) -> usize {
    RpUsbConsole::read(buffer)
}

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
