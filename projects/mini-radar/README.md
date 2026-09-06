# Peripheral Docs

### TFT 1.28inch 4-line-SPI IPS Module MSP1281 - GC9A01
<https://www.lcdwiki.com/res/MSP1281/1.28inch_4-line-SPI_IPS_Module_MSP1281_User_Manual_EN.pdf>
- LCD VCC → Pico 3V3
- LCD GND → Pico GND
- LCD SCL → GP2  (SPI0 SCK)
- LCD SDA → GP3  (SPI0 MOSI/TX)
- LCD DC  → GP4  (ordinary output)
- LCD CS  → GP5  (ordinary output)
- LCD RES → GP6  (ordinary output)

### LD2450 sensor
<https://www.tinytronics.nl/product_files/006000_HLK-LD2450-Instruction-Manual.pdf>
- Sensor RX → Pico UART0 TX (GP0)
- Sensor TX → Pico UART0 RX (GP1)
- Sensor 5v → Pico 5V(VCC)
- Sensor GND → Pico GND

---
```bash
# logger
screen /dev/cu.usbmodemA74D15D81

```

---
# Packages

### cortex-m handles:
  - Cortex-M processor support
  - Access to Cortex-M instructions and peripheral abstractions

### cortex-m-rt handles:
  - Cortex-M runtime startup
  - Entry-point and interrupt support

### defmt handles:
  - Compact logging for embedded systems
  - Efficient formatting of debug messages

### defmt-rtt handles:
  - Sending defmt logs to a debug probe over RTT

### embedded-hal handles:
  - Common hardware-independent embedded traits
  - Interfaces shared by HALs and device drivers

### embedded-hal-nb handles:
  - Nonblocking hardware-independent embedded traits
  - The serial Read interface used by the LD2450 UART driver

### panic-halt handles:
  - Halting the processor when a panic occurs

### panic-probe handles:
  - Reporting panic information through defmt when used as the panic handler

### rp235x-hal handles:
  - RP2350 peripheral abstractions
  - GPIO, clocks, UART, SPI, timers, resets, and other hardware
  - Runtime, critical-section, binary-info, and defmt integration

### rp-usb-serial handles:
  - USB CDC serial communication on the RP2350
  - Sending project logs to a computer over USB

### nb handles:
  - Nonblocking operation results
  - The WouldBlock state used by UART reads

### mipidsi handles:
  - GC9A01 initialization
  - Reset and sleep/display commands
  - Drawing-window commands
  - RGB565 pixel transmission
  - Rotation and orientation
  - Implementing the graphics DrawTarget interface

### embedded-graphics provides:
  - Lines
  - Circles
  - Rectangles
  - Text and fonts
  - Colors
  - Styles and strokes
- Iterators of pixels

### embedded-hal-bus handles:
  - Converting an SPI bus and chip-select GPIO into an SPI device
  - Managing the LCD chip-select signal for each SPI transaction

### tinytga handles:
  - Parsing TGA image data in `no_std` firmware
  - Drawing compile-time embedded TGA assets through embedded-graphics
