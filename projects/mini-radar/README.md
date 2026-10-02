# Improvement Checklist

Learning order: migrate to Embassy async on one core first, learn how interrupts wake async tasks, then explore multicore. Finishing the manual UART interrupt implementation with the current HAL is not a prerequisite.

### First goal: switch to Embassy async

- [ ] Migrate startup and peripheral setup to `embassy-rp` and an Embassy executor on a single core. Select its multicore-capable critical-section implementation as the only provider.
- [ ] Start with an async UART receive task and a timer-driven LED task. Use Embassy's buffered UART driver and its required interrupt bindings; reuse the synchronous parser and target calculations.
- [ ] Trace how an awaited UART read returns `Pending`, how the driver's interrupt handler wakes the task, and how the executor polls it again.
- [ ] Practice a raw interrupt handler alongside Embassy using a separate peripheral or interrupt not already owned by an Embassy driver; hand events to a task through a channel.
- [ ] Bring display updates and USB into the async application. Keep the latest targets separate from rendering and use async timers for periodic updates.
- [ ] Adapt the synchronous display transfer path so SPI transfers can yield. Adding `async fn` alone will not make the current drawing calls nonblocking.

### Reliability and responsiveness

- [x] Fix frame parsing so a frame is accepted only after both footer bytes (`0x55 0xCC`) are validated. Test valid, truncated, and malformed frames, plus recovery after invalid data.
- [ ] Verify that Embassy's buffered UART reception continues during display updates. Parse bytes in the sensor task, define overflow recovery, and size the receive buffer using measured delays.
- [ ] Keep the latest parsed targets separate from rendering so the display does not build up a backlog of old sensor frames.
- [ ] Update the display and sweep animation on a timer, independently of sensor reports. Start with a target of 20–30 FPS and adjust after measuring rendering time.
- [ ] Reduce display work by restoring only the background regions affected by old targets and the sweep instead of redrawing the entire radar grid each frame.
- [ ] Measure redraw duration, UART overruns, and ring-buffer overflows to check whether changes improve responsiveness and reliability.
- [ ] Keep USB tasks responsive and throttle target logs so logging does not delay sensor processing or display updates.

### After async: use both RP2350 cores

- [ ] Move display rendering to core 1 while core 0 handles UART reception, sensor parsing, and USB. Give each peripheral a single owner and configure its interrupts on the owning core.
- [ ] Share the latest target state using `critical_section::Mutex` and `critical_section::with()`, or an Embassy synchronization primitive using `CriticalSectionRawMutex`. Use the chosen HAL's multicore-capable critical-section implementation and keep only one implementation enabled; do not enable `cortex-m`'s `critical-section-single-core` feature.
- [ ] Keep shared-state access brief: copy the latest targets inside the critical section, then render outside it. Verify UART reliability and display responsiveness with both cores running.

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

### critical-section handles:

  - Shared-state protection through `critical_section::Mutex` and `critical_section::with()`
  - Uses `rp235x-hal`'s enabled `critical-section-impl` feature for local interrupt masking and a hardware spinlock to coordinate both cores
  - Keep this HAL implementation as the only provider; leave `cortex-m`'s `critical-section-single-core` feature disabled

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
  - The multicore-capable critical-section implementation (this does not start core 1)

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
