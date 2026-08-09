#![no_std]
#![no_main]

mod logging;

use cortex_m as _;
use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_halt as _;

#[entry]
fn main() -> ! {
    loop {}
}
