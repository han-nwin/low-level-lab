# Raw UART: Pico 2 → STM32 Nucleo

Goal

Implement UART communication between:

Pico 2 (RP2350)  ──UART──>  STM32 Nucleo
      TX                        RX
      GND ──────────────────── GND

Do the Pico side using raw memory-mapped I/O.


## Requirements

Pico 2

Configure manually through registers:

* Release required peripherals from reset
* Configure the TX GPIO
    * Set GPIO FUNCSEL to UART
    * Configure the pad as needed
* Configure UART
    * Set baud rate to 115200
    * 8 data bits
    * No parity
    * 1 stop bit (8N1)
    * Enable UART TX
* Implement:

unsafe fn uart_init();
unsafe fn uart_write_byte(byte: u8);
unsafe fn uart_write_str(s: &str);

* Send:

hello from pico

once per second.

STM32 Nucleo

Using its normal HAL is fine.

* Configure one UART RX pin
* Use 115200 8N1
* Receive bytes from the Pico
* Print received data to the PC through the Nucleo’s debug/serial connection

Wiring

Pico TX  ─────────> Nucleo RX
Pico GND ───────── Nucleo GND

Do not connect TX → TX.

Success

PC terminal repeatedly shows:

hello from pico
hello from pico
hello from pico

Stretch Goal

Make it bidirectional:

Pico TX ─────> Nucleo RX
Pico RX <───── Nucleo TX
GND     ────── GND

Have the Nucleo reply:

ACK
