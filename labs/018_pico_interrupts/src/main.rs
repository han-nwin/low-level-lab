#![no_std]
#![no_main]

mod lcd;
use lcd::{Lcd, LcdCol, LcdRow};
mod logging;
use core::sync::atomic::{AtomicBool, Ordering}; // use this to share state
use cortex_m as _;
use cortex_m_rt::entry;
use panic_halt as _;
use rp235x_hal::pac::{Interrupt, dma::TIMER0, interrupt};

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

// The goal is to measure real elapsed time without estimating CPU
// cycles.
//
//     clock source
//          |
//          v
//     timer tick generator
//          |
//          | one tick per microsecond
//          v
//     TIMER0 64-bit counter
//          |
//          | increments continuously in hardware
//          v
//     timer_ticks()
//          |
//          | reads the current counter value
//          v
//     delay_microsec_polling()
//     delay_microsec_intterupt()
//          |
//          | compares current time with a start time
//          v
//     application behavior
//          |
//          +--> compare polling based and interrupt based behavior
//          +--> USE the lcd to demonstrate the behavior
//
// 1. RESET
const RESETS_BASE: u32 = 0x4002_0000_u32;
const RESETS_RESET: *mut u32 = (RESETS_BASE) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32; // readonly
const TIMER0_RESET_MASK: u32 = (1 << 23) as u32; // GP0 and GP1

// 2. TIMER0 Tick config
const TIMER0_TICK_BASE: u32 = 0x4010_8000_u32;
const TIMER0_TICK_CTRL: *mut u32 = (TIMER0_TICK_BASE + 0x18) as *mut u32;
const TIMER0_TICK_CYCLES: *mut u32 = (TIMER0_TICK_BASE + 0x1c) as *mut u32;
const TIMER0_TICK_RUNNING_MASK: u32 = (1 << 1) as u32;
const TIMER0_TICK_ENABLE_MASK: u32 = (1 << 0) as u32;

// 3. TIMER ticks access
const TIMER0_BASE: u32 = 0x400b_0000_u32;
const TIMER0_TIME_HR: *mut u32 = (TIMER0_BASE + 0x08) as *mut u32; // readonly high bits
const TIMER0_TIME_LR: *mut u32 = (TIMER0_BASE + 0x0c) as *mut u32; // readonly low bits

// 4. ALARM for interrupt
const TIMER0_ALARM0: *mut u32 = (TIMER0_BASE + 0x10) as *mut u32;
const TIMER0_ARMED: *mut u32 = (TIMER0_BASE + 0x20) as *mut u32;
const TIMER0_INTR: *mut u32 = (TIMER0_BASE + 0x3c) as *mut u32; // raw interrupt
const TIMER0_INTE: *mut u32 = (TIMER0_BASE + 0x40) as *mut u32; // interrupt enable
const ALARM0_MASK: u32 = 1 << 0; // bit 0 is for ALARM0
const TICK_PER_MICROSEC: u64 = 2;
static ALARM_DONE: AtomicBool = AtomicBool::new(false); // Share state between interrupt handler
// and main

#[entry]
fn main() -> ! {
    // NOTE: logging:init set TIMER0_CYCLE = 12 -> 1 tick per micro sec
    // we'll reconfigure it below to a new value
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
                panic!("TIMER0 reset timeout")
            }
            core::hint::spin_loop();
        }

        // 2. Reconfigure the tick
        // 2.1 Disable the tick generator first
        core::ptr::write_volatile(TIMER0_TICK_CTRL, 0);
        // 2.2 Wait for it to stopped
        while (core::ptr::read_volatile(TIMER0_TICK_CTRL) & TIMER0_TICK_ENABLE_MASK) != 0 {
            core::hint::spin_loop();
        }
        // 2.3 Override the HAL's value of 12 to 6
        // This mean 6 cycle per tick -> 2 ticks per micro sec -> faster
        core::ptr::write_volatile(TIMER0_TICK_CYCLES, 6);
        // 2.4 Enable the tick generator again
        core::ptr::write_volatile(TIMER0_TICK_CTRL, TIMER0_TICK_ENABLE_MASK);
        // 2.5 wait till it's running
        while (core::ptr::read_volatile(TIMER0_TICK_CTRL) & TIMER0_TICK_RUNNING_MASK) == 0 {
            core::hint::spin_loop();
        }

        // 3. Read the timer ticks
        timer_ticks();

        // 4. Init the intterupt
        init_alarm0_interrupt();

        let mut lcd = Lcd::init();
        create_wave_glyphs(&mut lcd);
        let mut animation_step = 0;
        loop {
            logging::poll();

            // 1. Test with polling
            // animate_sine_wave(&mut lcd, animation_step);
            // animation_step = animation_step.wrapping_add(1);
            // cortex_m::asm::delay(50_000);
            // delay_microsec_polling(1_000_000);

            // 2. Test with intterupt
            // Arm the intterup then check for atomicbool state
            delay_microsec_interrupt(1_000_000);
            while !ALARM_DONE.load(Ordering::SeqCst) {
                animate_sine_wave(&mut lcd, animation_step);
                animation_step = animation_step.wrapping_add(1);
                cortex_m::asm::delay(50_000);
            }
        }
    }
}

fn timer_ticks() -> u64 {
    unsafe {
        // NOTE: always read low before high
        let low = core::ptr::read_volatile(TIMER0_TIME_LR);
        let high = core::ptr::read_volatile(TIMER0_TIME_HR);

        // Combine low and high into u64
        let timer_ticks: u64 = (high as u64) << 32 | (low as u64);
        timer_ticks
    }
}

fn delay_microsec_polling(micro_sec: u64) {
    let start = timer_ticks();
    let delay_tick = TICK_PER_MICROSEC * micro_sec; // 2 tick per microsec
    while timer_ticks().wrapping_sub(delay_tick) < start {
        logging::poll(); // keep usb alive
        core::hint::spin_loop();
    }
}

// NOTE:
// IRQ stands for Interrupt Request.
// It is a signal asking the processor to pause its current work and run an
// interrupt handler.
// Timer reaches ALARM0
//         ↓
// TIMER0 raises an Interrupt Request (IRQ)
//         ↓
// NVIC receives the request
//         ↓
// CPU pauses main
//         ↓
// TIMER0_IRQ_0 handler runs
//         ↓
// CPU returns to main
fn init_alarm0_interrupt() {
    unsafe {
        // Clear any old Alarm 0 interrupt
        core::ptr::write_volatile(TIMER0_INTR, ALARM0_MASK);

        // Allow Alarm 0 to produce a TIMER0 intterupt
        let interrupt_enable = core::ptr::read_volatile(TIMER0_INTE);
        core::ptr::write_volatile(TIMER0_INTE, interrupt_enable | ALARM0_MASK);

        // Clear an old pending processor interrupt, then reenable it
        cortex_m::peripheral::NVIC::unpend(Interrupt::TIMER0_IRQ_0);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIMER0_IRQ_0);
    }
}

// NOTE: Intterup handler for Timer0 Interrupt Request 0
#[interrupt]
fn TIMER0_IRQ_0() {
    unsafe {
        // INTR is write-one-to-clear
        // basically tell TIMER0 "I'll handler this interrupt"
        core::ptr::write_volatile(TIMER0_INTR, ALARM0_MASK);
    }
    // Then tell main the alarm has expired with the state
    ALARM_DONE.store(true, Ordering::SeqCst);
}

// This fn is to arm the alarm
fn delay_microsec_interrupt(micro_sec: u64) {
    let delay_ticks = TICK_PER_MICROSEC * micro_sec;

    assert!(
        delay_ticks < u32::MAX as u64,
        "Alarm 0 duration is to large (<u32)"
    );

    // Get TIMER0_TIME_LR value
    // it's the low 32 bit, simply cast it to u32
    let current_low = timer_ticks() as u32;
    let deadline = current_low.wrapping_add(delay_ticks as u32);

    unsafe {
        // Remove any stale completion before arming
        core::ptr::write_volatile(TIMER0_INTR, ALARM0_MASK);

        // Writing deadline to ALARM0 will set the deadline and arm it
        core::ptr::write_volatile(TIMER0_ALARM0, deadline);
    }
}

// Create eight custom LCD characters, each containing a five-pixel
// horizontal line at a different height. The sine animation selects
// one of these slots to draw each wave segment vertically.
// After this call, the LCD's eight custom-character slots contain:
//
// slot 0:  #####   slot 4:  .....
//          .....            .....
//          .....            .....
//          .....            .....
//          .....            #####
//          .....            .....
//          .....            .....
//          .....            .....
//
// slot 1:  .....   slot 5:  .....
//          #####            .....
//          .....            .....
//          .....            .....
//          .....            .....
//          .....            #####
//          .....            .....
//          .....            .....
//
// slot 2:  .....   slot 6:  .....
//          .....            .....
//          #####            .....
//          .....            .....
//          .....            .....
//          .....            .....
//          .....            #####
//          .....            .....
//
// slot 3:  .....   slot 7:  .....
//          .....            .....
//          .....            .....
//          #####            .....
//          .....            .....
//          .....            .....
//          .....            .....
//          .....            #####
//
// Writing byte 0 through 7 to a display cell selects the corresponding
// custom character from CGRAM.
fn create_wave_glyphs(lcd: &mut Lcd) {
    for height in 0..8 {
        let mut pixels = [0u8; 8];

        // One entire 5 pixels horizontal line
        pixels[height] = 0b11111;

        lcd.define_custom_char(height as u8, pixels);
    }
}

// The LCD has 2 row, each row has 8 vertical pixels -> 16 different heights 0:15
const SINE: [u8; 32] = [
    8, 9, 10, 12, 13, 14, 14, 15, 15, 15, 14, 14, 13, 12, 10, 9, 8, 6, 5, 3, 2, 1, 1, 0, 0, 0, 1,
    1, 2, 3, 5, 6,
];
const LCD_COLUMNS: [LcdCol; 16] = [
    LcdCol::Col1,
    LcdCol::Col2,
    LcdCol::Col3,
    LcdCol::Col4,
    LcdCol::Col5,
    LcdCol::Col6,
    LcdCol::Col7,
    LcdCol::Col8,
    LcdCol::Col9,
    LcdCol::Col10,
    LcdCol::Col11,
    LcdCol::Col12,
    LcdCol::Col13,
    LcdCol::Col14,
    LcdCol::Col15,
    LcdCol::Col16,
];

fn animate_sine_wave(lcd: &mut Lcd, frame: usize) {
    for column_index in 0..16 {
        let column = LCD_COLUMNS[column_index];

        // Erase the previous pixels in this column.
        lcd.write_byte_at(LcdRow::Row1, column, b' ');
        lcd.write_byte_at(LcdRow::Row2, column, b' ');

        // Multiplying the column by 2 fits one complete wave
        // across the 16-character display.
        let phase = (column_index * 2 + frame) % SINE.len();
        let height = SINE[phase];

        if height < 8 {
            // Top half of the display.
            lcd.write_byte_at(LcdRow::Row1, column, height);
        } else {
            // Bottom half of the display.
            lcd.write_byte_at(LcdRow::Row2, column, height - 8);
        }
    }
}
