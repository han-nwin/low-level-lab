#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

// NOTE: use static so that it's guaranteed to have a real object in memory
// A static has identity and can be addressed
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

#[entry]
fn main() -> ! {
    // Crystal Oscillator Config:
    // DataSheet: 8.1.2.3
    // NOTE: The setup flow
    // Crystal → XOSC → PLL_SYS → clk_sys → CPU
    // enable_xosc
    //     ↓
    // clk_sys → clk_ref: Since the cpu may be using clk_syc, redirect it's source to clk_ref
    //     ↓
    // clk_ref → ROSC: then route clk_ref to ROSC (ring oscillator clock source)
    //     ↓
    // reset/release PLL_SYS
    //     ↓
    // configure and lock PLL_SYS
    //     ↓
    // clk_ref → XOSC
    //     ↓
    // clk_sys → PLL_SYS
    //     ↓
    // clk_peri → clk_sys
    const CLOCK_BASE: *mut u32 = 0x4001_0000 as *mut u32;

    unsafe {
        //enable xosc and wait for stable
        enable_xosc_stable();

        // Reset PLL
        reset_pll();
    }
    loop {}
}

unsafe fn enable_xosc_stable() {
    // The XOSC config
    const XOSC_BASE: u32 = 0x4004_8000 as u32;
    const XOSC_CTRL: *mut u32 = (XOSC_BASE + 0x00) as *mut u32;
    const XOSC_STATUS: *const u32 = (XOSC_BASE + 0x04) as *const u32; //readonly
    const XOSC_STARTUP: *mut u32 = (XOSC_BASE + 0x0c) as *mut u32;
    // Table 599
    const XOSC_STABLE_BIT: u32 = (1 << 31) as u32;
    const XOSC_ENABLE_BIT: u32 = (0xfab << 12) as u32;
    const XOSC_FREQ_RANGE: u32 = 0xaa0_u32; // 1_15MHZ

    unsafe {
        // Datasheet table 588: CTRL Register
        // NOTE: Startup delay
        // Because the crystal need some time to start, let's wait 6ms

        // Datasheet: XOSC_STARTUP register
        // We want startup_delay = 6ms
        // Startup delay = DELAY x 256 x xtal_period
        // For 12 MHz crystal (RP235x)
        //   282 x 256 x (1 / 12 MHz) = 6 ms startup time
        // Gives the crystal oscillator time to stabilize before enabling use.
        core::ptr::write_volatile(XOSC_STARTUP, 282); // give config for XOSC when it start

        // Initiate the enable and select the frequency range
        core::ptr::write_volatile(XOSC_CTRL, XOSC_ENABLE_BIT | XOSC_FREQ_RANGE);

        // wait for stable
        while (core::ptr::read_volatile(XOSC_STATUS) & XOSC_STABLE_BIT) == 0 {
            core::hint::spin_loop();
        }
    }
    // Configure XOSC
    //       |
    //       |
    //       +--> XOSC_STARTUP
    //       |        "How long should startup take?"
    //       |
    //       +--> XOSC_CTRL
    //                "What crystal range? Enable oscillator?"
    //        ↓
    // XOSC begins running
    //        ↓
    // Hardware waits internally according to STARTUP.DELAY
    //        ↓
    // XOSC_STATUS.STABLE = 1
}

unsafe fn reset_pll() {
    const RESETS_BASE: u32 = 0x4002_0000 as u32;
    const RESETS_RESET: *mut u32 = (RESETS_BASE + 0x00) as *mut u32;
    const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32; // readonly
    const PLL_SYS_RESET_OFFSET: u32 = (1 << 14) as u32;

    unsafe {
        // Reset PLL
        // 1. Reset the PLL
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | PLL_SYS_RESET_OFFSET);
        // 2. Release the PLL
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !PLL_SYS_RESET_OFFSET);
        // 3. Wait for the reset status is done
        let mut time_out = 1_000_000;
        while (core::ptr::read_volatile(RESETS_RESET_DONE) & PLL_SYS_RESET_OFFSET)
            != PLL_SYS_RESET_OFFSET
        {
            time_out -= 1;
            if time_out <= 0 {
                panic!("PLL reset timeout")
            }
            core::hint::spin_loop();
        }
    }
}
