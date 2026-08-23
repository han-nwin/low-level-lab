// Pico 2 W                 LCD module
//
// 5V       ------------->  VDD
// GND      ------------->  GND
// GP10     <--- level shifter ---> SDA
// GP11     <--- level shifter ---> SCL

// 1. RESET
const RESETS_BASE: u32 = 0x4002_0000_u32;
const RESETS_RESET: *mut u32 = (RESETS_BASE) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32; // readonly
const I2C1_RESET_MASK: u32 = (1 << 5) as u32;

// 2. Configure GPIO
const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
const GP10_CTRL: *mut u32 = (IO_BANK0_BASE + 0x54) as *mut u32;
const GP11_CTRL: *mut u32 = (IO_BANK0_BASE + 0x5c) as *mut u32;

const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;
const GPIO10_PAD: *mut u32 = (PAD_BANK0_BASE + 0x2c) as *mut u32;
const GPIO11_PAD: *mut u32 = (PAD_BANK0_BASE + 0x30) as *mut u32;

// 3. Configure I2C1 registers
// NOTE: Target speed = 100kHz
//
// Registers needed to configure controller mode and bus timing
// Register          Why needed
// ━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  IC_ENABLE         Controller must be disabled while changing
//                    configuration
// ────────────────  ─────────────────────────────────────────────────────────
//  IC_CON            Select controller/master mode and protocol behavior
// ────────────────  ─────────────────────────────────────────────────────────
//  IC_FS_SCL_HCNT    Determines how long SCL stays high
// ────────────────  ─────────────────────────────────────────────────────────
//  IC_FS_SCL_LCNT    Determines how long SCL stays low
// ────────────────  ─────────────────────────────────────────────────────────
//  IC_SDA_HOLD       Keeps SDA stable after SCL changes
// ────────────────  ─────────────────────────────────────────────────────────
//  IC_FS_SPKLEN      Filters very short electrical noise pulses
//
const I2C1_BASE: u32 = 0x4009_8000_u32;
const IC_CON: *mut u32 = (I2C1_BASE) as *mut u32;
const IC_FS_SCL_HCNT: *mut u32 = (I2C1_BASE + 0x1c) as *mut u32; // Fast mode
const IC_FS_SCL_LCNT: *mut u32 = (I2C1_BASE + 0x20) as *mut u32; // Fast mode
const IC_ENABLE: *mut u32 = (I2C1_BASE + 0x6c) as *mut u32;
const IC_SDA_HOLD: *mut u32 = (I2C1_BASE + 0x7c) as *mut u32;
const IC_FS_SPKLEN: *mut u32 = (I2C1_BASE + 0xa0) as *mut u32;

// 4. Registers used to select a target and transmit bytes.
const IC_TAR: *mut u32 = (I2C1_BASE + 0x04) as *mut u32;
const IC_DATA_CMD: *mut u32 = (I2C1_BASE + 0x10) as *mut u32;
const IC_RAW_INTR_STAT: *const u32 = (I2C1_BASE + 0x34) as *const u32;
const IC_CLR_TX_ABRT: *const u32 = (I2C1_BASE + 0x54) as *const u32;
const IC_CLR_STOP_DET: *const u32 = (I2C1_BASE + 0x60) as *const u32;
const IC_STATUS: *const u32 = (I2C1_BASE + 0x70) as *const u32;
const IC_TX_ABRT_SOURCE: *const u32 = (I2C1_BASE + 0x80) as *const u32;

// 5. LCD registers
// 0100 1001
// ^^^^ ||||
// data │││└─ RS = 1: character data
//      ││└── RW = 0: write
//      │└─── EN = 0: 1: data ready, 0: accept the data
//      └──── Backlight = 1
// 0x08 sets P3, commonly the LCD backlight.
const LCD_RS: u8 = 1 << 0;
const LCD_EN: u8 = 1 << 2;
const LCD_RW: u8 = 1 << 1;
const LCD_BACKLIGHT: u8 = 1 << 3;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum LcdRow {
    Row1 = 0,
    Row2 = 1,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum LcdCol {
    Col1 = 0,
    Col2 = 1,
    Col3 = 2,
    Col4 = 3,
    Col5 = 4,
    Col6 = 5,
    Col7 = 6,
    Col8 = 7,
    Col9 = 8,
    Col10 = 9,
    Col11 = 10,
    Col12 = 11,
    Col13 = 12,
    Col14 = 13,
    Col15 = 14,
    Col16 = 15,
}

pub struct Lcd {
    _private: (),
}

impl Lcd {
    pub fn init() -> Self {
        unsafe {
            // 1. Reset I2C1
            // Reset
            let reset = core::ptr::read_volatile(RESETS_RESET);
            core::ptr::write_volatile(RESETS_RESET, reset | I2C1_RESET_MASK);
            // Enable
            let reset = core::ptr::read_volatile(RESETS_RESET);
            core::ptr::write_volatile(RESETS_RESET, reset & !I2C1_RESET_MASK);
            // Wait
            let mut time_out = 1_000_000;
            while core::ptr::read_volatile(RESETS_RESET_DONE) & I2C1_RESET_MASK != I2C1_RESET_MASK {
                time_out -= 1;
                if time_out <= 0 {
                    panic!("I2C1 reset timeout")
                }
                core::hint::spin_loop();
            }

            // 2. Configure GPIO mux pins
            // IO Bank
            let gp_set = [GP10_CTRL, GP11_CTRL];
            for &gp in &gp_set {
                //FUNSEL 0x03 for I2C
                let ctr = core::ptr::read_volatile(gp);
                let new_ctr = (ctr & !(0b1_1111)) | 0x03;
                core::ptr::write_volatile(gp, new_ctr);
            }
            //
            // NOTE:
            // PAD bank
            // I2C lines are bidirectional and open-drain.
            // The peripheral pulls them low for 0 and releases them for 1 per input.
            // IE stays enabled so SDA can receive data/ACK and SCL can detect
            // clock stretching. External resistors pull released lines high.
            let pad_set = [GPIO10_PAD, GPIO11_PAD];
            for &pad in &pad_set {
                let pad_value = core::ptr::read_volatile(pad);
                let new_pad_value = (pad_value
                & !(1 << 8) // ISO = 0: connect pad
                & !(1 << 7) // OD = 0: let peripheral control output
                & !(1 << 3) // PUE = 0: no need pull up, it uses external pull-up
                & !(1 << 2)) // PDE = 0: no pull-down
                | (1 << 6); // IE = 1: always allow input
                core::ptr::write_volatile(pad, new_pad_value);
            }

            // 3. Configure I2C
            //
            // 3.1 Disable it first
            core::ptr::write_volatile(IC_ENABLE, 0);

            // 3.2 Controller mode,
            // fast-speed register set
            // restart enabled
            // slave mode disable
            let ic_con = core::ptr::read_volatile(IC_CON);
            let new_ic_con = ic_con
        | (1<<0) // MASTER_MODE on
        | (2 << 1) // SPEED = fast-mode
        | (1 << 5) // Restart enabled
        | (1 << 6); // Slave disable
            core::ptr::write_volatile(IC_CON, new_ic_con);

            // 3.3 Configure I2C speed
            // We want 100KHz
            //                   one I²C clock
            //           ┌────────────┐
            // SCL  ─────┘            └──────────────────
            //           600 counts        900 counts
            //              high               low
            //
            // At a 150 MHz system clock:
            // 600 counts high + 900 counts low = 1500 counts
            // 150,000,000 / 1500 = 100,000 Hz
            //
            // In short:
            // SCL high = 600 system-clock cycles
            // SCL low  = 900 system-clock cycles
            // NOTE: I2C standard wants high value < low value -> ie 600h 900l
            // Minimum high time: 4.0 µs
            // Minimum low time:  4.7 µs
            // High time = 600 * (1/150MHz) = 4.0 µs
            // Low time = 900 * (1/150MHz) = 6.0 µs
            // => statisfied
            //
            // So:
            // Configure an approximately 100 kHz I2C clock.
            core::ptr::write_volatile(IC_FS_SCL_HCNT, 600);
            core::ptr::write_volatile(IC_FS_SCL_LCNT, 900);

            // 3.3 SDA hold time and input spike filter.
            // SDA must remain stable for about 300 ns after SCL falls.
            // At 150 MHz, one system-clock cycle is 6.67 ns:
            // 300 ns / 6.67 ns = 45 cycles; use 46 to avoid rounding below 300 ns.
            core::ptr::write_volatile(IC_SDA_HOLD, 46);

            // Filter electrical noise pulses of approximately 50 ns or less.
            // 50 ns / 6.67 ns ≈ 7.5 cycles; round up to 8 cycles.
            core::ptr::write_volatile(IC_FS_SPKLEN, 8);

            // 3.4 Enable the controller.
            core::ptr::write_volatile(IC_ENABLE, 1);

            // 4. Set LCD target address, its' 0x27
            // disable I2C first
            core::ptr::write_volatile(IC_ENABLE, 0);
            // Set target
            core::ptr::write_volatile(IC_TAR, 0x27);
            // enable I2C
            core::ptr::write_volatile(IC_ENABLE, 1);

            // 4. Write some data
            //  0100 1001
            // ^^^^ ||||
            // data │││└─ RS = 1: character data
            //      ││└── RW = 0: write
            //      │└─── EN = 0
            //      └──── Backlight = 1
            // 0x08 sets P3, commonly the LCD backlight.
            for _ in 0..3 {
                Self::i2c_write(0x00); // P3 = 0 backlight off
                cortex_m::asm::delay(3_000_000); // delay a bit
                Self::i2c_write(0x08); // P3 = 1 backlight on
                cortex_m::asm::delay(3_000_000); // delay a bit
            }
            crate::logln!("HLF8574 acknowledged address 0x27");

            // Wait for the LCD to finish powering up
            cortex_m::asm::delay(7_500_000);

            // NOTE:
            //  This places 0011 on LCD pins D7–D4. It tells the LCD to use 8-bit mode. We
            // send it three times so the LCD recognizes it regardless of its initial state.
            // 0011 → wait at least 4.1 ms
            // 0011 → wait at least 100 µs
            // 0011 → LCD is now synchronized in 8-bit mode
            Self::startup_sequence();
            // ============ END startup sequence =========//

            Self { _private: () }
        }
    }

    pub fn write_byte_at(&mut self, row: LcdRow, column: LcdCol, byte: u8) {
        Self::set_cursor(row, column);
        Self::lcd_write_byte(byte, true); // true = write data
    }

    fn i2c_write(byte: u8) {
        unsafe {
            // IC_STATUS.TFNF is bit 1:
            // 1 = transmit FIFO has space
            // 0 = transmit FIFO is full
            while core::ptr::read_volatile(IC_STATUS) & (1 << 1) == 0 {
                core::hint::spin_loop();
            }

            // Send one byte to the HLF8574.
            // IC_DATA_CMD bit 9 requests a STOP after this byte.
            core::ptr::write_volatile(IC_DATA_CMD, u32::from(byte) | (1 << 9));

            loop {
                let status = core::ptr::read_volatile(IC_RAW_INTR_STAT);
                // TX_ABRT: This bit indicates if DW_apb_i2c, as an I2C transmitter is unable
                // complete the intended actions on the contents of the transmit FIFO.
                if status & (1 << 6) != 0 {
                    let reason = core::ptr::read_volatile(IC_TX_ABRT_SOURCE);
                    // Clear TX_ABRT Interrupt Register by reading it
                    let _ = core::ptr::read_volatile(IC_CLR_TX_ABRT);
                    crate::logln!("TX_ABRT: reason = 0x{:x}", reason);
                    panic!("TX_ABRT: reason = {:#x}", reason);
                }

                // STOP_DET: Indicates whether a STOP condition has occurred on the I2C
                // interface regardless of whether DW_apb_i2c is operating in slave or master mode
                if status & (1 << 9) != 0 {
                    // Clear STOP_DET Interrupt Register by reading it
                    let _ = core::ptr::read_volatile(IC_CLR_STOP_DET);
                    break;
                }

                core::hint::spin_loop();
            }
        }
    }

    fn lcd_write_nibble(nibble: u8, is_data: bool) {
        let rs = if is_data { LCD_RS } else { 0 };

        // a nibble = 4 bits
        // it's the 7:4 in the LCD
        // 0100 1001
        // ^^^^
        // data
        let output = ((nibble & 0b0000_1111) << 4) | LCD_BACKLIGHT | rs;

        // Set data and control lines while EN is low.
        // This gives them time to settle before the enable pulse.
        Self::i2c_write(output);

        // EN high tells the LCD that data is ready.
        Self::i2c_write(output | LCD_EN);

        // Hold EN high for longer than the 450 ns minimum.
        cortex_m::asm::delay(150); // About 1 µs at 150 MHz

        // The falling edge makes the LCD accept the nibble.
        Self::i2c_write(output);

        // Allow the LCD instruction time to settle before the next nibble.
        cortex_m::asm::delay(7_500); // About 50 µs at 150 MHz
    }

    fn lcd_write_byte(byte: u8, is_data: bool) {
        // NOTE: each write_byte broken into 2 write_nibble
        // we write 2 nibbles so break it into 2 write nibbles
        // LCD only take 4bits (1 nibble) write at a time
        Self::lcd_write_nibble(byte >> 4, is_data); // write 4 most signigicant bits
        Self::lcd_write_nibble(byte & 0b0000_1111, is_data); // write last 4 significant bits
    }

    fn set_cursor(row: LcdRow, column: LcdCol) {
        // NOTE: each write_byte broken into 2 write_nibble
        // Row 1: 0x00–0x0F
        // Row 2: 0x40–0x4F
        let row_address = if row == LcdRow::Row1 { 0x00 } else { 0x40 };
        // 1. Set the row we wanna write on
        //  RS = 0  → instruction/command
        // R/W = 0 → write
        //
        // and LCD command bit 7 is 1, the LCD interprets
        // the remaining seven bits as a DDRAM address:
        //
        // bit:  7 6 5 4 3 2 1 0
        //       1 A A A A A A A
        //       ↑ └─ DDRAM address
        //       Set DDRAM Address command
        // Examples:
        // 1000_0000 → Row 1, column 0
        // 1000_0101 → Row 1, column 5
        // 1100_0000 → Row 2, column 0
        // 1100_0101 → Row 2, column 5
        let byte_to_select_row = 0b1000_0000 | (row_address as u8) | column as u8;
        Self::lcd_write_byte(byte_to_select_row, false);
    }

    // LCD STARTUP OVERVIEW
    //
    // The Pico does not directly control the LCD pins. It sends an 8-bit value over
    // I2C to the PCF8574 GPIO expander. Each bit controls one physical output:
    //
    //   PCF8574:  P7  P6  P5  P4  P3  P2  P1  P0
    //   LCD:      D7  D6  D5  D4  BL  EN  RW  RS
    //
    // NOTE: we don't control the D pins directly
    //
    // The LCD's "4-bit mode" refers only to LCD data pins D7-D4. It does not refer
    // to the 8-bit I2C value sent to the PCF8574. In 4-bit mode, every LCD byte is
    // sent in two parts:
    //
    //   first EN pulse  -> upper nibble on D7-D4
    //   second EN pulse -> lower nibble on D7-D4
    //
    // At power-up, the LCD may still expect 8-bit transfers. The datasheet startup
    // sequence synchronizes it and then switches it to 4-bit mode:
    //
    //   0x03, 0x03, 0x03 -> force the LCD into a known 8-bit state
    //   0x02             -> switch the LCD to 4-bit mode
    //
    // These are sent as individual nibbles because the LCD is not yet ready to
    // combine two nibbles into one byte. After 0x02, normal commands are sent with
    // lcd_write_byte(), which sends the upper nibble followed by the lower nibble.
    //
    // is_data = false keeps RS low, telling the LCD that these values are commands.
    // Each nibble is accepted on the falling edge of EN.
    unsafe fn startup_sequence() {
        unsafe {
            // NOTE:
            //  This places 0011 on LCD pins D7–D4. It tells the LCD to use 8-bit mode. We
            // send it three times so the LCD recognizes it regardless of its initial state.
            // 0011 → wait at least 4.1 ms
            // 0011 → wait at least 100 µs
            // 0011 → LCD is now synchronized in 8-bit mode
            // Required startup sequence. Initially the LCD expects 8-bit commands,
            // so these are sent as individual upper nibbles.
            Self::lcd_write_nibble(0x03, false);
            cortex_m::asm::delay(750_000); // 5 ms (minimum is 4.1 ms)

            Self::lcd_write_nibble(0x03, false);
            cortex_m::asm::delay(675_000); // 4.5 ms, matching Freenove's driver

            Self::lcd_write_nibble(0x03, false);
            cortex_m::asm::delay(22_500); // 150 µs

            Self::lcd_write_nibble(0x02, false); // Enter 4-bit mode

            // From here, commands are complete 8-bit values sent as two nibbles.
            Self::lcd_write_byte(0x28, false); // 4-bit mode, two lines, 5×8 characters
            Self::lcd_write_byte(0x08, false); // Display off
            Self::lcd_write_byte(0x01, false); // Clear display
            cortex_m::asm::delay(300_000); // Clear requires about 2 ms
            Self::lcd_write_byte(0x06, false); // Move cursor right after each character
            Self::lcd_write_byte(0x0c, false); // Display on, cursor off
            // ============ END sequence =========//
        }
    }
}
