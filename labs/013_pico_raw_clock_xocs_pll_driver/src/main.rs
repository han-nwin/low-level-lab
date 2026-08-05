#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use defmt_rtt as _;
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

// NOTE: This lab intentionally overclocks clk_sys to 300 MHz. RP2350 is rated
// for operation up to 150 MHz, so 300 MHz is an experiment, not a guaranteed
// production configuration.
//
// Clock tree looks like
//           ROSC          XOSC
//             |             |
//             +------+------+
//                    |
//                    v
//               +-----------+
//               |  clk_ref  |
//               +-----------+
//                    |
//         +----------+----------+
//         |                     |
//         v                     v
//     PLL_SYS              Other logic
//         |
//         v
//    1500 MHz VCO
//         |
//     POSTDIV1/2
//         |
//         v
//    300 MHz PLL output
//         |
//         v
//    +-----------+     +-------------+
//    | clk_sys   |---->| CLK_SYS_DIV |----> final clk_sys
//    | source mux|     | divide by 1 |
//    +-----------+     +-------------+
//                              |
//                        +-----+-----+
//                        |           |
//                        v           v
//                       CPU     +-----------+
//                               | clk_peri  |----> CLK_PERI_DIV ----> peripheral clock
//                               | source mux|
//                               +-----------+
//                                    |
//                         +----------+----------+
//                         |    |     |     |    |
//                       UART  SPI   I2C   PWM  DMA

#[entry]
fn main() -> ! {
    // Crystal Oscillator Config:
    // DataSheet: 8.1.2.3
    // NOTE: The setup flow
    // Crystal → XOSC → PLL_SYS → clk_sys mux → CLK_SYS_DIV → CPU
    //
    // IMPORTANT: The PLL configuration ends at the PLL_SYS output. Even when
    // PLL_SYS is 300 MHz, the separate clock-slice divider can divide it again:
    //
    // clk_sys = PLL_SYS output / CLK_SYS_DIV
    //         = 300 MHz / 1
    //         = 300 MHz
    //
    // CLK_SYS_DIV uses fixed-point encoding: divide-by-1 is 1 << 16. The
    // CLK_REF_DIV and CLK_PERI_DIV registers are separate downstream dividers;
    // they are not the PLL's REFDIV, POSTDIV1, or POSTDIV2 fields.

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
    // CLK_REF_DIV = 1, then clk_ref → XOSC
    //     ↓
    // CLK_SYS_DIV = 1, then clk_sys → PLL_SYS
    //     ↓
    // CLK_PERI_DIV = 1, then enable clk_peri → clk_sys

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
        route_clk_ref_to_xosc();
        // route clk_sys to pll_sys
        route_clk_sys_to_pll_sys();
        // route clk_peri to clk_sys
        route_clk_peri_to_clk_sys();

        // TESTING
        let measured_sys_khz = measure_clk_sys_khz();
        defmt::info!("measured clk_sys = {} kHz", measured_sys_khz);

        init_gpio16_output();

        // A blinking LED means FC0 measured clk_sys within 1% of the expected
        // frequency. If the measurement is outside that range, leave it off.

        // Change this when intentionally testing a different clk_sys frequency.
        const EXPECTED_SYS_KHZ: u32 = 300_000;

        let tolerance_khz = EXPECTED_SYS_KHZ / 100;
        if measured_sys_khz.abs_diff(EXPECTED_SYS_KHZ) <= tolerance_khz {
            const SIO_BASE: u32 = 0xd000_0000;
            const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x18) as *mut u32;
            const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x20) as *mut u32;
            const GPIO16: u32 = 1 << 16;

            loop {
                core::ptr::write_volatile(GPIO_OUT_SET, GPIO16);
                // NOTE: At the intentional 300 MHz overclock, 75 million cycles is about
                // 0.25 seconds. At the rated 150 MHz it would be about 0.5 seconds.
                cortex_m::asm::delay(75_000_000);

                core::ptr::write_volatile(GPIO_OUT_CLR, GPIO16);

                // Another approximately 0.25 seconds at 300 MHz.
                cortex_m::asm::delay(75_000_000);
            }
        }

        loop {
            core::hint::spin_loop();
            defmt::info!("The configured clock is outside of tolerance range!! SOMETHING WRONG");
        }
    }
}

unsafe fn measure_clk_sys_khz() -> u32 {
    const CLOCK_BASE: u32 = 0x4001_0000;
    const FC0_REF_KHZ: *mut u32 = (CLOCK_BASE + 0x8c) as *mut u32;
    const FC0_MIN_KHZ: *mut u32 = (CLOCK_BASE + 0x90) as *mut u32;
    const FC0_MAX_KHZ: *mut u32 = (CLOCK_BASE + 0x94) as *mut u32;
    const FC0_INTERVAL: *mut u32 = (CLOCK_BASE + 0x9c) as *mut u32;
    const FC0_SRC: *mut u32 = (CLOCK_BASE + 0xa0) as *mut u32;
    const FC0_STATUS: *const u32 = (CLOCK_BASE + 0xa4) as *const u32;
    const FC0_RESULT: *const u32 = (CLOCK_BASE + 0xa8) as *const u32;

    const STATUS_RUNNING: u32 = 1 << 8;
    const STATUS_DONE: u32 = 1 << 4;
    const SRC_CLK_SYS: u32 = 0x09;

    unsafe {
        // Do not reconfigure FC0 while an earlier measurement is running.
        while core::ptr::read_volatile(FC0_STATUS) & STATUS_RUNNING != 0 {
            core::hint::spin_loop();
        }

        // FC0 uses clk_ref as its reference. clk_ref is the 12 MHz XOSC after
        // route_clk_ref_to_xosc(), so tell FC0 that the reference is 12,000 kHz.
        core::ptr::write_volatile(FC0_REF_KHZ, 12_000);
        core::ptr::write_volatile(FC0_INTERVAL, 10);

        // We compare the result in software, so disable the hardware pass range.
        core::ptr::write_volatile(FC0_MIN_KHZ, 0);
        core::ptr::write_volatile(FC0_MAX_KHZ, 0x01ff_ffff);

        // Writing the source starts a measurement. 0x09 selects clk_sys.
        core::ptr::write_volatile(FC0_SRC, SRC_CLK_SYS);

        while core::ptr::read_volatile(FC0_STATUS) & STATUS_DONE == 0 {
            core::hint::spin_loop();
        }

        // RESULT bits 29:5 contain whole kHz; bits 4:0 are the fraction.
        core::ptr::read_volatile(FC0_RESULT) >> 5
    }
}

unsafe fn enable_xosc_stable() {
    // The XOSC config
    const XOSC_BASE: u32 = 0x4004_8000_u32;
    const XOSC_CTRL: *mut u32 = (XOSC_BASE + 0x00_u32) as *mut u32;
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

    // CLK_SYS_CTRL.SRC is bit 0:
    // 0 = CLK_REF
    // 1 = CLK_SYS_AUX
    const CLOCK_SYS_SRC_VALUE: u32 = 0; // set this to route to clk_ref

    // Route clk sys to clk ref
    //                  +-----------+
    // clk_ref -------->|           |
    //                  | Glitchless|-----> clk_sys
    // AUX mux ---xx--->|    MUX    |
    //                  +-----------+
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
    const RESETS_BASE: u32 = 0x4002_0000_u32;
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
    // PLL Output (not necessarily the final CPU clock yet)
    //
    // The frequency equation is:
    // VCO = XOSC / REFDIV x FBDIV
    // PLL_OUT = VCO / POSTDIV1 / POSTDIV2
    // XOSC      = 12 MHz
    // REFDIV    = 1
    // FBDIV     = 125
    // -> VCO = 1500MHz
    // POSTDIV1  = 5
    // POSTDIV2  = 1
    // Output = 12*125 /5 /1 = 300 MHz
    //
    // This function configures only the dividers inside PLL_SYS. After this
    // function returns, the 300 MHz PLL output still travels through:
    //
    // NOTE: PLL_SYS output -> CLK_SYS source mux -> CLK_SYS_DIV -> CPU/system fabric
    //     300 MHz                                  / 1  -> 300 MHz
    //
    // NOTE: If CLK_SYS_DIV were 2, the PLL would still be 300 MHz but the CPU would
    // receive 150 MHz. That is why the final clock-routing step must also
    // make CLK_SYS_DIV explicitly divide by 1.
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
        //         +--> POSTDIV2 = 1
        // 6. Enable PLL output divier. Since it got reset when we reset

        // 1. Power down PLL_SYS
        let pll_sys_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        // bit 0 is PD: Power down; bit 5 is VCOPD: VCP power down
        let new_pll_sys_pwr = (pll_sys_pwr & !(0b100001)) | (0b100001);
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_sys_pwr);

        // 2. Configure PLL parameters
        // Set REFDIV = 1
        let pll_sys_cs = core::ptr::read_volatile(PLL_SYS_CS);
        let new_pll_sys_cs = (pll_sys_cs & !(0b11_1111)) | 0b00_0001; // unset first 6 bit then set it to 0b00_0001
        core::ptr::write_volatile(PLL_SYS_CS, new_pll_sys_cs);

        // Set FBDIV = 125
        let fbdiv = core::ptr::read_volatile(PLL_FBDIV_INT);
        let bin_125 = 125_u32;
        let new_fbdiv = (fbdiv & !((1 << 12) - 1)) | bin_125;
        core::ptr::write_volatile(PLL_FBDIV_INT, new_fbdiv);

        // 3. Power up
        let pll_sys_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        let new_pll_sys_pwr = pll_sys_pwr & !(0b100001); // clear bit 0 and 5
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_sys_pwr);

        // 4. wait for pll lock
        while core::ptr::read_volatile(PLL_SYS_CS) & (1 << 31) == 0 {
            core::hint::spin_loop();
        }

        // 5. Output divider
        let pll_prim = core::ptr::read_volatile(PLL_PRIM);
        let new_pll_prim = (pll_prim & !((0b111) << 16)) | (0b101 << 16); // postdiv1 = 5
        let new_pll_prim = (new_pll_prim & !((0b111) << 12)) | (0b001 << 12); // postdiv2 = 1
        core::ptr::write_volatile(PLL_PRIM, new_pll_prim);

        // 6. Enable PLL output divider
        // POSTDIVPD resets high, so the post-divider output is disabled.
        // Clear it after configuring POSTDIV1/2.
        let pll_pwr = core::ptr::read_volatile(PLL_SYS_PWR);
        let new_pll_pwr = pll_pwr & !(0b1000); // clear bit 3
        core::ptr::write_volatile(PLL_SYS_PWR, new_pll_pwr);
    }
}

unsafe fn route_clk_ref_to_xosc() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_REF_CTRL: *mut u32 = (CLOCK_BASE + 0x30) as *mut u32;
    const CLOCK_REF_SELECTED: *const u32 = (CLOCK_BASE + 0x38) as *const u32;
    const CLOCK_REF_DIV: *mut u32 = (CLOCK_BASE + 0x34) as *mut u32;

    //1:0 SRC: Selects the clock source glitchlessly, can be changed on-the-fly
    // Enumerated values:
    // 0x0 ROSC_CLKSRC_PH
    // 0x1 CLKSRC_CLK_REF_AUX
    // 0x2 XOSC_CLKSRC
    // 0x3 LPOSC_CLKSRC
    const CLOCK_REF_SRC_VALUE: u32 = 0x2; // -> XOSC

    unsafe {
        // Set clk_ref_div to 1
        core::ptr::write_volatile(CLOCK_REF_DIV, 1 << 16);

        // Choose xosc
        let clk_ctr = core::ptr::read_volatile(CLOCK_REF_CTRL);
        let new_clk_ctr = (clk_ctr & !0b11_u32) | CLOCK_REF_SRC_VALUE;
        core::ptr::write_volatile(CLOCK_REF_CTRL, new_clk_ctr);

        // wait till it set
        // xosc is bit 2
        while (core::ptr::read_volatile(CLOCK_REF_SELECTED) & (0b1111)) != 0b0100 {
            core::hint::spin_loop();
        }
    }
}

unsafe fn route_clk_sys_to_pll_sys() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_SYS_CTRL: *mut u32 = (CLOCK_BASE + 0x3c) as *mut u32;
    const CLOCK_SYS_SELECTED: *const u32 = (CLOCK_BASE + 0x44) as *const u32;
    // This is a separate divider after the clk_sys source mux. Its 16.16
    // fixed-point encoding means divide-by-1 is written as 1 << 16.
    const CLOCK_SYS_DIV: *mut u32 = (CLOCK_BASE + 0x40) as *mut u32;

    const CLOCK_SYS_AUXSRC_VALUE: u32 = 0b000;

    unsafe {
        //
        // PLL_SYS output (300 MHz, intentionally overclocked)
        //     |
        //     v
        // +-----------+
        // |  AUX MUX  |----+
        // +-----------+    |
        //                  v
        //             +-----------+
        // clk_ref --->| Glitchless|
        //             |    MUX    |
        //             +-----+-----+
        //                   |
        //                   v
        //             +-------------+
        //             | CLK_SYS_DIV |  divide-by-1 passes all 300 MHz through
        //             +-------------+
        //                    |
        //                    v
        //             CPU/system fabric
        //
        // Selecting PLL_SYS below only chooses the mux input. CLK_SYS_DIV is a
        // later, independent divider and can reduce the final CPU clock again.

        // Set clk_sys_div to 1
        core::ptr::write_volatile(CLOCK_SYS_DIV, 1 << 16);

        // configure AUX source -> PLL_SYS
        let clk_ctr = core::ptr::read_volatile(CLOCK_SYS_CTRL);
        let new_clk_ctr = (clk_ctr & !(0b111 << 5)) | (CLOCK_SYS_AUXSRC_VALUE << 5);
        core::ptr::write_volatile(CLOCK_SYS_CTRL, new_clk_ctr);

        // switch SRC -> AUX
        let clk_ctr = core::ptr::read_volatile(CLOCK_SYS_CTRL);
        let new_clk_ctr = (clk_ctr & !(0b1)) | 0b1;
        core::ptr::write_volatile(CLOCK_SYS_CTRL, new_clk_ctr);

        // wait till it selected
        while (core::ptr::read_volatile(CLOCK_SYS_SELECTED) & (0b11)) != 0b10 {
            core::hint::spin_loop();
        }
    }
}

unsafe fn route_clk_peri_to_clk_sys() {
    const CLOCK_BASE: u32 = 0x4001_0000_u32;
    const CLOCK_PERI_CTRL: *mut u32 = (CLOCK_BASE + 0x48) as *mut u32;
    const CLOCK_PERI_DIV: *mut u32 = (CLOCK_BASE + 0x4c) as *mut u32;

    const CLOCK_PERI_CTRL_AUXSRC_VALUE: u32 = 0x0_u32;

    unsafe {
        // Disable
        let clk_ctr = core::ptr::read_volatile(CLOCK_PERI_CTRL);
        let new_clk_ctr = clk_ctr & !(1 << 11);
        core::ptr::write_volatile(CLOCK_PERI_CTRL, new_clk_ctr);

        // wait for Hardware to say it's diabled
        while core::ptr::read_volatile(CLOCK_PERI_CTRL) & (1 << 28) != 0 {
            core::hint::spin_loop();
        }

        // Set clk_peri_div to 1
        core::ptr::write_volatile(CLOCK_PERI_DIV, 1 << 16);

        // Route to clk_sys while disabled
        let clk_ctr = core::ptr::read_volatile(CLOCK_PERI_CTRL);
        let new_clk_ctr = (clk_ctr & !(0b111 << 5)) | (CLOCK_PERI_CTRL_AUXSRC_VALUE << 5);
        core::ptr::write_volatile(CLOCK_PERI_CTRL, new_clk_ctr);

        // enable
        let clk_ctr = core::ptr::read_volatile(CLOCK_PERI_CTRL);
        let new_clk_ctr = (clk_ctr & !(1 << 11)) | (1 << 11);
        core::ptr::write_volatile(CLOCK_PERI_CTRL, new_clk_ctr);

        // wait for Hardware to say it's enabled
        while core::ptr::read_volatile(CLOCK_PERI_CTRL) & (1 << 28) == 0 {
            core::hint::spin_loop();
        }

        // There's nothing to wait since there's only one mux controlled by AUXSRC
        //            +---------+
        // AUXSRC --->|  MUX    |----> clk_peri
        //            +---------+
    }
}

unsafe fn init_gpio16_output() {
    const RESETS_BASE: u32 = 0x4002_0000;
    const RESETS_RESET: *mut u32 = (RESETS_BASE + 0x00) as *mut u32;
    const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32;
    const IO_BANK0_RESET: u32 = 1 << 6;
    const PADS_BANK0_RESET: u32 = 1 << 9;
    const GPIO_RESETS: u32 = IO_BANK0_RESET | PADS_BANK0_RESET;

    const IO_BANK0_BASE: u32 = 0x4002_8000;
    const GPIO16_CTRL: *mut u32 = (IO_BANK0_BASE + 0x84) as *mut u32;

    const PADS_BANK0_BASE: u32 = 0x4003_8000;
    const GPIO16_PAD: *mut u32 = (PADS_BANK0_BASE + 0x44) as *mut u32;
    const PAD_ISO: u32 = 1 << 8;
    const PAD_OD: u32 = 1 << 7;
    const PAD_IE: u32 = 1 << 6;

    const SIO_BASE: u32 = 0xd000_0000;
    const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x20) as *mut u32;
    const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x38) as *mut u32;

    const GPIO16: u32 = 1 << 16;
    const FUNCSEL_SIO: u32 = 5;

    unsafe {
        // Match lab 011: reset and release both GPIO control blocks, then wait
        // until the reset controller confirms that they are usable.
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | GPIO_RESETS);

        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !GPIO_RESETS);

        while core::ptr::read_volatile(RESETS_RESET_DONE) & GPIO_RESETS != GPIO_RESETS {
            core::hint::spin_loop();
        }

        // Select SIO as GPIO16's controller.
        let ctrl = core::ptr::read_volatile(GPIO16_CTRL);
        core::ptr::write_volatile(GPIO16_CTRL, (ctrl & !0x1f) | FUNCSEL_SIO);

        // Match lab 011's physical pad setup: remove isolation, enable output,
        // and leave the input buffer enabled.
        let pad = core::ptr::read_volatile(GPIO16_PAD);
        let pad = (pad & !(PAD_ISO | PAD_OD)) | PAD_IE;
        core::ptr::write_volatile(GPIO16_PAD, pad);

        core::ptr::write_volatile(GPIO_OUT_CLR, GPIO16);
        core::ptr::write_volatile(GPIO_OE_SET, GPIO16);
    }
}
