//! Reusable USB CDC logger for RP2350 labs.
//!
//! Call [`init`] once near the start of `main`, then call [`poll`] frequently
//! (at least every few milliseconds). Use `log!` and `logln!` like `print!`
//! and `println!`.

// NOTE: screen /dev/cu.usbmodemA74D15D81

use rp_usb_serial::RpUsbConsole;
use rp235x_hal as hal;

/// Crystal frequency on a Raspberry Pi Pico 2.
const XTAL_FREQ_HZ: u32 = 12_000_000;

/// Configure the clocks and expose the RP2350 USB peripheral as a CDC serial
/// port. This intentionally uses the HAL so the labs can stay register-level.
pub fn init() {
    // moves the low-level WATCHDOG peripheral handle into a higher-level HAL object
    let mut peripherals = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(peripherals.WATCHDOG);

    // Configure the chip clock
    // NOTE: `init_clocks_and_plls()` uses the watchdog HAL to configure all tick generators.
    // With the 12 MHz crystal, it sets TIMER0_CYCLES to 12
    // and enables the TIMER0 tick generator, producing one tick per microsecond.
    // TIMER0_CYCLES: the numbers of cycles for 1 timer tick
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
    RpUsbConsole::init(
        peripherals.USB,
        peripherals.USB_DPRAM,
        &mut peripherals.RESETS,
        clocks.usb_clock,
    );
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
