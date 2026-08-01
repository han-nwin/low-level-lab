#![no_std]
#![no_main]

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

#[entry]
fn main() -> ! {
    const LED_PIN_MASK: u32 = 1 << 16;
    const BUTTON_PIN_MASK: u32 = 1 << 15;

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

    // NOTE: GPIO passes through two major pieces of hardware:
    // SIO signal
    //    ↓
    // IO_BANK0        selects what controls GP16
    //    ↓
    // PADS_BANK0      controls the physical/electrical pin
    //    ↓
    // GP16
    // Therefore, we need to reset IO_BANK0 and PADS_BANK0
    const IO_BANK0_BIT: u32 = 0x0000_0040_u32;
    // const IO_BANK0_MASK: u32 = (1 << 6) as u32; // bit 6
    const PADS_BANK0_BIT: u32 = 0x0000_0200_u32;
    // const PADS_BANK0_BIT: u32 = (1 << 9) as u32; // bit 9

    // NOTE: Route to GPIO 16
    // SIO -> IO_BANK -> PADS_BANK -> GPIO16
    const SIO_BASE: u32 = 0xD000_0000_u32;
    const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
    const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;

    // now define SIO
    const GPIO_OE_SET_OFFSET: u32 = 0x038_u32; // enable the pin as output, can use GPIO_OE but it will reset other pins
    const GPIO_OUT_SET_OFFSET: u32 = 0x018_u32; // use this bit to set it high
    const GPIO_OUT_CLR_OFFSET: u32 = 0x020_u32; // this use to clear the bit -> set low
    const GPIO_OE_SET: *mut u32 = (SIO_BASE + GPIO_OE_SET_OFFSET) as *mut u32;
    const GPIO_OUT_SET: *mut u32 = (SIO_BASE + GPIO_OUT_SET_OFFSET) as *mut u32;
    const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + GPIO_OUT_CLR_OFFSET) as *mut u32;
    // now tell IO_BANK what peripheral connected to the pin (it's the GPIO16)
    const GPIO16_STATUS_OFFSET: u32 = 0x080_u32;
    const GPIO16_CTRL_OFFSET: u32 = 0x084_u32;
    const GPIO16_STATUS: *mut u32 = (IO_BANK0_BASE + GPIO16_STATUS_OFFSET) as *mut u32;
    const GPIO16_CTRL: *mut u32 = (IO_BANK0_BASE + GPIO16_CTRL_OFFSET) as *mut u32;
    // now define electrical behavior with pad bank
    const GPIO16_PAD_OFFSET: u32 = 0x0000_0044_u32;
    const GPIO16_PAD: *mut u32 = (PAD_BANK0_BASE + GPIO16_PAD_OFFSET) as *mut u32;
    const PAD_ISO_BIT: u32 = 1 << 8; // ISO bit -> need clear
    const PAD_OD_BIT: u32 = 1 << 7; // this is output disable bit -> need clear
    const PAD_IE_BIT: u32 = 1 << 6; // input enable bit -> (optional)need set

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
        // 4.1 IO_BANK
        // To allocate a function to a GPIO, write to the FUNCSEL
        // field in the CTRL register corresponding to the pin.
        // In this case, we give SIO function -> F5
        core::ptr::write_volatile(GPIO16_CTRL, 0x05_u32);

        // 4.2 PAD_BANK
        // Datasheet said:
        // Bit 8: ISO Pad isolation control.
        // Remove this once the pad is configured by software.
        let pad_value = core::ptr::read_volatile(GPIO16_PAD);
        let new_pad_value = (pad_value & !(PAD_ISO_BIT | PAD_OD_BIT)) | PAD_IE_BIT; // remove 8th | 7th bits and set 6th bit
        core::ptr::write_volatile(GPIO16_PAD, new_pad_value);

        // 4.3 SIO - set the bit high low
        core::ptr::write_volatile(GPIO_OE_SET, LED_PIN_MASK); // make it and output
        core::ptr::write_volatile(GPIO_OUT_CLR, LED_PIN_MASK); // clear first
        loop {
            core::ptr::write_volatile(GPIO_OUT_SET, LED_PIN_MASK); // set high

            // delay a bit
            // each spin_loop() cost some cpu cycle
            // if the mcu speed is 15Mhz and a spin takes 4 cycle
            // 1000000 iteration will take about 4/15000000 = 0.27 sec
            for _ in 0..1000_000 {
                //  keep this loop and issue the
                // > architecture’s spin-wait instruction.
                core::hint::spin_loop();
            }

            core::ptr::write_volatile(GPIO_OUT_CLR, LED_PIN_MASK); // clear

            // delay a bit
            for _ in 0..1000_000 {
                //  keep this loop and issue the
                // > architecture’s spin-wait instruction.
                core::hint::spin_loop();
            }
        }
    };
}
