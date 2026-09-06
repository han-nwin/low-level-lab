#![no_std]
#![no_main]

mod display;
mod logging;
mod sensor;

use panic_halt as _;
use rp235x_hal as hal;

use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::gpio;
use hal::uart::{DataBits, StopBits, UartConfig, ValidatedPinRx, ValidatedPinTx};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal::delay::DelayNs;

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
    let mut delay = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // reset pins to default states
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // ===== SENSOR CONFIG ====//
    // create a tuple of uart0 pins
    // select funcsel and default pad with .into_function
    let uart0_pins = (
        pins.gpio0.into_function::<gpio::FunctionUart>(), // NOTE: not needed for receiving sensor data
        pins.gpio1.into_function::<gpio::FunctionUart>(),
    );

    // take ownership of UART0 peri and pins
    // then resets and releases UART0 harware block
    // the LD2450 module communicates with the outside world through a serial port
    // with a default baud rate of 256000, 1 stop bit, and no parity bits
    let uart0 = hal::uart::UartPeripheral::new(pac.UART0, uart0_pins, &mut pac.RESETS)
        .enable(
            // the ld2450 info: 256kHz, 8 bits data, no parity, 1 stop bit.
            UartConfig::new(256000.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Create new sensor instance and pass the uart0 ownership to it
    let mut sensor = sensor::Ld2450::new(uart0);
    // ===============//

    // ==== DISPLAY CONFIG ==== //
    let display_sclk = pins.gpio2.into_function::<gpio::FunctionSpi>();
    let display_mosi = pins.gpio3.into_function::<gpio::FunctionSpi>(); // sda
    let display_dc = pins.gpio4.into_function::<gpio::FunctionSioOutput>(); // dc
    let display_cs = pins.gpio5.into_function::<gpio::FunctionSioOutput>(); // cs
    let display_rst = pins.gpio6.into_function::<gpio::FunctionSioOutput>(); // reset

    // _ = SPI state, inferred
    // _ = SPI0 peripheral, inferred
    // _ = pin tuple type, inferred
    // 8 = 8 bits per SPI transfer word
    let spi_bus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (display_mosi, display_sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        32.MHz(),
        embedded_hal::spi::MODE_0, // frame format. idle low, read high
                                   // CPOL = 0 → clock idles LOW
                                   // CPHA = 0 → receiver captures data on the first/rising edge
                                   //
                                   // Roughly:
                                   //
                                   // Clock: ____/‾‾\____/‾‾\____
                                   //              ↑       ↑
                                   //           read bit  read bit
    );

    //The buffer is a temporary SPI transmission buffer owned by mipidsi’s SpiInterface.
    let mut display_buffer = [0_u8; 512];

    let mut my_lcd = display::init(
        spi_bus,
        display_cs,
        display_dc,
        display_rst,
        &mut display_buffer,
        &mut delay,
    );
    // DEBUG:
    // display::draw_screen(&mut my_lcd).unwrap();
    my_lcd.clear(Rgb565::BLACK).unwrap();
    display::draw_screen(&mut my_lcd).unwrap();

    let mut previous_targets: Option<[sensor::TargetInfo; 3]> = None;
    let mut sweep_step = 0usize;
    let mut sweep_forward = true;
    display::draw_sweep(&mut my_lcd, sweep_step, Rgb565::GREEN).unwrap();

    // ===============//

    //====  LOGGING CONFIG ====//
    // it will move ownership of the clocks
    logging::init(pac.USB, pac.USB_DPRAM, &mut pac.RESETS, clocks.usb_clock);
    // ===============//

    loop {
        logging::poll();
        match sensor.poll() {
            Ok(Some(sensor_info)) => {
                for target_info in &sensor_info.targets_info {
                    logln!("target info: {:?}", target_info);
                }

                // Remove only the old dynamic pixels. Redrawing the background
                // repairs any grid pixels that were underneath them.
                display::draw_sweep(&mut my_lcd, sweep_step, Rgb565::BLACK).unwrap();
                if let Some(targets) = &previous_targets {
                    display::erase_targets(&mut my_lcd, targets).unwrap();
                }
                display::draw_radar_background(&mut my_lcd).unwrap();

                if sweep_step == display::SWEEP_STEP_COUNT - 1 {
                    sweep_forward = false;
                } else if sweep_step == 0 {
                    sweep_forward = true;
                }

                if sweep_forward {
                    sweep_step += 1;
                } else {
                    sweep_step -= 1;
                }

                display::draw_sweep(&mut my_lcd, sweep_step, Rgb565::GREEN).unwrap();
                display::draw_targets(&mut my_lcd, &sensor_info.targets_info).unwrap();
                previous_targets = Some(sensor_info.targets_info);
            }
            Ok(None) => {
                // logln!("continuing..");
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
