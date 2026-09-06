#![no_std]
#![no_main]

mod logging;
mod sensor;

use cortex_m::prelude::_embedded_hal_serial_Read;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for HAL crate
use hal::gpio;
use rp235x_hal as hal;

// use core::fmt::Write;
// use embedded_hal::delay::DelayNs;
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

/// Entry point
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
/// The function configures the rp235x peripherals, then writes to the UART in
/// an infinite loop.
#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // set up watchdog for clock
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // timer for delay
    let delay = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // reset pins to default states
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // create a tuple of uart0 pins
    // select funcsel and default pad with .into_function
    let uart0_pins = (
        pins.gpio0.into_function::<gpio::FunctionUart>(),
        pins.gpio1.into_function::<gpio::FunctionUart>(),
    );

    // take ownership of UART0 peri and pins
    // then resets and releases UART0 harware block
    // the LD2450 module communicates with the outside world through a serial port
    // with a default baud rate of 256000, 1 stop bit, and no parity bits
    let mut uart0 = hal::uart::UartPeripheral::new(pac.UART0, uart0_pins, &mut pac.RESETS)
        .enable(
            // the ld2450 info: 256kHz, 8 bits data, no parity, 1 stop bit.
            UartConfig::new(256000.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Create new sensor instance and pass the uart0 ownership to it
    let mut sensor = sensor::Ld2450::new(uart0);

    // it will move ownership of the clocks
    logging::init(pac.USB, pac.USB_DPRAM, &mut pac.RESETS, clocks.usb_clock);

    loop {
        logging::poll();
        match sensor.poll() {
            Ok(Some(sensor_info)) => {
                for target_info in sensor_info.targets_info {
                    logln!("target info: {:?}", target_info);
                }
            }
            Ok(None) => {
                logln!("continuing..");
            }
            Err(sensor::SensorError::BufferFull) => {
                logln!("buffer full. Resetting until next Header...");
            }
            Err(sensor::SensorError::Uart(error)) => {
                logln!("UART error: {:?}", error);
            }
        }
    }
}
