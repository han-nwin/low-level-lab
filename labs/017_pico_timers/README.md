
# Lab: RP2350 Hardware Timer

## Goal

Use the RP2350 **hardware timer** instead of a software delay loop.

## Requirements

* Use Raspberry Pi Pico 2 / RP2350.
* Do **not** use `cortex_m::asm::delay()`.
* Do **not** use a busy loop like `for _ in 0..N`.
* Read the timer counter from its hardware registers.
* Create a function such as:

```rust
fn timer_us() -> u64
```

that returns elapsed time in microseconds.

* Create:

```rust
fn delay_us(us: u64)
```

using the hardware timer.

* Blink an LED:

```text
ON
wait 500 ms
OFF
wait 500 ms
```

using your timer implementation.

## Stretch Goal

Measure how long a piece of code takes:

```rust
let start = timer_us();

do_some_work();

let elapsed = timer_us() - start;
```

Print `elapsed` over your existing logging/USB setup.

## What you should understand after

```text
clock
  ↓
hardware timer counter
  ↓
counter continuously increments
  ↓
your code reads counter
  ↓
compare current time against start time
```

The important idea is:

```rust
while timer_us() - start < delay {
    // wait
}
```

You're still polling/busy-waiting, but now **time comes from an actual hardware timer**, not from guessing how many CPU instructions have executed.


For **this timer lab**, read these exact places in the **RP2350 Datasheet**:

1. **Section 12.8 — System timers**, starting on **datasheet page 1182**. Read **12.8.1 Overview** first. It tells you the key fact: `TIMER0`/`TIMER1` contain a **64-bit counter that increments once per microsecond**. 

2. **Section 12.8.2 — Counter**, page **1183**. This is the most important part for your first implementation. It explains that because the bus is 32-bit, the 64-bit time is read through:

   ```text
   TIMELR  -> low 32 bits
   TIMEHR  -> high 32 bits
   ```

   and specifically says to read **TIMELR first, then TIMEHR** because reading the low register latches the high value for a consistent 64-bit read. 

3. Then jump to the **TIMER register list around page 1188–1189**. This gives you the actual addresses:

   ```text
   TIMER0_BASE = 0x400B0000
   TIMER1_BASE = 0x400B8000

   offset 0x08 = TIMEHR
   offset 0x0C = TIMELR
   ```

   ([Raspberry Pi][1])

So for the first part of the lab, your raw-register target is basically:

```rust
const TIMER0_BASE: u32 = 0x400B_0000;

const TIMEHR: *const u32 =
    (TIMER0_BASE + 0x08) as *const u32;

const TIMELR: *const u32 =
    (TIMER0_BASE + 0x0C) as *const u32;
```

Don't read the alarm/interrupt stuff yet. First goal is simply:

```text
12.8.1 Overview
      ↓
12.8.2 Counter
      ↓
register table: TIMEHR / TIMELR
      ↓
implement timer_us()
```

After you get `timer_us()` working, then we can move to **12.8.3 Alarms** and make it interrupt-driven instead of busy-waiting. 
