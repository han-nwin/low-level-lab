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
`hw_.read(Reg::CONTROL_0)` or `hw_.write(Reg::CONTROL_1, irq::TX_EMPTY)`.
Methods model register side effects; they aren't plain RAM fields.

| Offset | Register | Access / meaning |
|---|---|---|
| 0x00 | CONTROL_0 | Status, enable, clear, reset; bits below |
| 0x04 | CONTROL_1 | Read/write interrupt mask; **1 enables delivery** |
| 0x08 | IRQ_STATUS | Read latched events; write one to clear selected events |
| 0x0c | RX_DATA | Read pops one byte; empty reads throw |
| 0x14 | RX_TIMEOUT_MS | Default 100 ms; 0 disables RX timeout |
| 0x18 | TX_OFFSET | Read/write start offset in **bytes**, default 0 |
| 0x1c | TX_BYTE_NUMBER | Read/write transfer length in bytes, default 0 |
| 0x20 | TX_BYTE_COUNT | Read-only bytes consumed from TX MEM into FIFO since last START |
| 0x24 | TX_COMMAND | Write `txcommand::START` (bit 0) to start; reads zero |
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

| CONTROL_1 / IRQ_STATUS bit | Event |
|---|---|
| 0 | RX_READY: RX changes from empty to nonempty |
| 1 | RX_FULL: RX becomes full |
| 2 | TX_EMPTY: TX FIFO drains (also latched by zero-length START) |
| 3 | TX_FULL: TX FIFO becomes full |
| 7 | RX_TIMEOUT: unread RX data remains without a new byte for the configured interval |

Events latch even while masked. IRQ delivery requires ENABLE and at least one
bit in `IRQ_STATUS & CONTROL_1`. Delivery is queued on Asio, never inline with a
register write. An unacknowledged, unmasked event keeps delivering interrupts.
Writing `irq::TX_EMPTY` to IRQ_STATUS clears just that latch; writing zero clears
nothing. Acknowledging an event does not change the FIFO.

Live status and latched events differ: reset sets TX_EMPTY status, but does not
latch a TX_EMPTY event. Likewise, acknowledging RX_READY with bytes still in RX
does not generate another RX_READY event. Check live status when starting a wait!

Each RX arrival restarts its timeout. Emptying RX cancels the timer; acknowledging
timeout does not restart it. A new byte or timeout-setting write restarts timing.
Disabling the UART pauses TX and cancels RX timing, preserving data, transfer progress, and latches;
re-enabling resumes TX and starts RX timing again. Reset cancels in-flight timing
and clears RX FIFO and TX memory, but preserves the test bench's transmission history.

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

## Scope and next experiments

All calls and callbacks use one io_context thread. Keep Uart and Hardware alive
while io.run() runs; do not restart the context after destroying them. This lab
does not implement cancellation or concurrent sends. Real hardware ISR/thread
synchronization and platform-specific interrupt handoff are separate work.

After TX and RX pass, try a receive-with-timeout operation, test RX already
containing data before a wait, or support sends larger than TX MEM. RX timeout
here measures inactivity with buffered data, not a timeout waiting for a first byte.
