# UART interrupt → async operation lab

Implement **uart.cxx** and the **asyncWait template in uart.hpp**. The hardware
and application are provided. Your original code is in `original_example.cxx.txt`.

```sh
make                 # C++20 and Boost headers; defaults to /opt/homebrew
./uart_lab           # TX exercise; initially prints TODO and exits with code 1
./uart_lab --rx      # Optional RX exercise
make test            # Tests the supplied hardware, independently of your TODOs
```

Override the include prefix if needed: `make BOOST_PREFIX=/usr/local`.
The installed Boost must provide `boost/asio/any_completion_handler.hpp`.

`send()` accepts `std::span<const std::uint8_t>` for text or binary payloads:

```cpp
const std::uint8_t message[] = {0x01, 0x00, 0xff};
co_await uart.send(message);
co_await uart.send({}); // Empty span
```

Arrays and vectors of `std::uint8_t` convert to a span. The span does not own
the data: keep its buffer alive through the awaited call. Each call accepts up
to 256 bytes; split larger spans with `first(n)` / `subspan(n)` and await each
send before starting the next.

## What you write

1. Constructor: reset the UART and enable it. Main supplies interrupt wiring.
2. `asyncWait`: use `async_initiate` to obtain and save a completion handler.
   Record the event being awaited, unmask it, and check the live status in case
   the condition is already true. The template lives in the header so callers
   can instantiate it for their completion token.
3. `send`: set TX_OFFSET, set TX_BYTE_NUMBER, fill TX MEM, signal START,
   then await FIFO-empty **and** TX_BYTE_COUNT == TX_BYTE_NUMBER. Limit each message to 256 bytes; throw for longer messages.
   No concurrent operations in this lab. See the exact sequence below.
4. Interrupt callback: inspect pending/unmasked events and acknowledge them.
   For TX_EMPTY, compare TX_BYTE_COUNT with TX_BYTE_NUMBER and check live FIFO
   status. Complete only when the counts match and FIFO is empty. On intermediate
   empty events, retain the handler and leave TX_EMPTY enabled; hardware refills
   automatically. Completion masks the event, removes any stale latch,
   moves out the handler, and posts its invocation to the executor exactly once.
5. Optional `receive`: wait for RX-ready, then read one byte from RX_DATA.

The `notImplemented()` awaitable only keeps the starter compilable. Replace its
use with `async_initiate`; you can delete that helper afterward.

Use register access in your driver. Main connects the simulated interrupt vector:
`Hardware → UART_IRQHandler() → uart.handleInterrupt()`. The public
`handleInterrupt()` method is your driver's entry point from the platform.
Main sets the target object, installs the vector, and disconnects it on shutdown.
`registerInterruptHandler()` is the simulator's stand-in for platform vector
wiring; your Uart class does not use it. The `injectRx()` and `transmitted()`
methods also belong to the test bench in main.
Timers inside hardware.cxx simulate physical time; your driver needs no timer
or polling loop. The runner supplies a five-second watchdog and keeps Asio alive.

```text
main: co_await uart.send(hello)  // uint8_t array containing h e l l o
  → your send sets offset/length, fills TX MEM, and signals START
  → your asyncWait initiation lambda saves Asio's handler and unmasks TX_EMPTY
  → hardware feeds a 16-byte FIFO from TX MEM; UART shifts bytes onto the wire
  → hardware invokes UART_IRQHandler(), which calls uart.handleInterrupt()
  → acknowledge each empty event; post completion only when count == length
  → intermediate empty events keep the coroutine waiting
  → saved handler runs; main continues after co_await
```

The runner exercises short, empty, repeated, 100-byte, and binary sends. The 100-byte
send crosses multiple FIFO-empty IRQs and must not return at the first one.
After sends complete, the runner lets the final wire byte finish and checks the
entire transmitted stream. `send()` itself does not promise wire idle.
The RX runner injects `R` after 100 ms and checks that you consume it.

## Hardware register contract

This is a fictional UART, with explicit rules for this exercise. Access it with
`hw_.read(Reg::CONTROL_0)`; update CONTROL_1 fields with the helpers below.
Methods model register side effects; they aren't plain RAM fields.

| Offset | Register | Access / meaning |
|---|---|---|
| 0x00 | CONTROL_0 | Live status and commands; bits below |
| 0x04 | CONTROL_1 | Packed baud divisor, 7-bit IRQ mask, and framing; layout below |
| 0x0c | RX_DATA | Read pops one byte; empty reads throw |
| 0x14 | RX_TIMEOUT_MS | Default 100 ms; 0 disables RX timeout |
| 0x18 | TX_OFFSET | Read/write start offset in **bytes**, default 0 |
| 0x1c | TX_BYTE_NUMBER | Read/write transfer length in bytes, default 0 |
| 0x20 | TX_BYTE_COUNT | Read-only bytes consumed from TX MEM into FIFO since last START |
| 0x24 | TX_COMMAND | Write `txcommand::START` (bit 0) to start; reads zero |
| 0x28 | RX_BYTE_NUMBER | Write 1–16 to arm a requested-length receive; default 0 = streaming |
| 0x2c | RX_BYTE_COUNT | Read-only cumulative bytes accepted since arming/reset |
| 0x100–0x1fc | TX MEM | 64 read/write 32-bit words; 256 payload bytes |

| CONTROL_0 bit | Name | Meaning |
|---|---|---|
| 0 | RX_EMPTY | Read-only: no received bytes |
| 1 | RX_FULL | Read-only: all 16 RX slots occupied |
| 2 | TX_EMPTY | Read-only: TX FIFO is empty, even if more memory remains or UART is shifting |
| 3 | TX_FULL | Read-only: all 16 TX FIFO slots occupied |
| 4 | CLEAR | Write 1 clears all interrupt latches, preserves FIFO data |
| 5 | ENABLE | Read/write peripheral enable |
| 6 | UART_RESET | Write 1 resets RX FIFO, TX memory/config/count, mask, events, timeout and enable |
| 7 | RX_TIMEOUT | Read-only: latched RX timeout |
| 8 | TX_BUSY | Read-only: memory transfer or shift register still active |

CLEAR and UART_RESET read as zero. Other read-only bits ignore writes. A
CONTROL_0 write also sets ENABLE to the value of bit 5, so clear while enabled
with `control0::ENABLE | control0::CLEAR`. RESET takes precedence over all other
bits and leaves the UART disabled; enable it in a separate write.

| CONTROL_1 IRQ-mask bit | Event source indicated by CONTROL_0 |
|---|---|
| 0 | RX_READY: RX changes empty → nonempty, or requested RX length is reached |
| 1 | RX_FULL: RX becomes full |
| 2 | TX_EMPTY: TX FIFO drains (also latched by zero-length START) |
| 3 | TX_FULL: TX FIFO becomes full |
| 6 | RX_TIMEOUT: unread RX data remains without a new byte for the configured interval |

CONTROL_0[0:8] are the live status/control bits listed above. They are not a
pending-event bitmap. CONTROL_1 contains the interrupt mask; its seven mask
bits correspond to the event sources in the table. The simulator internally
latches notifications until CONTROL_0.CLEAR, then queues an enabled, unmasked
notification on Asio. CLEAR preserves FIFO contents and live status. This
internal latch models edge notifications; software reads CONTROL_0 status to
decide whether the condition it needs is currently true. There is no separate
IRQ status register; the former 0x08 address is reserved and accesses throw.

Each RX arrival restarts its timeout, unless the requested length has been reached.
Emptying RX cancels the timer; acknowledging
timeout does not restart it. A new byte or timeout-setting write restarts timing.
Disabling the UART pauses TX and cancels RX timing, preserving data, transfer progress, and latches;
re-enabling resumes TX and starts RX timing again. Reset cancels in-flight timing
and clears RX FIFO and TX memory, but preserves the test bench's transmission history.

## CONTROL_1 packed fields

This is the simulator's chosen layout; your board's manual may use other offsets.
The interrupt mask is **7 bits inside a 32-bit register**:

| Bits | Field | Reset value |
|---|---|---|
| 15:0 | Baud divisor | 48 |
| 22:16 | Interrupt-enable mask, 1 = enabled | 0 |
| 23 | Parity enable | 0 |
| 24 | Odd parity (when parity enabled) | 0 |
| 25 | Two stop bits (0 = one) | 0 |
| 27:26 | Data bits: 0=5, 1=6, 2=7, 3=8 | 3 |
| 31:28 | Reserved, reads zero, writes ignored | 0 |

Baud/framing fields support configuration and readback; they do not change the
simulated 10 ms byte timing or emulate parity/framing errors. Payloads still use
all eight bits. No actual baud frequency is implied without a peripheral clock.
IRQ-mask bits 4 and 5 are stored but have no event source. RX_TIMEOUT now uses
unshifted event bit **6**; CONTROL_0's timeout status remains bit **7**.

```cpp
auto config = hw_.read(Reg::CONTROL_1);
auto mask = control1::irqMask(config); // Extract bits 22:16 into bits 6:0
mask |= irq::TX_EMPTY;
hw_.write(Reg::CONTROL_1,
          (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

// Disable only TX-empty; preserve RX enables, baud, and framing.
config = hw_.read(Reg::CONTROL_1);
mask = control1::irqMask(config) & ~irq::TX_EMPTY;
hw_.write(Reg::CONTROL_1,
          (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

// Change only the baud divisor.
config = hw_.read(Reg::CONTROL_1);
hw_.write(Reg::CONTROL_1, (config & ~control1::BAUD_DIV_MASK) | 96u);

// Derive source conditions from CONTROL_0 status and check the CONTROL_1 mask.
const auto status = hw_.read(Reg::CONTROL_0);
const auto mask = control1::irqMask(hw_.read(Reg::CONTROL_1));
```

A raw `write(CONTROL_1, irq::TX_EMPTY)` would now change the baud divisor and
clear the interrupt mask. Place event bits in the field with a read/modify/write, preserving the other settings. This is read/modify/write on the
lab's single thread; independently executing code would need synchronization.

## Prepare TX memory, then signal transmission

This simulator uses your requested sequence. These are lab-defined registers,
not a verified map of your board. Both RX and TX FIFOs have 16 slots
(`FIFO_CAPACITY`). TX MEM is separate and holds 256 bytes (`TX_MEM_BYTES`).

Example register sequence for sending `"hello"` (your async `send()` remains TODO):

```cpp
hw_.write(Reg::TX_OFFSET, 0);
hw_.write(Reg::TX_BYTE_NUMBER, 5);

// Write to TX MEM as 32-bit words, four bytes per word.
const std::uint8_t payload[] = {'h', 'e', 'l', 'l', 'o'};
const std::span<const std::uint8_t> message = payload;
const std::size_t wordCount = (message.size() + 3) / 4;
for (std::size_t wordIndex = 0; wordIndex < wordCount; ++wordIndex) {
    std::uint32_t word = 0;
    for (std::size_t lane = 0; lane < 4; ++lane) {
        const std::size_t byteIndex = wordIndex * 4 + lane;
        if (byteIndex < message.size())
            word |= static_cast<std::uint32_t>(message[byteIndex]) << (lane * 8);
    }
    hw_.getTxMemory()[wordIndex].setTxMem(word);
}

// Signal TX transfer. Nothing above starts transmission.
hw_.write(Reg::TX_COMMAND, txcommand::START);

// Your bridge completes only when FIFO is empty AND count == length.
co_await asyncWait(irq::TX_EMPTY, asio::use_awaitable);
```

Use `hw_.getTxMemory()[index].setTxMem(word)` for one simulated 32-bit MMIO
write. `getTxMem()` reads that word back. You can also keep the view:

```cpp
auto txMem = hw_.getTxMemory(); // View, not a copy of the 64 words
txMem[0].setTxMem(0x6c6c6568u);
auto word = txMem[0].getTxMem();
```

Indices are word indices 0–63; invalid indices throw. The setter retains the
active-transfer write check, even when you acquired the view before START.
Keep Hardware alive while using the view. `send()` has an unfinished packing
loop for you to implement; the complete example above shows the pattern.
Packing is explicit:
the first byte is bits 7:0, then 15:8, 23:16, 31:24. Thus word `0x6c6c6568`
contains `h e l l` in transmission order. The next word is `0x0000006f` for `o`.
Unused lanes in the final word are zero; TX_BYTE_NUMBER remains 5 so padding
is never sent. This does not depend on host endianness.

TX_OFFSET selects the first byte to send; it does not auto-increment or change
which word `getTxMemory()[index]` accesses. START validates `offset + length <= 256`, resets
TX_BYTE_COUNT, clears old TX_EMPTY/TX_FULL events, and loads up to 16 bytes into
its FIFO. **Count increments when bytes leave TX MEM and enter the FIFO.**
Memory contents persist for reuse.

The simulator deliberately feeds in bursts to demonstrate FIFO starvation:

```text
TX MEM → 16-byte FIFO → one-byte shift register → completed wire bytes

100-byte transfer: FIFO-empty IRQ counts = 16, 32, 48, 64, 80, 96, 100
```

Every 10 ms, any byte in the shift register finishes, then the next FIFO byte
enters that register. When the FIFO drains it raises TX_EMPTY. If more TX MEM
bytes remain, the next tick automatically refills the FIFO. This deliberate gap
makes intermediate empty interrupts reproducible. Acknowledging them does not
stop the feeder. A stale latched IRQ can outlive the empty condition, so check
live FIFO status as well as count before completing the wait.

At the last empty IRQ, count equals the programmed length, but the final byte
is still shifting. In this exercise, `send()` may complete at that point and
TX MEM may be reused. TX_BUSY remains set until the wire is idle. A following
START may queue another transfer while the previous final byte shifts; ordering
is preserved. Waiting for wire idle would be a separate operation, not part of
this exercise. No separate wire-idle interrupt is implemented.

Loading memory or changing offset/length does not send bytes. ENABLE only enables
the peripheral; START starts a new transfer. A zero-length START completes
immediately, with count zero and TX_EMPTY latched; it does not cancel a previous
byte still shifting. Starting while disabled or before the previous transfer's
FIFO has fully drained throws. Writing TX memory/offset/length before that point
also throws, including while paused. Reset aborts all stages and clears the FIFO
and shift register. TX_BYTE_COUNT writes throw.

In this lab all activity runs on one event-loop thread, and interrupt delivery
is queued. Starting before `co_await asyncWait(...)` is supported. In both the
initiation lambda's already-ready check and the ISR, require live TX_EMPTY AND
TX_BYTE_COUNT == TX_BYTE_NUMBER. Checking just FIFO-empty or just count is wrong:
FIFO can empty mid-transfer, and count may reach length while FIFO still has data.
On real hardware with independently executing ISRs, save the handler and arm
completion before START; that ordering belongs in the initiation lambda.

## Requested-length RX

RX currently has a 16-byte FIFO, not a separate memory window. To start a receive,
write its expected length before waiting for interrupts:

```cpp
hw_.write(Reg::RX_BYTE_NUMBER, 5); // Arm: expect 5 bytes, reset running count
// asyncWait's RX branch enables RX_READY | RX_TIMEOUT, then checks:
const bool ready =
    !(hw_.read(Reg::CONTROL_0) & control0::RX_EMPTY) &&
    hw_.read(Reg::RX_BYTE_COUNT) == hw_.read(Reg::RX_BYTE_NUMBER);
```

Use that same condition in the RX_READY interrupt branch before completing.
RX_FULL means all 16 slots are occupied, so it cannot indicate completion of a
5-byte receive. The first byte raises RX_READY; acknowledge it and keep waiting
if count is below the target. Reaching the target raises RX_READY again, even
if the FIFO never became empty between arrivals. Completion doesn't consume
the data: read RX_DATA once for each byte afterward. FIFO reads do not decrease
RX_BYTE_COUNT. The existing `receive()` exercise returns one byte, so program
RX_BYTE_NUMBER = 1 there; receiving a block needs an appropriate buffer API.

Programming RX_BYTE_NUMBER requires an empty FIFO and clears old RX interrupt
latches and timeout timing. Valid lengths are 1–16; larger values throw. Writing
0 restores the original streaming mode, with no target, and resets the count.
In requested-length mode, extra incoming bytes are rejected once the target is
reached until software drains the FIFO and rearms reception. This is a simulator
rule, not a claim that a real UART will stop a peer from transmitting.

Partial buffered data can trigger RX_TIMEOUT; inspect RX_BYTE_COUNT to see how
much arrived. Reaching the target cancels the timeout. No incoming bytes means
no RX inactivity timer in this model; a first-byte deadline is separate. Reset
clears both RX registers, restores streaming mode, and empties the FIFO.
Your async RX implementation remains an exercise: ensure completion disables
both RX masks, acknowledges pending RX events, and resets the saved event.

## Scope and next experiments

All calls and callbacks use one io_context thread. Keep Uart and Hardware alive
while io.run() runs; do not restart the context after destroying them. This lab
does not implement cancellation or concurrent sends. Real hardware ISR/thread
synchronization and platform-specific interrupt handoff are separate work.

After TX and RX pass, try a receive-with-timeout operation, test RX already
containing data before a wait, or support sends larger than TX MEM. RX timeout
here measures inactivity with buffered data, not a timeout waiting for a first byte.
