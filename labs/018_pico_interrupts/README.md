# Lab 018: Busy-Wait Delay vs Timer Interrupt

## Goal

Start with the blocking timer delay from lab 017:

```rust
fn delay_microsec(micro_sec: u64) {
    let start = timer_ticks();
    let delay_tick = 2 * micro_sec; // 2 ticks per microsecond
    while timer_ticks().wrapping_sub(delay_tick) < start {
        logging::poll(); // keep USB alive
        core::hint::spin_loop();
    }
}
```

Then create a second delay mechanism using a TIMER0 alarm and an interrupt.
Compare the two designs and prove whether foreground code can do useful work
while each delay is active.

```text
busy-wait delay                     interrupt delay

call delay_microsec()               arm timer alarm
        |                                  |
        v                                  v
main stays inside loop              function returns to main
        |                                  |
        v                                  v
foreground work stops               foreground work continues
        |                                  |
        v                                  v
deadline reached                    alarm interrupt fires
        |                                  |
        v                                  v
function returns                    handler marks delay complete
```

The goal is not merely to make an interrupt fire. The goal is to understand
the difference between a blocking delay and a non-blocking, interrupt-driven
deadline.

## Starting point

This directory carries the working setup from `../017_pico_timers`:

- `memory.x`
- `.cargo/config.toml`
- `.vscode/settings.json`
- `Cargo.toml`
- `src/logging.rs`
- `src/main.rs`

Keep the working timer setup and `timer_ticks()` implementation. The copied
program deliberately still contains the old busy-wait delay so it can serve as
the baseline measurement.

For a fair first comparison, keep the current configuration of two timer ticks
per microsecond in both tests. If you later restore the normal one-tick-per-
microsecond configuration, update both delay calculations together.

## Important API difference

A function that arms an interrupt and then loops until the handler sets a flag
is still a blocking function. It changes how completion is detected, but it
does not free `main` to do other work.

Design the interrupt version as a non-blocking operation with two concepts:

```text
start an interrupt delay -> returns immediately
check/receive completion -> tells main when the handler has fired
```

Choose your own function names and shared-state design. Do not force the new
mechanism into the same blocking shape as `delay_microsec()`.

## The comparison experiment

Run two tests with the same delay, for example three seconds.

### Test A: busy-wait delay

1. Show `BUSY WAIT` on the LCD.
2. Reset a foreground work counter to zero.
3. Call `delay_microsec(3_000_000)`.
4. Try to update the foreground counter and LCD from `main`.
5. When the delay returns, log the elapsed time and final work count.

Expected observation: the LCD display freezes during the delay because `main`
cannot reach the foreground work. USB polling still occurs only because it was
manually placed inside `delay_microsec()`.

### Test B: interrupt delay

1. Show `IRQ WAIT` on the LCD.
2. Reset the same foreground work counter to zero.
3. Arm a TIMER0 alarm for the same three-second duration.
4. Return immediately to the main loop.
5. Increment the work counter and periodically update it on the LCD.
6. Stop the test when the interrupt handler reports completion.
7. Log the elapsed time and final work count.

Expected observation: the LCD counter continues changing during the delay.
The final count should be much larger than the busy-wait result because the
foreground remained available.

The work counter is the measurement; the LCD makes the result visible. Do not
compare raw loop counts as a CPU benchmark—the I²C writes and USB polling also
consume time. The important result is stopped versus continuing foreground
progress.

## LCD setup from lab 015

Reuse the raw I²C1 and LCD code from `../015_pico_i2c` after you have the timer
interrupt working by itself.

```text
Pico 2 W                 LCD module

5V       ------------->  VDD
GND      ------------->  GND
GP10     <--- level shifter ---> SDA
GP11     <--- level shifter ---> SCL
```

Bring over only the pieces needed to initialize I²C1/LCD, position the cursor,
and write a short counter. Do not access I²C or the LCD from the timer interrupt
handler. The handler should only acknowledge the timer and publish minimal
completion state; `main` owns the display.

If the LCD is not connected, use the GP16 external LED from lab 012 as a
fallback heartbeat. It should freeze during the busy wait and keep blinking
during the interrupt-driven wait. Keep the USB work-count logs in either case.

## Timer interrupt requirements

- Use TIMER0 Alarm 0 through raw register access.
- Enable Alarm 0 in the TIMER0 peripheral.
- Enable the corresponding interrupt in the Cortex-M33 NVIC.
- Write the interrupt handler yourself.
- Clear/acknowledge the timer interrupt in the handler.
- Keep the handler short and bounded.
- Do not log, update the LCD, or perform I²C transfers inside the handler.
- Share completion state safely; do not use an unsynchronized `static mut`.
- Keep calling `logging::poll()` from foreground code.
- Do not poll `timer_ticks()` in the interrupt-driven test to decide whether
  the three-second deadline has arrived.

## Read first

In section 12.8 of the RP2350 datasheet, continue where lab 017 stopped:

1. **12.8.3 — Alarms**
2. **12.8.4 — Interrupts**
3. The TIMER register list for `ALARM0`, `ARMED`, `INTR`, `INTE`, and `INTS`

Also find the RP2350 interrupt table and confirm which NVIC interrupt belongs
to TIMER0 Alarm 0. Review how `cortex-m-rt` declares device interrupt handlers
and how the `cortex-m` crate controls NVIC interrupts.

Get register offsets and bit meanings from the datasheet rather than from a
finished implementation.

## Suggested checkpoints

1. Build and run the copied lab 017 program unchanged.
2. Add the TIMER0 alarm and make one interrupt fire.
3. Have the handler acknowledge the interrupt and publish completion state.
4. Build the non-blocking start/completion interface.
5. Compare busy-wait and interrupt delays using only a foreground work counter
   and USB logs.
6. Add the lab 015 LCD as the visual foreground workload.
7. Confirm the LCD freezes in the busy test and continues in the IRQ test.

## Questions to answer

1. Why does the original `delay_microsec()` block application work even though
   it calls `logging::poll()`?
2. Why would waiting on an interrupt-completion flag inside the new function
   still block the caller?
3. Why are both the TIMER0 interrupt enable and the NVIC enable required?
4. Why must the handler acknowledge the peripheral before returning?
5. Why must I²C, LCD, and USB operations stay outside the handler?
6. How is completion state safely shared between the handler and `main`?
7. What happens to the alarm comparison when the low timer word wraps?
8. If the core used `wfi` while waiting, would the application be non-blocking,
   CPU-efficient, both, or neither?

## Done when

- Both tests use the same requested delay and timer tick rate.
- The busy-wait test visibly stops foreground progress.
- The interrupt-driven test returns control to `main` while the deadline is
  pending.
- TIMER0 Alarm 0 invokes the handler without deadline polling in `main`.
- The handler acknowledges the correct source and safely reports completion.
- The LCD counter or LED heartbeat continues during the interrupt wait.
- USB logs show elapsed time and foreground work counts for both tests.
- You can explain why the interrupt version frees the foreground rather than
  merely replacing one polling condition with another.
