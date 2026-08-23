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
//     timer_us()
//          |
//          | reads the current counter value
//          v
//     delay_us()
//          |
//          | compares current time with a start time
//          v
//     application behavior
//          |
//          +--> blink LED
//          +--> measure execution time
//          +--> later: schedule timer alarms and interrupts
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

        loop {
            logging::poll();
            let start = timer_ticks();
            cortex_m::asm::delay(100_000); // poll every 100k iterations, around 3 cycles
            let end = timer_ticks();

            // NOTE: Since our clk_sys is 150MHZ
            // asm::delay(100_000) performs about 100k loop iterations.
            // On this Cortex-M33, each iteration takes about 3 CPU cycles.
            // 100k * 3 = 300k CPU cycles
            // 300k / 150MHz = 2,000 microseconds
            // 2,000 us * 2 timer ticks/us = about 4,000 ticks
            logln!("Elapsed time: {} ticks", end - start); // should print 4000

            logln!("Start test delay_microsec. Count 1 2 3, it should delay 3 sec...");
            delay_microsec(1_000_000);
            logln!("1");
            delay_microsec(1_000_000);
            logln!("2");
            delay_microsec(1_000_000);
            logln!("3");
            logln!("DONE");
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

fn delay_microsec(micro_sec: u64) {
    let start = timer_ticks();
    let delay_tick = 2 * micro_sec; // 2 tick per microsec
    while timer_ticks().wrapping_sub(delay_tick) < start {
        logging::poll(); // keep usb alive
        core::hint::spin_loop();
    }
}
