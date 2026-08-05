#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_halt as _;

#[used]
#[unsafe(link_section = ".start_block")]
static IMAGE_DEF: [u32; 5] = [
    // this is the RP2350 metadata
    // in datasheet: 5.9.5.1. Minimum Arm IMAGE_DEF
    0xffff_ded3, // Start marker: tells the Boot ROM an RP2350 metadata block begins here
    0x1021_0142, // Image type: executable Secure Arm code for the RP2350
    0x0000_01ff, // Last-item marker: there are no more metadata items in this block
    0x0000_0000, // Next-block relative offset: 0 makes this single block link to itself
    0xab12_3579, // End marker: tells the Boot ROM the metadata block ends here
];

#[used]
#[unsafe(link_section = ".my_metadata")]
static METADATA: [u32; 1] = [u32::from_le_bytes(*b"Han!")];

//RP2350              RFID
// SCK    -----------> SCK
// MOSI   -----------> MOSI
// MISO   <----------- MISO
// CS     -----------> SDA/SS
// GND    -----------> GND
// 3.3V   -----------> VCC
//
//                  +-------------+
// GPIO pin --------|   MUX       |
//                  |             |
//                  | choices:    |
//                  | GPIO        |
//                  | SPI0_TX     |
//                  | SPI0_RX     |
//                  | UART        |
//                  | PWM         |
//                  +-------------+
//                        |
//                        v
//
//               internal hardware
//               SPI controller
#[entry]
fn main() -> ! {
    // GP10 -> SPI1 SCK
    // GP11 -> SPI1 TX
    // GP12 -> SPI1 RX
    // GP13 -> SPI1 CSn

    // 1. RESET
    const RESETS_BASE: u32 = 0x4002_0000_u32;
    const RESETS_RESET: *mut u32 = (RESETS_BASE + 0x00) as *mut u32;
    const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32; // readonly
    const SPI1_RESET_OFFSET: u32 = (1 << 19) as u32;

    unsafe {
        // 1. Reset then enable SPI peri
        // Reset
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | SPI1_RESET_OFFSET);
        // Enable
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !SPI1_RESET_OFFSET);
        // Wait
        let mut time_out = 1_000_000;
        while core::ptr::read_volatile(RESETS_RESET_DONE) & SPI1_RESET_OFFSET != SPI1_RESET_OFFSET {
            time_out -= 1;
            if time_out <= 0 {
                panic!("SPI1 reset timeout")
            }
            core::hint::spin_loop();
        }

        // 2. Configure GPIO mux pins
        // 3. Configure SPI controller
        // 4. Enable SPI
        // 5. Implement byte transfer
        // 6. Test SPI before RFID
        // 7. Add RFID driver
    }

    loop {}
}
