#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    const PIN_MASK: u32 = 1 << 16;

    // NOTE: RP2350 reset controller
    // RESET       32 bits = 4 bytes → offset 0x00
    // WDSEL       32 bits = 4 bytes → offset 0x04
    // RESET_DONE  32 bits = 4 bytes → offset 0x08
    //                  one 32-bit value
    //         ┌───────────────────────────┐
    // 0x2000  │ byte 0 │ 8 bits           │
    // 0x2001  │ byte 1 │ 8 bits           │
    // 0x2002  │ byte 2 │ 8 bits           │
    // 0x2003  │ byte 3 │ 8 bits           │
    //         └───────────────────────────┘
    // This is the reset engine, we mask it with what we wanna reset
    const RESETS_RESET: *mut u32 = 0x4002_0000 as *mut u32;
    const RESETS_WDSEL: *mut u32 = 0x4002_0004 as *mut u32;
    const RESETS_RESET_DONE: *const u32 = 0x4002_0008 as *const u32; //read only

    // NOTE:   GPIO passes through two major pieces of hardware:
    // SIO signal
    //    ↓
    // IO_BANK0        selects what controls GP16
    //    ↓
    // PADS_BANK0      controls the physical/electrical pin
    //    ↓
    // GP16
    // Therefore, we need to reset IO_BANK0 and PADS_BANK0
    const IO_BANK0_BIT: u32 = 0x0000_0040 as u32;
    // const IO_BANK0_MASK: u32 = (1 << 6) as u32; // bit 6
    const PADS_BANK0_BIT: u32 = 0x0000_0200 as u32;
    // const PADS_BANK0_BIT: u32 = (1 << 9) as u32; // bit 9

    // NOTE: GP16 PIN constants

    unsafe {
        // 1. Reset IO_BANK0 and PADS_BANK0
        // Hardware power-on usually already do this but this is for learning
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | IO_BANK0_BIT | PADS_BANK0_BIT);

        // 2. Release the reset: clear IO_BANK0 and PADS_BANK0
        // clear bit use & !
        // we have to do this even if we don't do step 1
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset & !IO_BANK0_BIT & !PADS_BANK0_BIT);

        // 3. Wait till the reset it done
        // Aka the DONE bits will have those bit above
        while (core::ptr::read_volatile(RESETS_RESET_DONE) & (IO_BANK0_BIT | PADS_BANK0_BIT))
            != (IO_BANK0_BIT | PADS_BANK0_BIT)
        {}

        // 4. Configure GP16

        // IO BANK == //
        const IO_BANK0_BASE = 0x4002_8000_u32;
        const GPIO16_CTRL_OFFSET = 0x0000_0084_u32;
        //  GPIO16 CONTROL becomes:
        const GPIO16_CTRL: u32 = IO_BANK0_BASE + GPIO16_CTRL_OFFSET;

        // PAD === //
        const PAD_BANK0_BASE = 0x4003_8000_u32;
        const GPIO16_PAD_OFFSET = 0x0000_0044_u32;
    };

    loop {}
}
