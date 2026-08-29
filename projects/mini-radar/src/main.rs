#![no_std]
#![no_main]

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal as hal;

use hal::gpio;

// Some things we need
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use hal::clocks::Clock;
use hal::fugit::RateExtU32;

// UART related types
use hal::uart::{DataBits, StopBits, UartConfig, ValidatedPinRx, ValidatedPinTx};

/// Tell the Boot ROM about our application
#[used]
#[unsafe(link_section = ".start_block")]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp235x peripherals, then writes to the UART in
/// an infinite loop.
#[hal::entry]
fn main() -> ! {
    loop {}
}
