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

    // Since at start, CPU may already use PLL
    // We need to reconfigure from downstream → upstream
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

    unsafe {
        //enable xosc and wait for stable
        enable_xosc_stable();
        // route clock sys to clock ref
        route_clk_sys_to_clk_ref();
        // Route lock ref to ROSC
        route_clk_ref_to_rosc();

        // Reset PLL
        reset_pll_sys();
        // config and lock PLL
        config_and_lock_pll();
        // route clk_ref to XOSC
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
    // CTRL register
    // 31        24 23                  12 11                   0
    // +-----------+----------------------+----------------------+
    // | Reserved  | ENABLE (12 bits)     | FREQ_RANGE (12 bits) |
    // +-----------+----------------------+----------------------+
    //               0xFAB                 0xaa0
    const XOSC_ENABLE_BIT: u32 = (0xfab << 12) as u32; // has to be from 23-12 bits
    const XOSC_FREQ_RANGE: u32 = 0xaa0_u32; // 1_15MHZ, bits 0-11

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

unsafe fn route_clk_sys_to_clk_ref() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_SYS_CTRL: *mut u32 = (CLOCK_BASE + 0x3c) as *mut u32;
    const CLOCK_SYS_SELECTED: *const u32 = (CLOCK_BASE + 0x44) as *const u32;
    const CLOCK_REF_CTRL: *mut u32 = (CLOCK_BASE + 0x30) as *mut u32;
    const CLOCK_REF_SELECTED: *const u32 = (CLOCK_BASE + 0x38) as *const u32;

    // CLK_SYS_CTRL.SRC is bit 0:
    // 0 = CLK_REF
    // 1 = CLK_SYS_AUX
    const CLOCK_SYS_SRC_VALUE: u32 = 0; // set this to route to clk_ref

    // Route clk sys to clk ref
    unsafe {
        let clk_ctr = core::ptr::read_volatile(CLOCK_SYS_CTRL);
        // unset the last bit with & !0b1 then set it to new value
        let new_clk_ctr = (clk_ctr & !0b1_u32) | CLOCK_SYS_SRC_VALUE;
        core::ptr::write_volatile(CLOCK_SYS_CTRL, new_clk_ctr);

        // use CLOCK_SYS_SELECTED to see if switching is completed
        // 01 → CLOCK_REF
        // 10 → CLKSRC_CLK_SYS_AUX
        // 00 → switching in progress
        // use & 0b11 to ignore the other 30 bit
        while core::ptr::read_volatile(CLOCK_SYS_SELECTED) & 0b11 != 0b01 {
            core::hint::spin_loop();
        }
    }
}

unsafe fn route_clk_ref_to_rosc() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_REF_CTRL: *mut u32 = (CLOCK_BASE + 0x30) as *mut u32;
    const CLOCK_REF_SELECTED: *const u32 = (CLOCK_BASE + 0x38) as *const u32;

    // 0x0 → ROSC_CLKSRC_PH  = 0b00
    // 0x1 → CLKSRC_CLK_REF_AUX = 0b01
    // 0x2 → XOSC_CLKSRC = 0b10
    // 0x3 → LPOSC_CLKSRC = 0b11
    const CLOCK_REF_SRC_VALUE: u32 = 0b00_u32; // set this to route to ROSC

    // route clk_ref to ROSC
    unsafe {
        let clk_ctr = core::ptr::read_volatile(CLOCK_REF_CTRL);
        // unset the last 2 bit and set it to 0
        // can just be unset but here for later if we need to use different mask
        let new_clk_ctr = (clk_ctr & !0b11_u32) | CLOCK_REF_SRC_VALUE;
        core::ptr::write_volatile(CLOCK_REF_CTRL, new_clk_ctr);

        // use CLOCK_REF_SELECTED to see if switching is completed
        // 0b0001 → ROSC_CLKSRC_PH
        // 0b0010 → CLKSRC_CLK_REF_AUX
        // 0b0100 → XOSC_CLKSRC
        // 0b1000 → LPOSC_CLKSRC
        while core::ptr::read_volatile(CLOCK_REF_SELECTED) & 0b1111 != 0b0001 {
            core::hint::spin_loop();
        }
    }
}

unsafe fn reset_pll_sys() {
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

unsafe fn config_and_lock_pll() {
    // 12 MHz crystal
    // Reference Divider (REFDIV)
    //        |
    //        v
    // Phase Locked Loop
    //        |
    //        v
    // Feedback Divider (FBDIV)
    //        |
    //        v
    // VCO
    //        |
    //        v
    // POSTDIV1
    //        |
    // POSTDIV2
    //        |
    //        v
    // PLL Output
    //
    // The frequency equation is:
    // VCO = XOSC / REFDIV x FBDIV
    // PLL_OUT = VCO / POSTDIV1 / POSTDIV2
    // XOSC      = 12 MHz
    // REFDIV    = 1
    // FBDIV     = 125
    // -> VCO = 1500MHz
    // POSTDIV1  = 5
    // POSTDIV2  = 2
    // Output = 12*125 /5 /2 = 150Mhz
    const PLL_SYS_BASE: u32 = 0x4005_0000_u32;
    const PLL_SYS_CS: *mut u32 = (PLL_SYS_BASE + 0x00_u32) as *mut u32; // has refdiv and also lock
    const PLL_SYS_PWR: *mut u32 = (PLL_SYS_BASE + 0x04_u32) as *mut u32; // for power
    const PLL_FBDIV_INT: *mut u32 = (PLL_SYS_BASE + 0x08_u32) as *mut u32; // for fbdiv
    const PLL_PRIM: *mut u32 = (PLL_SYS_BASE + 0x0c_u32) as *mut u32; // for postdiv1 and postdiv2

    unsafe {
        // 1. Power down PLL_SYS |
        //         v
        // 2. Configure PLL parameters
        //         |
        //         +--> REFDIV = 1
        //         |
        //         +--> FBDIV = 125
        //         |
        //         v
        // 3. Power up PLL_SYS
        //         |
        //         v
        // 4. Wait for PLL LOCK
        //         |
        //         v
        // 5. Configure output divider
        //         |
        //         +--> POSTDIV1 = 5
        //         |
        //         +--> POSTDIV2 = 2
        // 6. Enable PLL output divier. Since it got reset when we reset

        // 1. Power down PLL_SYS
        let pll_sys_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        // bit 0 is PD: Power down; bit 5 is VCOPD: VCP power down
        let new_pll_sys_pwr = (pll_sys_pwr & !(0b100001)) | (0b100001);
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_sys_pwr);

        // 2. Configure PLL parameters
        // Set REFDIV = 1
        let pll_sys_cs = core::ptr::read_volatile(PLL_SYS_CS);
        let new_pll_sys_cs = (pll_sys_cs & !(0b11111) ) | 0b00001 // unset first 5 bit then set it to 0b00001
        core::ptr::write_volatile(PLL_SYS_CS, new_pll_sys_cs);

        // Set FBDIV = 125
        let fbdiv = core::ptr::read_volatile(PLL_FBDIV_INT);
        let bin_125 = 125_u32;
        let new_fbdiv = (fbdiv & !((1<<12) -1)) | bin_125;
        core::ptr::write_volatile(PLL_FBDIV_INT, new_fbdiv);

        // 3. Power up
        let pll_sys_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        let new_pll_sys_pwr = (pll_sys_pwr & !(0b100001)); // clear bit 0 and 5
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_sys_pwr);

        // 4. wait for pll lock
        while core::ptr::read_volatile(PLL_SYS_CS) & (1 << 31) == 0 {
            core::hint::spin_loop();
        }

        // 5. Output divider
        let pll_prim = core::ptr::read_volatile(PLL_PRIM);
        let new_pll_prim = (pll_prim & !((0b111) << 16)) | (0b101 << 16); // postdiv1 = 5
        let new_pll_prim = (new_pll_prim & !((0b111) << 12)) | (0b010 << 12); // postdiv2 = 2
        core::ptr::write_volatile(PLL_PRIM, new_pll_prim);

        // 6. Enable PLL output divider
        let pll_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        let new_pll_pwr = pll_pwr & !(0b1000); // clear bit 3
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_pwr);
    }
}

// TODO:
unsafe fn route_clk_ref_to_xosc() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_REF_CONTROL: *mut u32 = (CLOCK_BASE + 0x30) as *mut u32;
    const CLOCK_REF_SELECTED: *const u32 = (CLOCK_BASE + 0x38) as *const u32;
}

// TODO:
unsafe fn route_clk_sys_to_pll_sys() {}

// TODO:
unsafe fn route clk_prei_to_clk_sys() {}
