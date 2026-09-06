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
