#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_probe as _;

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

    // 2. Configure GPIO
    const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
    const GP10_CTRL: *mut u32 = (IO_BANK0_BASE + 0x54) as *mut u32;
    const GP11_CTRL: *mut u32 = (IO_BANK0_BASE + 0x5c) as *mut u32;
    const GP12_CTRL: *mut u32 = (IO_BANK0_BASE + 0x64) as *mut u32;
    const GP13_CTRL: *mut u32 = (IO_BANK0_BASE + 0x6c) as *mut u32;

    const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;
    const GPIO10_PAD: *mut u32 = (PAD_BANK0_BASE + 0x2c) as *mut u32;
    const GPIO11_PAD: *mut u32 = (PAD_BANK0_BASE + 0x30) as *mut u32;
    const GPIO12_PAD: *mut u32 = (PAD_BANK0_BASE + 0x34) as *mut u32;
    const GPIO13_PAD: *mut u32 = (PAD_BANK0_BASE + 0x38) as *mut u32;

    // SIO for GP13
    const SIO_BASE: u32 = 0xd000_0000_u32;
    const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x18) as *mut u32;
    const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x20) as *mut u32;
    const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x38) as *mut u32;

    const GP13_CS_MASK: u32 = 1 << 13; // chip select mask

    // 3. Configure SPI1 (disable -> config -> enable)
    const SPI1_BASE: u32 = 0x4008_8000_u32;
    const SSPCR0: *mut u32 = (SPI1_BASE + 0x00) as *mut u32;
    const SSPCR1: *mut u32 = (SPI1_BASE + 0x04) as *mut u32;
    const SSPDR: *mut u32 = (SPI1_BASE + 0x08) as *mut u32;
    const SSPSR: *mut u32 = (SPI1_BASE + 0x0c) as *mut u32;
    const SSPCPSR: *mut u32 = (SPI1_BASE + 0x10) as *mut u32;

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
                defmt::panic!("SPI1 reset timeout")
            }
            core::hint::spin_loop();
        }

        // 2. Configure GPIO mux pins
        // IO Bank
        let gp_set = [GP10_CTRL, GP11_CTRL, GP12_CTRL];
        for gp_ref in &gp_set {
            let gp = *gp_ref;
            // FUNCSEL: 0x01 → SPI1
            let io = core::ptr::read_volatile(gp);
            let new_io = (io & !0b1_1111) | 0x01;
            core::ptr::write_volatile(gp, new_io);
        }
        // GP13 route to SIO, FUNCSEL = 5
        let io = core::ptr::read_volatile(GP13_CTRL);
        let new_io = (io & !0b1_1111) | 0x05;
        core::ptr::write_volatile(GP13_CTRL, new_io);

        // PAD Bank
        // GP10 -> SPI1 SCK
        let pad = core::ptr::read_volatile(GPIO10_PAD);
        let new_pad = pad
            & !(1 << 8) // ISO = 0: connect the pad
            & !(1 << 7) // OD = 0: allow output
            & !(1 << 6) // IE = 0: input not needed
            & !(1 << 3) // PUE = 0: no pull up
            & !(1 << 2); // PDE = 0: no pull down
        core::ptr::write_volatile(GPIO10_PAD, new_pad);

        // GP11 -> SPI1 TX
        let pad = core::ptr::read_volatile(GPIO11_PAD);
        let new_pad = pad
            & !(1 << 8) // ISO = 0: connect the pad
            & !(1 << 7) // OD = 0: allow output
            & !(1 << 6) // IE = 0: input not needed
            & !(1 << 3) // PUE = 0: no pull up
            & !(1 << 2); // PDE = 0: no pull down
        core::ptr::write_volatile(GPIO11_PAD, new_pad);

        // GP12 -> SPI1 RX
        let pad = core::ptr::read_volatile(GPIO12_PAD);
        let new_pad = (pad
            & !(1 << 8) // ISO = 0: connect the pad
            & !(1 << 7) // OD = 0: allow output
            & !(1 << 3) // PUE = 0: no pull up
            & !(1 << 2) ) // PDE = 0: no pull down
            | (1 << 6); // NOTE: IE = 1: input is needed here
        core::ptr::write_volatile(GPIO12_PAD, new_pad);

        // GP13 -> SPI1 CSn -> SIO output
        let pad = core::ptr::read_volatile(GPIO13_PAD);
        let new_pad = (pad
            & !(1 << 8) // ISO = 0: connect the pad
            & !(1 << 7) // OD = 0: allow output
            & !(1 << 6) // IE = 0: input not needed
            & !(1 << 2) ) // PDE = 0: no pull down
            | (1 << 3); // NOTE: PUE = 1: need pull up here, since when deslect it needs to be high
        core::ptr::write_volatile(GPIO13_PAD, new_pad);

        // Configure SIO for GP13
        // NOTE: This pin is used to tell if we should select this SPI connection
        // The n in CSn means “active-low.”
        // Aka when we drive it low -> select this spi slave
        // Therefore, drive it high before enable it as output to
        // prevent accidentally select
        core::ptr::write_volatile(GPIO_OUT_SET, GP13_CS_MASK); // drive it HIGH
        core::ptr::write_volatile(GPIO_OE_SET, GP13_CS_MASK); // set as output

        // 3. Configure SPI controller
        // 3.1 Disable SPI first
        // disable synschronous serial port
        let sspcr1 = core::ptr::read_volatile(SSPCR1);
        let new_sspcr1 = sspcr1 & !(1 << 1); // SSE = 0: disable SPI
        core::ptr::write_volatile(SSPCR1, new_sspcr1);

        // 3.2 Configure clock speed
        // NOTE: SPI clock = SSPCLK / (CPSDVSR x (1 + SCR))
        // We want SPI lock = 1MHz
        // SSPCLK = clk_sys = 150Mhz (can be overclocked)
        // Plan:
        // Set SCR = 0
        // Set CPSDVSR = 150Mhz

        // SCR comes from SSPCR0: serial clock rate
        let control = core::ptr::read_volatile(SSPCR0);
        let new_control = control & !(0b1111_1111 << 8); // set SCR = 0 
        core::ptr::write_volatile(SSPCR0, new_control);

        // CPSDVSR comes from SSPCPSR : clock prescale divisor
        let clock_prescale = core::ptr::read_volatile(SSPCPSR);
        let new_clock_prescale = (clock_prescale & !(0b1111_1111)) | 150_u32;
        core::ptr::write_volatile(SSPCPSR, new_clock_prescale);

        // 3.3 Configure SSPCR0
        // 3.4 Configure SSPCR1
        // 3.5. Enable SPI

        //
        //
        // 4. Implement byte transfer
        // 5. Test SPI before RFID
        // 6. Add RFID driver
    }

    loop {}
}
