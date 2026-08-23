# Lab 018: A Reminder Alarm

## Goal

Build two hardcoded five-second reminder alarms:

- **Polling alarm:** `main` waits inside the delay, so its LCD animation freezes.
- **Interrupt alarm:** `main` starts the alarm and continues updating the LCD.

When either alarm expires, the LCD displays `TIME UP`.

The LCD animation represents normal application work. It makes the difference
between blocking and non-blocking code easy to see.

## Starting point

The starter code already has:

- `delay_microsec_polling()` from lab 017;
- USB logging;
- the LCD driver from lab 015;
- TIMER0 configured for **two ticks per microsecond**.

Use a fixed duration in both examples:

```rust
const ALARM_US: u64 = 5_000_000;
```

## LCD activity

Have `main` animate one LCD character using these frames:

```rust
let frames = [b'|', b'/', b'-', b'\\'];
```

For example:

```text
POLL ALARM 5s
Main active  |
```

The last character changes while `main` is available to update it.

## Example 1: Polling alarm

1. Display `POLL ALARM 5s` and the first animation frame.
2. Call `delay_microsec_polling(ALARM_US)`.
3. Display `TIME UP` after the function returns.

During the five-second delay, the animation is frozen because `main` is stuck
inside `delay_microsec_polling()`.

```text
show POLL ALARM 5s
         |
         v
main enters polling delay ----> LCD freezes
         |
         v
five seconds pass ----> show TIME UP
```

Calling `logging::poll()` inside the delay keeps USB alive, but it does not let
`main` run the LCD animation.

## Example 2: Interrupt alarm

Implement a non-blocking alarm using TIMER0 Alarm 0.

1. Display `IRQ ALARM 5s`.
2. Arm Alarm 0 for five seconds in the future.
3. Return immediately to `main`.
4. Keep cycling through the LCD animation frames.
5. When the interrupt reports completion, display `TIME UP`.

```text
show IRQ ALARM 5s
         |
         v
arm Alarm 0 and return ----> main keeps animating the LCD
         |
         v
interrupt fires ----> handler sets flag ----> main shows TIME UP
```

The alarm-start function must only arm the alarm and return. If it loops until
the interrupt sets a flag, it is still blocking.

Only `main` updates the LCD. The interrupt handler should acknowledge the alarm
and set a completion flag—nothing more.

## Implementation checkpoints

1. Make the hardcoded LCD animation run.
2. Run the polling example and observe it freeze for five seconds.
3. Configure TIMER0 Alarm 0 and make one interrupt fire.
4. In the handler, acknowledge the alarm and set an `AtomicBool`.
5. Make the interrupt alarm function return immediately after arming it.
6. Animate the LCD in `main` until the completion flag is set.
7. Display `TIME UP` from `main`.

## Interrupt requirements

- Use TIMER0 Alarm 0 through raw register access.
- Enable Alarm 0 in both TIMER0 and the Cortex-M33 NVIC.
- Acknowledge the timer interrupt in the handler.
- Keep the handler short and bounded.
- Share completion safely, for example with `AtomicBool`; do not use an
  unsynchronized `static mut`.
- Do not perform LCD, I²C, USB, or logging work in the handler.
- Do not poll `timer_ticks()` in `main` to decide when the interrupt alarm has
  expired.
- Use the same five-second duration and tick rate in both examples.

Read RP2350 datasheet sections 12.8.3 (Alarms), 12.8.4 (Interrupts), the timer
register descriptions, and the interrupt table.

## LCD wiring

```text
Pico 2 W                 LCD module

5V       ------------->  VDD
GND      ------------->  GND
GP10     <--- level shifter ---> SDA
GP11     <--- level shifter ---> SCL
```

## Done when

- The polling alarm freezes the LCD animation until `TIME UP` appears.
- The interrupt alarm lets the LCD animation continue until `TIME UP` appears.
- Alarm 0 invokes the handler without deadline polling in `main`.
- The handler acknowledges the alarm and safely reports completion.
- You can explain why the interrupt version leaves `main` free to do other
  work.
