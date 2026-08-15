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
const UART1_RESET_MASK: u32 = (1 << 27) as u32;

// 2. Configure GPIO
const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
const GP4_CTRL: *mut u32 = (IO_BANK0_BASE + 0x24) as *mut u32;
const GP5_CTRL: *mut u32 = (IO_BANK0_BASE + 0x2c) as *mut u32;

const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;
const GPIO4_PAD: *mut u32 = (PAD_BANK0_BASE + 0x14) as *mut u32;
const GPIO5_PAD: *mut u32 = (PAD_BANK0_BASE + 0x18) as *mut u32;

#[entry]
fn main() -> ! {
    unsafe {
        // 1. Reset UART1
        // Reset
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | UART1_RESET_MASK);
        // Enable
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !UART1_RESET_MASK);
        // Wait
        let mut time_out = 1_000_000;
        while core::ptr::read_volatile(RESETS_RESET_DONE) & UART1_RESET_MASK != UART1_RESET_MASK {
            time_out -= 1;
            if time_out <= 0 {
                panic!("UART1 reset timeout")
            }
            core::hint::spin_loop();
        }

        // 2. Configure GPIO mux pins
        // IO Bank
        // NOTE: GP4 is UART TX
        // FUNSEL 0x02 for UART1_TX
        let ctr = core::ptr::read_volatile(GP4_CTRL);
        let new_ctr = (ctr & !(0b1_1111)) | 0x02;
        core::ptr::write_volatile(GP4_CTRL, new_ctr);

        // NOTE: GP5 is UART RX
        // FUNSEL 0x02 for UART1_TX
        let ctr = core::ptr::read_volatile(GP5_CTRL);
        let new_ctr = (ctr & !(0b1_1111)) | 0x02;
        core::ptr::write_volatile(GP5_CTRL, new_ctr);

        // PAD bank
        // 8 ISO: 0=connect pad
        // 7 OD:  0=allow output, 1=disable output
        // 6 IE:  1=enable input
        // 5:4 DRIVE: 0=2mA, 1=4mA, 2=8mA, 3=12mA
        // 3 PUE: 1=pull-up
        // 2 PDE: 1=pull-down
        // 1 SCHMITT: 1=noise filtering
        // 0 SLEWFAST: 1=fast output edges
        // NOTE: UART mechanism
        // UART is asynchronous serial: TX sends one bit at a time; RX samples at the baud rate.
        // Line idles HIGH, a LOW start bit begins a byte, then 8 data bits, then a HIGH stop bit.
        // -> UART 8N1 sends 10 bits per byte:
        //     - 1 LOW start bit, 8 data bits (LSB first), and 1 HIGH stop bit.
        //     - "N" means no parity bit.
        // Therefore TX needs output enabled; RX needs input enabled and an optional pull-up to stay idle HIGH.

        // NOTE: GPIO4 is the transmitter
        let pad_value = core::ptr::read_volatile(GPIO4_PAD);
        let new_pad_value = pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 7) // OD = 0: enable output (this is output disable pin)
                & !(1 << 3) // PUE = 0: no need pull up, it uses external pull-up
                & !(1 << 2) // PDE = 0: no pull-down
                & !(1 << 6); // IE = 0: not an input
        core::ptr::write_volatile(GPIO4_PAD, new_pad_value);

        // NOTE: GPIO5 is the receiver
        let pad_value = core::ptr::read_volatile(GPIO5_PAD);
        let new_pad_value = (pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 2)) //PDE = 0: no pull-down
                | (1 << 7) // OD = 1: disable output
                | (1 << 3) // PUE = 1: allow pull up
                | (1 << 6); // IE = 1: this is an input
        core::ptr::write_volatile(GPIO5_PAD, new_pad_value);
    }

    loop {
        logging::poll();
        cortex_m::asm::delay(100_000); // poll every 100k cycles
    }
}
