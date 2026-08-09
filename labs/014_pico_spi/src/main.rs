#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_halt as _;

mod logging;

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

// GP10 -> SPI1 SCK
// GP11 -> SPI1 TX
// GP12 -> SPI1 RX
// GP13 -> SPI1 CSn

// 1. RESET
const RESETS_BASE: u32 = 0x4002_0000_u32;
const RESETS_RESET: *mut u32 = (RESETS_BASE) as *mut u32;
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
const SSPCR0: *mut u32 = (SPI1_BASE) as *mut u32;
const SSPCR1: *mut u32 = (SPI1_BASE + 0x04) as *mut u32;
const SSPSR: *mut u32 = (SPI1_BASE + 0x0c) as *mut u32;
const SSPCPSR: *mut u32 = (SPI1_BASE + 0x10) as *mut u32;

const SSPSR_BSY: u32 = 1 << 4;
#[entry]
fn main() -> ! {
    // CHEAT: use rp235x-hal for USB logging while the lab itself stays
    // register-level. Keep polling USB in the main loop below.
    logging::init();

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
        // 3.1 Disable SPI first: Use SSPCR1
        // disable synschronous serial port
        let sspcr1 = core::ptr::read_volatile(SSPCR1);
        let new_sspcr1 = sspcr1 & !(1 << 1); // SSE = 0: disable SPI
        core::ptr::write_volatile(SSPCR1, new_sspcr1);

        // 3.2 Configure clock speed
        // NOTE: SPI clock = clk_peri / (CPSDVSR × (1 + SCR))
        // RP2350 attachs clk_peri to clk_sys by default, unless we configure. Check lab13
        // ASSUMPTION:
        // CPSDVSR = 150 (a unitless divider, not 150 MHz)
        // SCR = 0
        // IF clk_peri = 150 MHz:
        // SPI clock = 150 MHz / 150 = 1 MHz

        // SCR comes from SSPCR0: serial clock rate
        let control = core::ptr::read_volatile(SSPCR0);
        let new_control = control & !(0b1111_1111 << 8); // set SCR = 0 
        core::ptr::write_volatile(SSPCR0, new_control);

        // CPSDVSR comes from SSPCPSR : clock prescale divisor
        // set it to 150
        let clock_prescale = core::ptr::read_volatile(SSPCPSR);
        let new_clock_prescale = (clock_prescale & !(0b1111_1111)) | 150_u32;
        core::ptr::write_volatile(SSPCPSR, new_clock_prescale);

        // 3.3 Configure DSS in SSPCR0: Data size
        // How many bits make up one transfer? 8 bits
        let cr0 = core::ptr::read_volatile(SSPCR0);
        let new_cr0 = (cr0 & !(0b1111)) | (0b0111); // set DSS = 8
        core::ptr::write_volatile(SSPCR0, new_cr0);

        // 3.4 Configure FRF in SSPCR0: Format
        // just use the motorola format
        let cr0 = core::ptr::read_volatile(SSPCR0);
        let new_cr0 = cr0 & !(0b11 << 4); // set FRF = 00
        core::ptr::write_volatile(SSPCR0, new_cr0);

        // 3.5 Configure SPO in SSPCR0: Idle level
        // SP0 = 0: CLock is idle LOW
        //       ___     ___
        // _____|   |___|   |____
        // idle = 0
        // First transition goes: Low -> High
        let cr0 = core::ptr::read_volatile(SSPCR0);
        let new_cr0 = cr0 & !(0b1 << 6); // set SPO = 00
        core::ptr::write_volatile(SSPCR0, new_cr0);

        // 3.6 Configure SPH in SSPCR0: Phase
        // Controls when data is sampled relative
        // SPH = 0
        //       ^       ^
        // ______|_______|______
        // DATA:
        // =====[sample]=======
        // The slave read data on the first edge
        let cr0 = core::ptr::read_volatile(SSPCR0);
        let new_cr0 = cr0 & !(0b1 << 7); // set SPH = 00
        core::ptr::write_volatile(SSPCR0, new_cr0);

        // 3.7. Enable SPI: Use SSPCR1
        let sspcr1 = core::ptr::read_volatile(SSPCR1);
        let new_sspcr1 = (sspcr1 & !(1 << 1)) | (1 << 1); // SSE = 1: enable SPI
        core::ptr::write_volatile(SSPCR1, new_sspcr1);

        // 5. Add RFID driver, and confirm it's connected
        // Read VersionReg
        //   RC522 pin    Connect to RP2350     Purpose
        // ━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        //  SDA / SS     GP13                  Manual active-low chip select
        // ───────────  ────────────────────  ───────────────────────────────
        //  SCK          GP10                  SPI1 clock
        // ───────────  ────────────────────  ───────────────────────────────
        //  MOSI         GP11                  SPI1 TX
        // ───────────  ────────────────────  ───────────────────────────────
        //  MISO         GP12                  SPI1 RX
        // ───────────  ────────────────────  ───────────────────────────────
        //  IRQ          Leave disconnected    Not needed for polling
        // ───────────  ────────────────────  ───────────────────────────────
        //  GND          GND                   Common ground
        // ───────────  ────────────────────  ───────────────────────────────
        //  RST          3.3 V initially       Keep reader out of reset| can be controlled by a GPIO
        // ───────────  ────────────────────  ───────────────────────────────
        //  3.3V         3.3 V                 Module power

        // 5.1 Since VersionReg address is 0x37
        let version = read_rc522_register(0x37);
        // 5.2 log
        logln!("RC522 version: 0x{:02x}", version);

        // 6. Write the data to the RC522
        // 0x0B: WaterLevelReg
        // Writable bits: 5:0
        // Reset value: 0x08
        let original_water = read_rc522_register(0x0B);
        write_rc522_register(0x0B, 0x33); // Write 33
        let new_water = read_rc522_register(0x0B);
        logln!(
            "WaterLevelReg: 0x{:02x} -> 0x{:02x}",
            original_water,
            new_water
        );
        write_rc522_register(0x0B, original_water);

        let mut log_countdown = 1_000;
        loop {
            // NOTE:
            // USB is a polled protocol. Calling this often lets the Mac
            // enumerate the device and drains queued log messages.
            logging::poll();
            cortex_m::asm::delay(100_000); // poll every 100k cycles

            log_countdown -= 1;
            if log_countdown == 0 {
                // 5.1 Since VersionReg address is 0x37
                let version = read_rc522_register(0x37);
                // 5.2 log
                logln!("RC522 version: 0x{:02x}", version);

                // 6. Write the data to the RC522
                // 0x0B: WaterLevelReg
                // Writable bits: 5:0
                // Reset value: 0x08
                let original_water = read_rc522_register(0x0B);
                write_rc522_register(0x0B, 0x33); // Write 33
                let new_water = read_rc522_register(0x0B);
                logln!(
                    "WaterLevelReg: 0x{:02x} -> 0x{:02x}",
                    original_water,
                    new_water
                );
                write_rc522_register(0x0B, original_water);

                log_countdown = 1_000;
            }
        }
    }
}

// 4. Implement byte transfer
// NOTE:   Because SPI is full duplex, every transmitted frame also creates one received frame.
// Therefore:
// one write to SSPDR → eventually one readable value in SSPDR
// Even if you only want to write to the RFID module, you still need to
// read and discard the corresponding received byte so the RX FIFO
// doesn’t fill up.
unsafe fn spi_transfer_byte(tx: u8) -> u8 {
    //                 SSPDR
    //                /     \
    // CPU writes → TX FIFO  RX FIFO → CPU reads
    //                |         ^
    //                v         |
    //              MOSI      MISO
    const SPI1_BASE: u32 = 0x4008_8000_u32;
    // we read/write data here. 16bits 15:0. 2bytes
    // But since we configure DSS 0b0111 = 8bits frame
    // we can only use 7:0. 8bits = 1 byte data in SSPDR
    const SSPDR: *mut u32 = (SPI1_BASE + 0x08) as *mut u32; // for read/write data

    const SSPSR: *const u32 = (SPI1_BASE + 0x0c) as *const u32; //readonly. To check status
    const SSPSR_RNE: u32 = 1 << 2; // Receive FIFO not empty
    const SSPSR_TNF: u32 = 1 << 1; // Transmit FIFO not full

    unsafe {
        //wait until there is room in the transmit FIFO
        while core::ptr::read_volatile(SSPSR) & SSPSR_TNF == 0 {
            core::hint::spin_loop();
        }

        // Place outgoing byte in the transmit FIFO
        core::ptr::write_volatile(SSPDR, tx as u32);

        // wait till the received byte enters the receive FIFO
        while core::ptr::read_volatile(SSPSR) & SSPSR_RNE == 0 {
            core::hint::spin_loop();
        }

        // SSPDR reads from the receive FIFO
        core::ptr::read_volatile(SSPDR) as u8
    }
}

unsafe fn read_rc522_register(reg: u8) -> u8 {
    unsafe {
        // 1. Clear GP13 to select the RC522
        core::ptr::write_volatile(GPIO_OUT_CLR, GP13_CS_MASK);

        // 2. read from register
        // Table 8 in RC522 Datasheet
        // bit 7:    1 = read, 0 = write
        // bits 6:1: register address
        // bit 0:    always 0
        let read_command = (1 << 7) | (reg << 1);
        let _discard = spi_transfer_byte(read_command);

        // 3. Send some dummy data to generate clocks and receive the value
        let data = spi_transfer_byte(0x00);

        // 4. wait for the transmitting to finished. AKA SSPSR.BSY = 0a
        while core::ptr::read_volatile(SSPSR) & SSPSR_BSY != 0 {
            core::hint::spin_loop();
        }

        // 5. Delelect Gp13
        core::ptr::write_volatile(GPIO_OUT_SET, GP13_CS_MASK);

        data
    }
}

unsafe fn write_rc522_register(reg: u8, data: u8) {
    unsafe {
        // 1. Clear GP13 to select the RC522
        core::ptr::write_volatile(GPIO_OUT_CLR, GP13_CS_MASK);

        // 2. write to register
        // Table 8 in RC522 Datasheet
        // bit 7:    1 = read, 0 = write
        // bits 6:1: register address
        // bit 0:    always 0
        let write_command = (0 << 7) | (reg << 1);
        let _discard = spi_transfer_byte(write_command);

        // 3. Send the data
        spi_transfer_byte(data);

        // 4. wait for the transmitting to finished. AKA SSPSR.BSY = 0a
        while core::ptr::read_volatile(SSPSR) & SSPSR_BSY != 0 {
            core::hint::spin_loop();
        }

        // 5. Delelect Gp13
        core::ptr::write_volatile(GPIO_OUT_SET, GP13_CS_MASK);
    }
}
