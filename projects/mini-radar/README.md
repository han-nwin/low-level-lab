# Improvement Checklist

### Reliability and responsiveness

- [x] Fix frame parsing so a frame is accepted only after both footer bytes (`0x55 0xCC`) are validated. Test valid, truncated, and malformed frames, plus recovery after invalid data.
- [ ] Receive UART bytes in an RX interrupt and feed a ring buffer; parse buffered bytes in the main loop so reception continues during display updates. Define overflow handling and size the buffer using measured delays.
- [ ] Keep the latest parsed targets separate from rendering so the display does not build up a backlog of old sensor frames.
- [ ] Update the display and sweep animation on a timer, independently of sensor reports. Start with a target of 20–30 FPS and adjust after measuring rendering time.
- [ ] Reduce display work by restoring only the background regions affected by old targets and the sweep instead of redrawing the entire radar grid each frame.
- [ ] Measure redraw duration, UART overruns, and ring-buffer overflows to check whether changes improve responsiveness and reliability.
- [ ] Keep USB polling responsive and throttle target logs so logging does not dominate the main loop.
- [ ] Evaluate Embassy async tasks for sensor reception, display updates, USB, and future motor control after the basic improvements. The current synchronous display path would need adaptation for SPI transfers to yield; adding `async fn` alone will not make drawing nonblocking.

### Future: rotate the radar to a chosen angle

Planned as physical rotation of the sensor on a motorized mount.

- [ ] Choose a positioning mechanism (such as a servo or stepper with a driver), mount, power supply, and available control pins based on the required angle range and accuracy.
- [ ] Add a way to request an angle in degrees, initially through USB serial, and display the requested angle.
- [ ] Define the zero-angle reference, calibrate travel limits, and reject commands outside the supported range.
- [ ] Implement nonblocking movement control so UART reception, display updates, and USB remain responsive while turning.
- [ ] Track commanded angle separately from measured or estimated position; add homing or position feedback if the mechanism requires it.
- [ ] Account for the sensor's orientation when drawing targets in a fixed reference frame, and define how readings taken during movement are handled.

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
