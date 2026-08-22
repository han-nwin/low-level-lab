#![no_std]
#![no_main]

mod logging;
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

// 1. RESET
const RESETS_BASE: u32 = 0x4002_0000_u32;
const RESETS_RESET: *mut u32 = (RESETS_BASE) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32; // readonly
const TIMER0_RESET_MASK: u32 = (1 << 23) as u32; // GP0 and GP1

// 2.
const TIMER0_BASE: u32 = 0x400b_0000_u32;

#[entry]
fn main() -> ! {
    logging::init(); // logging and configure clock sys to 150MHZ
    unsafe {
        // 1. Reset TIMER0
        // Reset
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | TIMER0_RESET_MASK);
        // Enable
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !(TIMER0_RESET_MASK));
        // Wait
        let mut time_out = 1_000_000;
        while core::ptr::read_volatile(RESETS_RESET_DONE) & TIMER0_RESET_MASK != TIMER0_RESET_MASK {
            time_out -= 1;
            if time_out <= 0 {
                panic!("UART1 reset timeout")
            }
            core::hint::spin_loop();
        }
    }

    loop {
        logging::poll();
        cortex_m::asm::delay(100_000); // poll every 100k cycles
    }
}
