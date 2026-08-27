// NOTE:
// UART
// ----
// TX = transmit
// RX = receive
// ----
// Connect CROSSED:
// MCU TX -> device RX
// MCU RX <- device TX
// ----
// Think:
// TX talks to RX
// RX listens to TX
// ----
// Optional:
// RTS / CTS = hardware flow control
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
const UART0_RESET_MASK: u32 = (1 << 26) as u32; // GP0 and GP1
const UART1_RESET_MASK: u32 = (1 << 26) as u32; // GP4 and GP5

// 2. Configure GPIO
const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
const GP0_CTRL: *mut u32 = (IO_BANK0_BASE + 0x04) as *mut u32;
const GP1_CTRL: *mut u32 = (IO_BANK0_BASE + 0x0c) as *mut u32;
const GP4_CTRL: *mut u32 = (IO_BANK0_BASE + 0x24) as *mut u32;
const GP5_CTRL: *mut u32 = (IO_BANK0_BASE + 0x2c) as *mut u32;

const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;
const GPIO0_PAD: *mut u32 = (PAD_BANK0_BASE + 0x04) as *mut u32;
const GPIO1_PAD: *mut u32 = (PAD_BANK0_BASE + 0x08) as *mut u32;
const GPIO4_PAD: *mut u32 = (PAD_BANK0_BASE + 0x14) as *mut u32;
const GPIO5_PAD: *mut u32 = (PAD_BANK0_BASE + 0x18) as *mut u32;

// NOTE: A useful general technique for any peripheral is:
// - Reset/release it
// - Configure its clock or speed
// - Configure its data format
// - Enable it
// - Check status
// - Read/write data
//
// NOTE: Requirements:
// - UART0 and UART1 both send and receive
// - 115200 baud
// - 8N1: 8 data bits, no parity, 1 stop bit
// - Polling, no interrupts or DMA
//
// NOTE: Translate requirements into registers:
// Where does data go?       -> UARTDR
// How do I know it's ready? -> UARTFR
// How fast does it run?     -> UARTIBRD + UARTFBRD
// What frame format?        -> UARTLCR_H
// How do I turn it on?      -> UARTCR

// 3. Configure UART0 & UART1 registers
const UART0_BASE: u32 = 0x4007_0000_u32;
const UART0_DR: *mut u32 = (UART0_BASE) as *mut u32; // data
const UART0_FR: *const u32 = (UART0_BASE + 0x0018) as *const u32; // flag
const UART0_IBRD: *mut u32 = (UART0_BASE + 0x0024) as *mut u32; // integer braud rate
const UART0_FBRD: *mut u32 = (UART0_BASE + 0x0028) as *mut u32; // fractional braud rate
const UART0_LCR_H: *mut u32 = (UART0_BASE + 0x002c) as *mut u32; // line control
const UART0_CR: *mut u32 = (UART0_BASE + 0x0030) as *mut u32; // control

const UART1_BASE: u32 = 0x4007_8000_u32;
const UART1_DR: *mut u32 = (UART1_BASE) as *mut u32; // data
const UART1_FR: *const u32 = (UART1_BASE + 0x0018) as *const u32; // flag
const UART1_IBRD: *mut u32 = (UART1_BASE + 0x0024) as *mut u32; // integer braud rate
const UART1_FBRD: *mut u32 = (UART1_BASE + 0x0028) as *mut u32; // fractional braud rate
const UART1_LCR_H: *mut u32 = (UART1_BASE + 0x002c) as *mut u32; // line control
const UART1_CR: *mut u32 = (UART1_BASE + 0x0030) as *mut u32; // control

#[entry]
fn main() -> ! {
    logging::init(); // logging and configure clock sys to 150MHZ
    unsafe {
        // 1. Reset UART0 and UART1
        // Reset
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | UART0_RESET_MASK | UART1_RESET_MASK);
        // Enable
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !(UART0_RESET_MASK | UART1_RESET_MASK));
        // Wait
        let mut time_out = 1_000_000;
        while core::ptr::read_volatile(RESETS_RESET_DONE) & UART0_RESET_MASK != UART0_RESET_MASK
            || core::ptr::read_volatile(RESETS_RESET_DONE) & UART1_RESET_MASK != UART1_RESET_MASK
        {
            time_out -= 1;
            if time_out <= 0 {
                panic!("UART1 reset timeout")
            }
            core::hint::spin_loop();
        }

        // 2. Configure GPIO mux pins
        // IO Bank
        // NOTE: GP0 and GP4 is UART TX
        // FUNSEL 0x02 for UART0_TX
        let ctr = core::ptr::read_volatile(GP0_CTRL);
        let new_ctr = (ctr & !(0b1_1111)) | 0x02;
        core::ptr::write_volatile(GP0_CTRL, new_ctr);
        // FUNSEL 0x02 for UART1_TX
        let ctr = core::ptr::read_volatile(GP4_CTRL);
        let new_ctr = (ctr & !(0b1_1111)) | 0x02;
        core::ptr::write_volatile(GP4_CTRL, new_ctr);

        // NOTE: GP1 and GP5 is UART RX
        // FUNSEL 0x02 for UART0_TX
        let ctr = core::ptr::read_volatile(GP1_CTRL);
        let new_ctr = (ctr & !(0b1_1111)) | 0x02;
        core::ptr::write_volatile(GP1_CTRL, new_ctr);
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

        // NOTE: GPIO0 and GPIO4 are the transmitters
        let pad_value = core::ptr::read_volatile(GPIO0_PAD);
        let new_pad_value = pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 7) // OD = 0: enable output (this is output disable pin)
                & !(1 << 3) // PUE = 0: no need pull up, it uses external pull-up
                & !(1 << 2) // PDE = 0: no pull-down
                & !(1 << 6); // IE = 0: not an input
        core::ptr::write_volatile(GPIO0_PAD, new_pad_value);

        let pad_value = core::ptr::read_volatile(GPIO4_PAD);
        let new_pad_value = pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 7) // OD = 0: enable output (this is output disable pin)
                & !(1 << 3) // PUE = 0: no need pull up, it uses external pull-up
                & !(1 << 2) // PDE = 0: no pull-down
                & !(1 << 6); // IE = 0: not an input
        core::ptr::write_volatile(GPIO4_PAD, new_pad_value);

        // NOTE: GPOP1 and GPIO5 are the receivers
        let pad_value = core::ptr::read_volatile(GPIO1_PAD);
        let new_pad_value = (pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 2)) //PDE = 0: no pull-down
                | (1 << 7) // OD = 1: disable output
                | (1 << 3) // PUE = 1: allow pull up
                | (1 << 6); // IE = 1: this is an input
        core::ptr::write_volatile(GPIO1_PAD, new_pad_value);

        let pad_value = core::ptr::read_volatile(GPIO5_PAD);
        let new_pad_value = (pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 2)) //PDE = 0: no pull-down
                | (1 << 7) // OD = 1: disable output
                | (1 << 3) // PUE = 1: allow pull up
                | (1 << 6); // IE = 1: this is an input
        core::ptr::write_volatile(GPIO5_PAD, new_pad_value);

        // 3. Configure UART registers
        // 3.1 UART_CR, disable it before configure
        core::ptr::write_volatile(UART0_CR, 0);
        core::ptr::write_volatile(UART1_CR, 0);

        // 3.2 UART_IBRD, integer baud rate divisor
        // UART commonly want 115200HZ
        // NOTE: 115200HZ baud from clk_peri, which is 150MHZ
        // UARTCLK = clk_peri = 150MHZ
        // baud = UARTCLK / (16 × divisor) = 115200
        // -> divisor = 150M / (16 * 115200) = 81.3802
        // -> integer = 81
        // -> fraction = 0.3802
        core::ptr::write_volatile(UART0_IBRD, 81);
        core::ptr::write_volatile(UART1_IBRD, 81);

        // 3.3 UART_FBRD, fractional baud rate divisor
        // NOTE: FBRD is a 6-bit fractional register, so it represents fractions in 1/64 steps:
        // -> fraction = 0.3802 * 64 = 24
        core::ptr::write_volatile(UART0_FBRD, 24);
        core::ptr::write_volatile(UART1_FBRD, 24);

        // 3.4 UART_LCR_H, line control
        // 8N1: 8 data bits, no parity, 1 stop bit
        // bit 7 = 0: no parity
        // bit 5:6 = 11 = 8bits data
        // bit 4 = 1 = enable fifo
        // bit 3 = 0 = no 2 stop bits
        // bit 2 = 0 = no parity bit
        // bit 1 = 0 = parity disable
        // bit 0 = 0 = no send break
        // -> value 0b01110000 = 0x70
        core::ptr::write_volatile(UART0_LCR_H, 0x70);
        core::ptr::write_volatile(UART1_LCR_H, 0x70);

        // 3.5 Enable UART: RX and TX
        // 0b0011_0000_0001
        // bit 0 = 1: enable UART
        // bit 8 = 1: enable TX
        // bit 9 = 1: enable RX
        core::ptr::write_volatile(UART0_CR, 0x301);
        core::ptr::write_volatile(UART1_CR, 0x301);
    }

    loop {
        logging::poll();
        cortex_m::asm::delay(100_000); // poll every 100k iterations, around 3 cycles

        // Write from UART0 to UART1
        let message = b"Message from UART0 to UART1\r\n";
        for &byte in message {
            unsafe {
                uart_write_byte(UART1_DR, UART1_FR, byte);
                let read_byte = uart_read_byte(UART0_DR, UART0_FR);
                logln!("Read byte from UART0: {}", read_byte as char);
            }
        }
    }
}

unsafe fn uart_write_byte(uart_dr: *mut u32, uart_fr: *const u32, byte: u8) {
    unsafe {
        // bit 5 TXFF = 1 mean transmit FIFO is full
        // wait till it's empty aka = 0
        while core::ptr::read_volatile(uart_fr) & (1 << 5) != 0 {
            core::hint::spin_loop();
        }

        //then write the byte
        core::ptr::write_volatile(uart_dr, byte as u32);
    }
}

unsafe fn uart_read_byte(uart_dr: *mut u32, uart_fr: *const u32) -> u8 {
    unsafe {
        // bit 5 RXFF = 1 mean receive FIFO is empty
        // wait till it's full aka = 0
        while core::ptr::read_volatile(uart_fr) & (1 << 4) != 0 {
            core::hint::spin_loop();
        }

        // DATA is bit 7:0 aka 0b1111_1111 = 0xff
        (core::ptr::read_volatile(uart_dr) & 0xff) as u8
    }
}
