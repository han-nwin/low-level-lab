#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

// Keep a predictable field order and C-compatible memory layout.
#[repr(C)]
struct MyMetadata {
    magic: [u8; 12], // 12 bytes
    name: u32,
    version: u32,
    led_pin: u32,
    button_pin: u32,
}

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

// Custom meta data
#[used]
#[unsafe(link_section = ".my_metadata")]
static MY_METADATA: MyMetadata = MyMetadata {
    magic: *b"Han Dep Trai",
    name: u32::from_le_bytes(*b" HEY"), // pico is little-endian
    version: 1,
    led_pin: 16,
    button_pin: 15,
};

#[entry]
fn main() -> ! {
    const LED_PIN_MASK: u32 = 1 << 16;
    const BUTTON_PIN_MASK: u32 = 1 << 15;

    // RP2350 reset controller
    // This is the reset engine, we mask it with what we wanna reset
    const RESETS_RESET: *mut u32 = 0x4002_0000 as *mut u32;
    const RESETS_WDSEL: *mut u32 = 0x4002_0004 as *mut u32;
    const RESETS_RESET_DONE: *const u32 = 0x4002_0008 as *const u32; //read only

    // GPIO passes through two major pieces of hardware:
    // SIO signal
    //    ↓
    // IO_BANK0        selects what controls GP16
    //    ↓
    // PADS_BANK0      controls the physical/electrical pin
    //    ↓
    // GP PINS

    // NOTE: SHARED Config
    // We need to reset IO_BANK0 and PADS_BANK0
    const IO_BANK0_BIT: u32 = 0x0000_0040_u32;
    // const IO_BANK0_MASK: u32 = (1 << 6) as u32; // bit 6
    const PADS_BANK0_BIT: u32 = 0x0000_0200_u32;
    // const PADS_BANK0_BIT: u32 = (1 << 9) as u32; // bit 9

    // Route to GPIO 16
    // SIO -> IO_BANK -> PADS_BANK -> GPIO16
    const SIO_BASE: u32 = 0xD000_0000_u32;

    // now define SIO
    // output Pin
    const GPIO_OE_SET_OFFSET: u32 = 0x038_u32; // enable the pin as output, can use GPIO_OE but it will reset other pins
    const GPIO_OUT_SET_OFFSET: u32 = 0x018_u32; // use this bit to set it high
    const GPIO_OUT_CLR_OFFSET: u32 = 0x020_u32; // this use to clear the bit -> set low
    const GPIO_OE_SET: *mut u32 = (SIO_BASE + GPIO_OE_SET_OFFSET) as *mut u32;
    const GPIO_OUT_SET: *mut u32 = (SIO_BASE + GPIO_OUT_SET_OFFSET) as *mut u32;
    const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + GPIO_OUT_CLR_OFFSET) as *mut u32;
    // input Pin
    const GPIO_IN_OFFSET: u32 = 0x004_u32; // enable the pin as input
    const GPIO_IN: *const u32 = (SIO_BASE + GPIO_IN_OFFSET) as *const u32; // input is readonly

    // Share Pad config
    const PAD_ISO_BIT: u32 = 1 << 8; // ISO bit -> need clear
    const PAD_OD_BIT: u32 = 1 << 7; // this is output disable bit
    const PAD_IE_BIT: u32 = 1 << 6; // input enable bit
    const PAD_PUE_BIT: u32 = 1 << 3; // pull up enable bi
    const PAD_PDE_BIT: u32 = 1 << 2; // pull down enable bit
    // END Share config

    const IO_BANK0_BASE: u32 = 0x4002_8000_u32;
    const PAD_BANK0_BASE: u32 = 0x4003_8000_u32;

    // Datasheet: 9.11 Registers table
    // NOTE: GP16 Config Constants
    // now tell IO_BANK what peripheral connected to the pin (it's the GPIO16)
    const GPIO16_STATUS_OFFSET: u32 = 0x080_u32;
    const GPIO16_CTRL_OFFSET: u32 = 0x084_u32;
    const GPIO16_STATUS: *const u32 = (IO_BANK0_BASE + GPIO16_STATUS_OFFSET) as *const u32;
    const GPIO16_CTRL: *mut u32 = (IO_BANK0_BASE + GPIO16_CTRL_OFFSET) as *mut u32;
    // now define electrical behavior with pad bank
    const GPIO16_PAD_OFFSET: u32 = 0x0000_0044_u32;
    const GPIO16_PAD: *mut u32 = (PAD_BANK0_BASE + GPIO16_PAD_OFFSET) as *mut u32;
    // End GP16 Config Constans

    // NOTE: GP15 Config Constant
    const GPIO15_STATUS_OFFSET: u32 = 0x078_u32;
    const GPIO15_CTRL_OFFSET: u32 = 0x07c_u32;
    const GPIO15_STATUS: *const u32 = (IO_BANK0_BASE + GPIO15_STATUS_OFFSET) as *const u32; //RO
    const GPIO15_CTRL: *mut u32 = (IO_BANK0_BASE + GPIO15_CTRL_OFFSET) as *mut u32;
    // now define electrical behavior with pad bank
    const GPIO15_PAD_OFFSET: u32 = 0x0000_0040_u32;
    const GPIO15_PAD: *mut u32 = (PAD_BANK0_BASE + GPIO15_PAD_OFFSET) as *mut u32;
    // End GP15 Config Constans

    unsafe {
        // 1. Reset IO_BANK0 and PADS_BANK0
        // optional
        let reset = core::ptr::read_volatile(RESETS_RESET);
        core::ptr::write_volatile(RESETS_RESET, reset | IO_BANK0_BIT | PADS_BANK0_BIT);

        // 2. Release the reset: clear IO_BANK0 and PADS_BANK0
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
        // SIO = FUNCSEL 5
        core::ptr::write_volatile(GPIO16_CTRL, 0x05_u32);
        core::ptr::write_volatile(GPIO15_CTRL, 0x05_u32);

        // 4.2 PAD_BANK
        let output_pad_value = core::ptr::read_volatile(GPIO16_PAD);
        let new_output_pad_value = (output_pad_value & !(PAD_ISO_BIT | PAD_OD_BIT)) | PAD_IE_BIT; // remove 8th | 7th bits and set 6th bit
        core::ptr::write_volatile(GPIO16_PAD, new_output_pad_value);

        let input_pad_value = core::ptr::read_volatile(GPIO15_PAD);
        // Configure GP15 as an input with an internal pull-up:
        // - When the button is released, the pull-up holds GP15 at 3.3 V (reads 1).
        // - When the button connects GP15 to GND, GP15 reads 0.
        // Clear ISO and pull-down; enable input and pull-up.
        let new_input_pad_value =
            (input_pad_value & !(PAD_ISO_BIT | PAD_PDE_BIT)) | PAD_IE_BIT | PAD_PUE_BIT; // remove 8th | 2th bits and set 6th | 3th bits
        core::ptr::write_volatile(GPIO15_PAD, new_input_pad_value);

        // 4.3 SIO - set the bit high low
        // led
        core::ptr::write_volatile(GPIO_OE_SET, LED_PIN_MASK); // make it and output
        core::ptr::write_volatile(GPIO_OUT_CLR, LED_PIN_MASK); // clear first

        let mut blink_enabled = false;
        let mut led_on = false;
        let mut led_state_counter = 0_u32;

        // NOTE: Each loop here handle both led on/off and button pressed
        loop {
            // Pull-up configuration: pressed means GP15 reads 0.
            let button_pressed = (core::ptr::read_volatile(GPIO_IN) & BUTTON_PIN_MASK) == 0;

            if button_pressed {
                blink_enabled = !blink_enabled;
                led_state_counter = 0;

                // Wait until the button is released.
                while (core::ptr::read_volatile(GPIO_IN) & BUTTON_PIN_MASK) == 0 {
                    core::hint::spin_loop();
                }

                // Ideal release:
                // 0000000011111111
                //
                // Real release:
                // 0000000101011111
                //        |- bouncing -|
                // Simple release debounce.
                for _ in 0..100_000 {
                    core::hint::spin_loop();
                }

                led_on = blink_enabled;

                if led_on {
                    core::ptr::write_volatile(GPIO_OUT_SET, LED_PIN_MASK);
                } else {
                    core::ptr::write_volatile(GPIO_OUT_CLR, LED_PIN_MASK);
                }
            }

            // we iterate through the counter to turn the led on/off -> blinky
            if blink_enabled {
                led_state_counter += 1;
                // the speed between each on/off
                if led_state_counter > 50_000 {
                    led_on = !led_on;
                    led_state_counter = 0;
                }

                if led_on {
                    core::ptr::write_volatile(GPIO_OUT_SET, LED_PIN_MASK);
                } else {
                    core::ptr::write_volatile(GPIO_OUT_CLR, LED_PIN_MASK);
                }
            }

            core::hint::spin_loop();
        }
    };
}
