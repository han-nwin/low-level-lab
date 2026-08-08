[ ] Find SPI0/SPI1 base address
[ ] Enable SPI clock/reset
[ ] Pick GPIO pins
[ ] Set GPIO function select (MUX)
[ ] Configure SPI CR0
[ ] Configure SPI CR1
[ ] Configure clock divider
[ ] Enable SPI
[ ] Write transfer()
[ ] Toggle CS
[ ] Talk RFID protocol

## USB logging (without a debug probe)

`src/logging.rs` configures the Pico 2's USB port as a CDC serial device. Call
`logging::init()` once, call `logging::poll()` at least every few milliseconds,
and write messages with `log!(...)` or `logln!(...)`.

Flash, start the firmware, and attach the current terminal to its USB serial
port:

```sh
cargo run --release
```

The Cargo runner invokes `picotool`, waits for `/dev/cu.usbmodem*` to appear,
then opens it with macOS `screen`. Exit the terminal with `Ctrl-A`, then
`Ctrl-\\`.

To connect manually instead:

```sh
ls /dev/cu.usbmodem*
screen /dev/cu.usbmodemXXXX 115200
```

The baud rate is ignored by USB CDC.
