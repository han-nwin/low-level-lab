# Raw UART Loopback on Pico 2

## Goal

Implement UART communication between the two hardware UART peripherals on one
Pico 2 (RP2350), using raw memory-mapped I/O.

```text
UART1 TX (GPIO4) ──jumper──> UART0 RX (GPIO1)
```

UART1 sends a message, UART0 receives it, and the Pico prints the received data
to the Mac through the existing USB logger.

## Requirements

Configure everything manually through registers:

- Release UART0 and UART1 from reset.
- Configure GPIO4 as UART1 TX (`FUNCSEL = 2`).
  - Connect the pad and allow output.
  - Disable the input path and internal pulls.
- Configure GPIO1 as UART0 RX (`FUNCSEL = 2`).
  - Connect the pad and enable input.
  - Disable output and the pull-down.
  - An RX pull-up is optional because UART idles high.
- Configure both UARTs for:
  - 115200 baud
  - 8 data bits
  - No parity
  - 1 stop bit (8N1)
- Enable UART1 TX and UART0 RX.

Implement:

```rust
unsafe fn uart_init();
unsafe fn uart_write_byte(byte: u8);
unsafe fn uart_write_str(s: &str);
unsafe fn uart_read_byte() -> u8;
```

Send through UART1:

```text
hello from uart1
```

Receive the message through UART0 and print it through the USB logger once per
second.

## Wiring

With the Pico powered off, connect one jumper:

```text
GPIO4 (UART1 TX) ─────────> GPIO1 (UART0 RX)
```

No separate ground jumper is required because both UARTs are on the same board.
Do not configure GPIO1 as an output.

## Success

The Mac terminal repeatedly shows data that was transmitted by UART1 and
received by UART0:

```text
hello from uart1
hello from uart1
hello from uart1
```

## Stretch Goal

Make the loopback bidirectional with a second jumper:

```text
GPIO4 (UART1 TX) ─────────> GPIO1 (UART0 RX)
GPIO0 (UART0 TX) ─────────> GPIO5 (UART1 RX)
```

Have UART0 reply to UART1 with:

```text
ACK
```
