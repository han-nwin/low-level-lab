//! Raspberry Pi Pico 2 W + HLK-LD2450 starter firmware.
//!
//! Wiring (physical pin numbers are in parentheses):
//! - Pico GP0 / UART0 TX (pin 1) -> LD2450 RX
//! - Pico GP1 / UART0 RX (pin 2) <- LD2450 TX
//! - Pico GND           (pin 3) -- LD2450 GND
//! - Pico VBUS          (pin 40) -> LD2450 VCC (5 V)

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use panic_probe as _;
use rp235x_hal as hal;

use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::uart::{DataBits, StopBits, UartConfig};

const XTAL_FREQ_HZ: u32 = 12_000_000;
const RADAR_BAUD: u32 = 256_000;
const REPORT_HEADER: [u8; 4] = [0xAA, 0xFF, 0x03, 0x00];
const REPORT_FOOTER: [u8; 2] = [0x55, 0xCC];
const REPORT_FRAME_LEN: usize = 30;

#[derive(Clone, Copy)]
struct Target {
    x_mm: i16,
    y_mm: i16,
    speed_cm_s: i16,
    distance_resolution_mm: u16,
}

/// The LD2450 uses bit 15 as a sign flag rather than two's complement:
/// set means positive, clear means negative, and bits 0..14 are magnitude.
fn decode_signed(raw: u16) -> i16 {
    let magnitude = (raw & 0x7FFF) as i16;
    if raw & 0x8000 != 0 {
        magnitude
    } else {
        -magnitude
    }
}

fn parse_target(bytes: &[u8]) -> Option<Target> {
    if bytes.iter().all(|byte| *byte == 0) {
        return None;
    }

    Some(Target {
        x_mm: decode_signed(u16::from_le_bytes([bytes[0], bytes[1]])),
        y_mm: decode_signed(u16::from_le_bytes([bytes[2], bytes[3]])),
        speed_cm_s: decode_signed(u16::from_le_bytes([bytes[4], bytes[5]])),
        distance_resolution_mm: u16::from_le_bytes([bytes[6], bytes[7]]),
    })
}

/// Find a report header in the byte stream, then read and validate one frame.
/// This lets the program start cleanly even if the radar is already mid-frame.
fn read_report<U>(uart: &mut U, frame: &mut [u8; REPORT_FRAME_LEN]) -> Result<(), ()>
where
    U: embedded_io::Read,
{
    let mut matched = 0;

    loop {
        let mut byte = [0_u8; 1];
        embedded_io::Read::read_exact(uart, &mut byte).map_err(|_| ())?;

        if byte[0] == REPORT_HEADER[matched] {
            matched += 1;
            if matched == REPORT_HEADER.len() {
                break;
            }
        } else if byte[0] == REPORT_HEADER[0] {
            matched = 1;
        } else {
            matched = 0;
        }
    }

    frame[..REPORT_HEADER.len()].copy_from_slice(&REPORT_HEADER);
    embedded_io::Read::read_exact(uart, &mut frame[REPORT_HEADER.len()..]).map_err(|_| ())?;

    if frame[REPORT_FRAME_LEN - REPORT_FOOTER.len()..] != REPORT_FOOTER {
        return Err(());
    }

    Ok(())
}

/// Tell the RP2350 Boot ROM how to load this image.
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[hal::entry]
fn main() -> ! {
    info!("mini-radar starting");

    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

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

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        pins.gpio0.into_function::<hal::gpio::FunctionUart>(),
        pins.gpio1.into_function::<hal::gpio::FunctionUart>(),
    );

    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(RADAR_BAUD.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    info!("LD2450 listening on UART0 at {} baud", RADAR_BAUD);
    let mut frame = [0_u8; REPORT_FRAME_LEN];

    loop {
        match read_report(&mut uart, &mut frame) {
            Ok(()) => {
                let mut any_target = false;

                // Bytes 4..28 contain three consecutive 8-byte target records.
                for (slot, bytes) in frame[4..28].chunks_exact(8).enumerate() {
                    if let Some(target) = parse_target(bytes) {
                        any_target = true;
                        info!(
                            "target {}: x={} mm, y={} mm, speed={} cm/s, resolution={} mm",
                            slot + 1,
                            target.x_mm,
                            target.y_mm,
                            target.speed_cm_s,
                            target.distance_resolution_mm,
                        );
                    }
                }

                if !any_target {
                    info!("no targets");
                }
            }
            Err(_) => {
                // A bad frame can occur when starting in the middle of the
                // sensor's byte stream. The next read scans for a new header.
                warn!("LD2450 read error; resynchronizing");
            }
        }
    }
}

/// Metadata displayed by `picotool info`.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Pico 2 W LD2450 radar"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
