# UART interrupt → async operation lab

The driver in **uart.cxx** and **uart.hpp** bridges UART interrupts to Asio.
TX is caller-driven; RX continuously captures into a software ring.

```sh
make                 # C++20 and Boost headers; defaults to /opt/homebrew
./uart_lab           # TX checks
./uart_lab --rx      # RX exercise: background capture, ring reads, and overlapping TX
make test            # Hardware tests + UART RX implementation checks
make test-hardware   # Supplied hardware checks, independently of the driver
make test-rx         # Continuous RX/ring-buffer checks
make compare         # Compare async send with a blocking send and a middle task
```

`make compare` sends the same 20-byte payload both ways and schedules a
nonblocking 50 ms middle task to start 50 ms into the send. The async version
lets that task run while `co_await` is suspended. The blocking version runs the
simulated UART and IRQ on a hardware thread, but blocks the application thread
until completion, so its middle task runs afterward. The summary reports when
the middle task starts and finishes, each send wait, and total time until both
are done. Async overlaps the task with the transfer; it does not make UART
transmission faster.

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

## Driver flow

1. Constructor: reset the UART and enable it. Main supplies interrupt wiring.
2. `asyncTxWait` / `asyncRxWait`: use `async_initiate` to obtain and save a completion handler.
   TX unmasks its hardware interrupt and checks live completion status. RX waits
   for software-ring data without changing hardware masks. Templates live in the
   header so callers can instantiate them for their completion token.
3. `send`: set TX_OFFSET, set TX_BYTE_NUMBER, fill TX MEM, signal START,
   then await FIFO-empty **and** TX_BYTE_COUNT == TX_BYTE_NUMBER. Limit each message to 256 bytes; throw for longer messages.
   One send at a time; background RX and one application read may overlap TX.
4. Interrupt callback: inspect pending/unmasked events and acknowledge them.
   For TX_EMPTY, compare TX_BYTE_COUNT with TX_BYTE_NUMBER and check live FIFO
   status. Complete only when the counts match and FIFO is empty. On intermediate
   empty events, retain the handler and leave TX_EMPTY enabled; hardware refills
   automatically. Completion masks the event, removes any stale latch,
   moves out the handler, and posts its invocation to the executor exactly once.
5. `read(buffer)`: consume available ring bytes, waiting only when the ring is
   empty. Background RX captures and rearms independently of application reads.

Each direction has its own wait and completion functions:
`asyncTxWait` → `completeTxWait`, and `asyncRxWait` → `completeRxWait`.
TX completion masks/acknowledges TX_EMPTY and posts `txHandler_`. RX completion
only posts `rxHandler_`; it must not mask RX or stop capture. `handleRxInterrupt`
owns RX acknowledgement, chunk publication, and rearming. There is no shared
`EventType`: TX and RX have separate handlers, and the ISR services both.

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
  → your asyncTxWait initiation lambda saves Asio's handler and unmasks TX_EMPTY
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
The RX runner sends 50 bytes before the application calls `read()`, then checks
that reads of size 10 and 100 return 10 and 40 bytes respectively. It also
checks binary reception during TX and that capture remains armed afterward.

## Continuous RX ring

```text
wire → hardware FIFO → RX memory → completed chunk → software ring → read()
```

The constructor calls `armRxChunk()` to start background capture. It programs
RX_OFFSET = 0 and RX_BYTE_NUMBER = RX_CHUNK_BYTES (256), issues START, and enables
RX_EMPTY | RX_TIMEOUT while preserving TX/config fields.

`handleRxInterrupt(active)` must distinguish intermediate FIFO drains from a
finished chunk. On intermediate RX_EMPTY, acknowledge and continue waiting.
On full count or timeout with RX_BUSY clear, copy RX_BYTE_COUNT bytes from RX
memory into the software ring exactly once, acknowledge the handled sources,
rearm the hardware, then notify any waiting reader. This must run even without
an application read. `handleInterrupt()` must not return after handling TX if
RX events also need service.

The provided fields describe a 512-byte ring:

- `rxHead_`: next byte to read.
- `rxSize_`: number of queued bytes; insert at `(rxHead_ + rxSize_) % RX_RING_CAPACITY`.
- `rxDropped_`: cumulative count of incoming bytes discarded when the ring is full.

Overflow policy: preserve existing queued bytes and drop new bytes. Always
rearm hardware, even when the ring is full. `rxBuffered()` and `rxDropped()`
expose these diagnostics on the event-loop thread. Hardware FIFO overrun and
framing errors are separate from software-ring drops; their IRQs remain masked
in this exercise.

```cpp
std::array<std::uint8_t, 100> out;
auto count = co_await uart.read(out); // Some available bytes, up to 100.
```

`read` returns `min(out.size(), queuedBytes)`. If empty, wait for a published
chunk, then consume available bytes. There is no output-size limit tied to RX
memory: a 1024-byte output may receive any available ring bytes. Empty output
returns 0 without altering state, even while another nonempty read waits.
Leave the unused output suffix untouched. The caller keeps its buffer alive.

Use `readPending_` to reject a second overlapping nonempty read with
`std::logic_error`. Keep it set until the first read actually resumes and
consumes bytes, not merely until its handler is posted. Clear it on success
or error. `asyncRxWait()` stores the reader's handler and checks the ring for
already-available data; it must not reprogram hardware. `completeRxWait()` moves
and posts that handler exactly once, keeping RX interrupts enabled.

Hardware inactivity timeout publishes a partial chunk; it is not an application
read deadline. No incoming bytes means the reader continues waiting. A byte in
RX memory is not yet visible to `read()` until its chunk fills or times out.
The ring decouples capture from consumption, but cannot prevent loss in the
hardware stop/rearm gap. This simulator rejects arrivals while unarmed; a real
board needs its own buffering/flow-control strategy.

### Verification

The RX path is implemented in `armRxChunk`, `handleRxInterrupt`, `asyncRxWait`,
`completeRxWait`, and `read`. TX uses independent `txHandler_` state. All activity remains
on one executor thread; support one send and one read at a time, concurrently.

`make test-rx` runs nine cases with two-second watchdogs: background prebuffering,
pending readers, full chunks/disabled timeout, binary ring wrap, overflow and
recovery, simultaneous TX/RX, overlapping-read rejection, config/mask preservation,
and empty reads. The RX implementation passes these checks.
`make test-hardware` independently checks the unchanged RX memory engine and
selective acknowledgement contract.

## Hardware register contract

This is a fictional UART, with explicit rules for this exercise. Access it with
`hw_.read(Reg::CONTROL_0)`; update CONTROL_1 fields with the helpers below.
Methods model register side effects; they aren't plain RAM fields.

| Offset | Register | Access / meaning |
|---|---|---|
| 0x00 | CONTROL_0 | Live status and commands; bits below |
| 0x04 | CONTROL_1 | Packed baud divisor, 7-bit IRQ mask, and framing; layout below |
| 0x08 | IRQ_STATUS | Read-only latched notifications, bits [6:0], including masked sources |
| 0x0c | Reserved | Old FIFO pop register removed; accesses throw |
| 0x10 | IRQ_CLEAR | Write-one-to-clear selected notifications, bits [6:0]; reads zero |
| 0x14 | RX_TIMEOUT_MS | Default 100 ms; 0 disables RX timeout |
| 0x18 | TX_OFFSET | Read/write start offset in **bytes**, default 0 |
| 0x1c | TX_BYTE_NUMBER | Read/write transfer length in bytes, default 0 |
| 0x20 | TX_BYTE_COUNT | Read-only bytes consumed from TX MEM into FIFO since last START |
| 0x24 | TX_COMMAND | Write `txcommand::START` (bit 0) to start; reads zero |
| 0x28 | RX_BYTE_NUMBER | Read/write requested RX length, 0–256; does not start transfer |
| 0x2c | RX_BYTE_COUNT | Read-only bytes committed to RX memory since START |
| 0x30 | RX_OFFSET | Read/write start offset in RX memory, in bytes |
| 0x34 | RX_COMMAND | Write `rxcommand::START` to arm transfer; reads zero |
| 0x100–0x1fc | TX MEM | 64 read/write 32-bit words; 256 payload bytes |
| 0x200–0x2fc | RX MEM | 64 read-only 32-bit words; 256 captured bytes |

| CONTROL_0 bit | Name | Meaning |
|---|---|---|
| 0 | RX_EMPTY | Read-only: no received bytes |
| 1 | RX_FULL | Read-only: all 16 RX slots occupied |
| 2 | TX_EMPTY | Read-only: TX FIFO is empty, even if more memory remains or UART is shifting |
| 3 | TX_FULL | Read-only: all 16 TX FIFO slots occupied |
| 4 | Reserved | Reads zero; writes ignored |
| 5 | ENABLE | Read/write peripheral enable |
| 6 | UART_RESET | Write 1 resets RX FIFO, TX memory/config/count, mask, events, timeout and enable |
| 7 | RX_TIMEOUT | Read-only: latched RX timeout |
| 8 | TX_BUSY | Read-only: memory transfer or shift register still active |
| 9 | RX_BUSY | Read-only: RX engine armed or receiving; clear after full/timeout completion |

UART_RESET reads as zero. Read-only status bits ignore writes. CONTROL_0
writes set ENABLE from bit 5; RESET takes precedence and leaves UART disabled.
Acknowledgement uses a separate register and does not write CONTROL_0.

### Selective interrupt acknowledgement

IRQ_STATUS, IRQ_CLEAR, and CONTROL_1's seven-bit IRQ-enable field share this layout:

| Bit | Source | Notification trigger |
|---|---|---|
| 0 | RX_EMPTY | Hardware drains staging FIFO into RX memory; also zero-length RX START |
| 1 | RX_FULL | All 16 staging FIFO slots fill |
| 2 | TX_EMPTY | TX FIFO drains; also latched by zero-length START |
| 3 | TX_FULL | TX FIFO fills |
| 4 | RX_ERROR | Byte rejected because RX FIFO is full, or test-bench injected error |
| 5 | FRAME_ERROR | Test-bench injected framing error |
| 6 | RX_TIMEOUT | Partial transfer goes inactive for the configured interval; engine stops |

```cpp
// Acknowledge TX-empty only. RX notifications remain latched.
hw_.write(Reg::IRQ_CLEAR, irq::TX_EMPTY);
// Acknowledge two selected RX sources.
hw_.write(Reg::IRQ_CLEAR, irq::RX_EMPTY | irq::RX_TIMEOUT);
const auto active = hw_.read(Reg::IRQ_STATUS) &
                    control1::irqMask(hw_.read(Reg::CONTROL_1));
```

Writing a 1 clears that source's latch; a 0 leaves it unchanged. Writing zero
is a no-op; writing `irq::ALL` deliberately clears all seven. Bits above bit 6
are ignored. IRQ_CLEAR reads zero, so use a direct write, not read/modify/write.
IRQ_STATUS is read-only and includes masked notifications. Clearing does not
change FIFO data, byte counts, ENABLE, interrupt masks, or live FIFO status.
Clearing RX_TIMEOUT removes its latched CONTROL_0 status too.

Pending enabled sources queue an interrupt on Asio. A queued callback checks
current pending bits before delivery; clearing TX does not suppress pending RX.
Sources remain latched until selectively acknowledged, reset, or cleared by
starting their next transfer (TX START clears TX_EMPTY/TX_FULL; RX START clears
RX sources). A still-true FIFO condition does not immediately relatch an edge.
RX_EMPTY is a staging FIFO drain event, not necessarily transfer completion.
Use RX_BYTE_COUNT == RX_BYTE_NUMBER and !RX_BUSY for full completion, or
RX_TIMEOUT for a partial snapshot. RX_FULL is only staging FIFO pressure;
software does not pop its data. Hardware owns FIFO-to-memory movement.

`injectRxErrors(irq::RX_ERROR | irq::FRAME_ERROR)` lets the test bench inject
error notifications without modifying FIFO data or counts. It accepts only
those two bits and returns false when disabled. Framing/parity waveforms are
not simulated. Rejecting a byte at a requested-length limit is flow
control in this simulator, not a FIFO-overrun error.

Each accepted RX arrival restarts inactivity timing. Reaching the requested
memory count cancels it; FIFO draining does not. Timeout commits any remaining
staging bytes, stops capture, and latches RX_TIMEOUT. Clearing that notification
does not restart capture or its timer. Disabling UART pauses both engines,
cancels timing, and preserves memory/FIFOs/counts. Re-enabling resumes staging
movement and restarts RX timing if a partial transfer had data. Reset cancels
queued movement, clears both memories and FIFOs, and disables the UART.
The test bench's transmission history is preserved across reset.

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
IRQ-mask bits 4 and 5 enable RX_ERROR and FRAME_ERROR. RX_TIMEOUT uses
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

// Read latched events independently of live FIFO status.
const auto pending = hw_.read(Reg::IRQ_STATUS);
const auto mask = control1::irqMask(hw_.read(Reg::CONTROL_1));
```

A raw `write(CONTROL_1, irq::TX_EMPTY)` would now change the baud divisor and
clear the interrupt mask. Place event bits in the field with a read/modify/write, preserving the other settings. This is read/modify/write on the
lab's single thread; independently executing code would need synchronization.

## Prepare TX memory, then signal transmission

This simulator uses your requested sequence. These are lab-defined registers,
not a verified map of your board. Both RX and TX FIFOs have 16 slots
(`FIFO_CAPACITY`). TX MEM and RX MEM are separate 256-byte windows (`TX_MEM_BYTES`, `RX_MEM_BYTES`).

Example register sequence for sending `"hello"`:

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
co_await asyncTxWait(asio::use_awaitable);
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
Keep Hardware alive while using the view. `send()` packs bytes using the same
pattern shown above.
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
is queued. Starting before `co_await asyncTxWait(...)` is supported. In both the
initiation lambda's already-ready check and the ISR, require live TX_EMPTY AND
TX_BYTE_COUNT == TX_BYTE_NUMBER. Checking just FIFO-empty or just count is wrong:
FIFO can empty mid-transfer, and count may reach length while FIFO still has data.
On real hardware with independently executing ISRs, save the handler and arm
completion before START; that ordering belongs in the initiation lambda.

## RX memory transfer engine

This is a lab model inspired by a UART with a hardware transfer engine, not a
verified register map or timing specification for a particular MUART.

```text
incoming bytes → 16-byte staging FIFO → hardware transfer → 256-byte RX MEM
                                                    ↓
                                             RX_BYTE_COUNT
```

Program a byte offset and requested count, then issue START:

```cpp
hw_.write(Reg::RX_OFFSET, 0);
hw_.write(Reg::RX_BYTE_NUMBER, RX_CHUNK_BYTES);
hw_.write(Reg::RX_COMMAND, rxcommand::START);
// Background ISR publishes completed RX memory into the software ring.
// Application read() waits on that ring, independently of this START.
```

START validates offset + length <= 256, resets the running count, clears RX
notifications, and arms reception. Configuration writes alone never start RX.
Starting while disabled or already active throws. Offset/length writes during
an active transfer (including while paused) throw. A zero-length START completes
immediately and latches RX_EMPTY; an empty application read simply
returns 0 without issuing START.

`injectRx()` models an incoming byte. It rejects bytes while disabled, before
START, after completion/timeout, beyond the requested length, or when the
staging FIFO is full. A full-FIFO rejection latches RX_ERROR. Hardware queues
FIFO-to-memory movement on the executor; the byte count increases only when
bytes are committed. Each drain latches RX_EMPTY, including intermediate drains.
The FIFO may therefore be empty while RX_BUSY is still set and count is short.
RX_FULL indicates actual FIFO capacity and is not the receive completion source.

Enable RX_EMPTY | RX_TIMEOUT for this exercise. Acknowledge an intermediate
RX_EMPTY and keep waiting. Publish a chunk when count == requested and
RX_BUSY is clear, or when RX_TIMEOUT is latched. Copy it to the ring before
rearming; then post a pending reader. Clear only handled sources through IRQ_CLEAR.

RX timeout starts after the first byte and measures inactivity between arrivals,
not FIFO occupancy. If 37 of 100 requested bytes arrive and the peer goes quiet,
hardware stops with count 37 and latches RX_TIMEOUT. Late bytes are rejected,
so software can safely copy a stable result. No first-byte deadline is provided.

Read memory words using `hw_.read(Hardware::rxMemWord(index))`. The first byte
occupies bits 7:0, then 15:8, 23:16, and 31:24. RX_OFFSET is a byte offset and may
be unaligned: byte offset 3 lies in bits 31:24 of word 0. Reads never pop data or
change counts; writes to RX MEM or RX_BYTE_COUNT throw. The old RX_DATA register
at 0x0c is reserved. Software never drains the hardware FIFO directly; it copies completed memory
chunks into the separate software ring.

RX memory persists across START, so only the count's bytes are valid for the
current transfer; bytes outside that range may be from an older transfer.
Reset clears the entire hardware memory. Background capture uses 256-byte
chunks, independent of application read size. The software ring retains bytes
across hardware START commands.

## Scope and next experiments

All calls and callbacks use one io_context thread. Keep Uart and Hardware alive
while io.run() runs; do not restart the context after destroying them. This lab
does not implement cancellation or multiple concurrent sends/reads. One send
and one read may overlap while RX capture runs continuously. Real hardware ISR/thread
synchronization and platform-specific interrupt handoff are separate work.

After TX and RX pass, try a first-byte deadline, support continuous RX with double buffering, or support sends larger than TX MEM. RX timeout
here measures inactivity during a partial memory transfer, not a timeout waiting for a first byte.
