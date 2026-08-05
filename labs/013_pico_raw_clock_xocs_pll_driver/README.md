# Lab 013 — RP2350 Raw Clock + 300 MHz PLL Overclock Driver
<https://www.ti.com/video/series/precision-labs/ti-precision-labs-introduction-to-clocks-and-timing.html>
<https://www.analog.com/en/resources/analog-dialogue/articles/phase-locked-loop-pll-fundamentals.html>

Implement the Raspberry Pi Pico 2 system-clock initialization yourself in raw Rust.

You are **not** building a PLL. The RP2350 already contains the oscillator, PLL,
clock muxes, dividers, and reset hardware. Your job is to write the register-level
driver that configures them.

```text
12 MHz crystal
    ↓
XOSC hardware
    ↓
PLL_SYS: 12 MHz / 1 × 125 = 1500 MHz VCO
    ↓                   / 5 / 1
300 MHz PLL output
    ↓
clk_sys mux
    ↓
Cortex-M33 cores + system fabric
```

## Goal

By the end of the lab, this call intentionally configures a 300 MHz system-clock
overclock:

```rust
let clocks = unsafe { clock::init_300mhz_overclock() };
assert_eq!(clocks.sys_hz, 300_000_000);
```

> **Overclock warning:** RP2350 is rated for operation up to 150 MHz. This lab's
> 300 MHz `clk_sys` is outside that specification. PLL lock and a successful FC0
> measurement do not guarantee CPU, SRAM, flash, peripheral, voltage, temperature,
> or long-term stability. Use the 150 MHz configuration for production work.

The implementation must use:

- raw addresses
- volatile reads and writes
- manually defined offsets, masks, shifts, and keyed values
- no PAC
- no HAL
- no Pico SDK clock functions

This lab assumes your existing Pico 2 project already has working RP2350 boot,
linker, vector-table, panic-handler, build, and flashing setup.

---

## Why this lab matters

This lab connects several low-level ideas:

```text
memory-mapped I/O
    + volatile access
    + reset controller
    + oscillator startup
    + PLL frequency synthesis
    + glitchless clock muxes
    + hardware status polling
    = a real MCU clock driver
```

After this lab, “the CPU runs at 300 MHz” should no longer feel like a magical
chip property. It is the result of a hardware configuration performed by code.

---

## Safety rules

1. Call `init_300mhz_overclock()` before enabling interrupts or starting core 1.
2. Never reset PLL_SYS while `clk_sys` is using PLL_SYS.
3. Move `clk_sys` to `clk_ref` before touching PLL_SYS.
4. Poll `SELECTED`; writing a mux selector does not mean the switch is complete.
5. Power the VCO first, wait for `LOCK`, then enable the post-dividers.
6. Establish and verify the rated 150 MHz configuration before enabling the
   intentional 300 MHz overclock.
7. Keep a BOOTSEL recovery path available. A bad clock write can freeze execution.
8. Do not treat a successful blink test as proof that 300 MHz is stable across
   workloads, peripherals, voltage, temperature, or different chips.

---

## Known target configuration

```text
XOSC       = 12 MHz
REFDIV     = 1
FBDIV      = 125
VCO        = 12 MHz / 1 × 125 = 1500 MHz
POSTDIV1   = 5
POSTDIV2   = 1
PLL output = 1500 MHz / 5 / 1 = 300 MHz
```

```text
12 MHz / 1 × 125 / 5 / 1 = 300 MHz
```

The RP2350 PLL constraints relevant to this configuration are:

```text
reference clock: 5–800 MHz
feedback divider: 16–320
VCO:              400–1600 MHz
post-dividers:     1–7 each
```

---

## Project layout

Copy the starter files into your existing Pico 2 bare-metal project:

```text
src/
├── main.rs
├── clock.rs
└── register.rs
```

---

# Phase 0 — Draw the hardware path

Before coding, draw this from memory:

```text
ROSC ───────────────┐
                    ├─ clk_ref mux ───────────┐
XOSC ───────────────┘                         │
                                              ├─ clk_sys mux ─ CPU
PLL_SYS ──────────────────────────────────────┘

clk_sys ── clk_peri mux/divider ── UART / SPI reference clock
```

Answer in your notes:

1. Why can PLL_SYS be reset without stopping the CPU only after `clk_sys` moves?
2. Why does software poll `SELECTED` instead of trusting the `CTRL` write?
3. Why are the post-dividers kept powered down until the VCO locks?
4. Which value is the internal VCO frequency: 300 MHz or 1500 MHz?

---

# Phase 1 — Register-access layer

Implement these primitives in `register.rs`:

```rust
read(address)
write(address, value)
modify(address, clear_mask, set_mask)
set_bits(address, mask)
clear_bits(address, mask)
wait_for_set(address, mask)
wait_for_value(address, mask, expected)
```

### Checkpoint

Use a debugger to read these addresses without changing them:

```text
CLOCKS_BASE  = 0x4001_0000
RESETS_BASE  = 0x4002_0000
XOSC_BASE    = 0x4004_8000
PLL_SYS_BASE = 0x4005_0000
```

---

# Phase 2 — Enable XOSC

Relevant registers:

| Register | Address | Purpose |
|---|---:|---|
| `XOSC_CTRL` | `XOSC_BASE + 0x00` | frequency range + keyed enable field |
| `XOSC_STATUS` | `XOSC_BASE + 0x04` | enabled/stable status |
| `XOSC_STARTUP` | `XOSC_BASE + 0x0c` | startup delay |

Important fields:

```text
XOSC_CTRL.ENABLE      bits 23:12
XOSC_CTRL.FREQ_RANGE  bits 11:0
XOSC_STATUS.STABLE    bit 31
XOSC_STATUS.BADWRITE  bit 24
XOSC_STATUS.ENABLED   bit 12
```

Keyed values:

```text
ENABLE     = 0xfab
DISABLE    = 0xd1e
1–15 MHz   = 0xaa0
```

For a conservative roughly 6 ms startup delay with a 12 MHz crystal:

```text
STARTUP.DELAY = 282
```

### Your work

Implement:

```rust
unsafe fn enable_xosc();
```

Required order:

```text
1. Program the 1–15 MHz range.
2. Program the startup delay.
3. Write the keyed ENABLE value without destroying FREQ_RANGE.
4. Wait until STATUS.STABLE = 1.
5. During debugging, also check BADWRITE = 0.
```

### Checkpoint

Expected status conditions:

```text
STATUS.STABLE  = 1
STATUS.ENABLED = 1
STATUS.BADWRITE = 0
```

Do not continue until this passes.

---

# Phase 3 — Move the CPU to a safe clock path

Relevant clock registers:

| Register | Offset |
|---|---:|
| `CLK_REF_CTRL` | `0x30` |
| `CLK_REF_DIV` | `0x34` |
| `CLK_REF_SELECTED` | `0x38` |
| `CLK_SYS_CTRL` | `0x3c` |
| `CLK_SYS_DIV` | `0x40` |
| `CLK_SYS_SELECTED` | `0x44` |

For `CLK_SYS_CTRL`:

```text
SRC bit 0:
    0 = CLK_REF
    1 = CLK_SYS_AUX

AUXSRC bits 7:5:
    0 = PLL_SYS
    1 = PLL_USB
    2 = ROSC
    3 = XOSC
```

For `CLK_REF_CTRL.SRC` bits 1:0:

```text
0 = ROSC
1 = REF_AUX
2 = XOSC
3 = LPOSC
```

### Your work

Implement:

```rust
unsafe fn select_clk_ref_for_clk_sys();
unsafe fn select_rosc_for_clk_ref();
```

Required sequence:

```text
1. Set CLK_SYS_CTRL.SRC = CLK_REF.
2. Wait until CLK_SYS_SELECTED == 0b01.
3. Set CLK_REF_CTRL.SRC = ROSC.
4. Wait until CLK_REF_SELECTED == 0b0001.
```

This follows the same safety idea used by the Pico SDK: move the glitchless
muxes away from auxiliary PLL sources before changing a PLL.

### Checkpoint

```text
CLK_SYS_SELECTED = 0x1   // clk_ref selected
CLK_REF_SELECTED = 0x1   // ROSC selected
```

---

# Phase 4 — Reset and release PLL_SYS

Relevant reset registers:

| Register | Address |
|---|---:|
| `RESET` | `RESETS_BASE + 0x00` |
| `RESET_DONE` | `RESETS_BASE + 0x08` |

```text
PLL_SYS reset bit = bit 14
```

A `1` in `RESET` holds the block in reset. A `1` in `RESET_DONE` means reset
release has completed.

### Your work

Implement:

```rust
unsafe fn reset_pll_sys();
```

Required sequence:

```text
1. Set RESET.PLL_SYS.
2. Clear RESET.PLL_SYS.
3. Wait until RESET_DONE.PLL_SYS = 1.
```

### Checkpoint

```text
RESET bit 14      = 0
RESET_DONE bit 14 = 1
```

---

# Phase 5 — Configure PLL_SYS

Relevant registers:

| Register | Offset | Purpose |
|---|---:|---|
| `PLL_CS` | `0x00` | lock, bypass, reference divider |
| `PLL_PWR` | `0x04` | PLL/VCO/post-divider power-down bits |
| `PLL_FBDIV_INT` | `0x08` | feedback multiplier |
| `PLL_PRIM` | `0x0c` | output post-dividers |

Fields:

```text
PLL_CS.LOCK       bit 31
PLL_CS.BYPASS     bit 8
PLL_CS.REFDIV     bits 5:0

PLL_PWR.VCOPD     bit 5
PLL_PWR.POSTDIVPD bit 3
PLL_PWR.DSMPD     bit 2
PLL_PWR.PD        bit 0

PLL_FBDIV_INT     bits 11:0

PLL_PRIM.POSTDIV1 bits 18:16
PLL_PRIM.POSTDIV2 bits 14:12
```

### Your work

Implement:

```rust
unsafe fn configure_pll_sys_300mhz_overclock();
```

Required sequence:

```text
1. Write REFDIV = 1.
2. Write FBDIV = 125.
3. Clear PLL_PWR.PD and PLL_PWR.VCOPD.
4. Leave POSTDIVPD set while the VCO starts.
5. Wait until PLL_CS.LOCK = 1.
6. Write POSTDIV1 = 5 and POSTDIV2 = 1.
7. Clear PLL_PWR.POSTDIVPD.
```

Expected encoded post-divider value:

```text
(5 << 16) | (1 << 12) = 0x0005_1000
```

Expected final power register low bits:

```text
PLL_PWR = 0x0000_0004
```

`DSMPD` remains set because this PLL uses integer feedback division.

### Checkpoint

```text
PLL_CS.REFDIV       = 1
PLL_CS.LOCK         = 1
PLL_FBDIV_INT       = 125
PLL_PRIM.POSTDIV1   = 5
PLL_PRIM.POSTDIV2   = 1
PLL_PWR.POSTDIVPD   = 0
```

---

# Phase 6 — Switch to the final clock tree

Implement:

```rust
unsafe fn select_xosc_for_clk_ref();
unsafe fn select_pll_sys_for_clk_sys();
unsafe fn configure_clk_peri();
```

Required sequence:

```text
1. Set CLK_REF divider to 1.
2. Select XOSC as CLK_REF.
3. Wait until CLK_REF_SELECTED == 0b0100.
4. Set CLK_SYS divider to 1.
5. Select PLL_SYS as CLK_SYS AUXSRC.
6. Select CLK_SYS_AUX as CLK_SYS SRC.
7. Wait until CLK_SYS_SELECTED == 0b10.
8. Set CLK_PERI divider to 1.
9. Select CLK_SYS as CLK_PERI AUXSRC.
10. Enable CLK_PERI.
```

### Important mux rule

`AUXSRC` is not glitchless. Set it while that AUX path is not selected, then
switch the glitchless `SRC` field to AUX.

### Final checkpoint

```text
CLK_REF_SELECTED = 0x4   // XOSC
CLK_SYS_SELECTED = 0x2   // AUX, with AUXSRC = PLL_SYS
CLK_PERI enabled  = 1
```

---

# Phase 7 — Compose the public driver

Your final public API should remain small:

```rust
pub struct Clocks {
    pub xosc_hz: u32,
    pub ref_hz: u32,
    pub sys_hz: u32,
    pub peri_hz: u32,
}

pub unsafe fn init_300mhz_overclock() -> Clocks;
```

Recommended high-level sequence:

```text
enable_xosc
    ↓
clk_sys → clk_ref
    ↓
clk_ref → ROSC
    ↓
reset/release PLL_SYS
    ↓
configure and lock PLL_SYS
    ↓
clk_ref → XOSC
    ↓
clk_sys → PLL_SYS
    ↓
clk_peri → clk_sys
```

---

# Phase 8 — Verification

Use at least two verification methods.

## Method A — Register snapshot

Halt with the debugger and inspect:

```text
XOSC_STATUS
PLL_CS
PLL_PWR
PLL_FBDIV_INT
PLL_PRIM
CLK_REF_CTRL
CLK_REF_SELECTED
CLK_SYS_CTRL
CLK_SYS_SELECTED
CLK_PERI_CTRL
```

## Method B — Clock output pin

Stretch goal: route `CLOCK GPOUT0` to GPIO21 and divide `clk_sys` by 300.
The pin should output approximately 1 MHz:

```text
300 MHz / 300 = 1 MHz
```

Measure it using an oscilloscope or logic analyzer.

## Method C — Timing ratio

Run the same fixed instruction loop once with `clk_sys = clk_ref` and once with
`clk_sys = PLL_SYS`. Compare it against a time base derived from `clk_ref`.
The CPU-side loop should become much faster after the switch.

---

# Required deliverables

Commit:

```text
src/register.rs
src/clock.rs
src/main.rs
README.md or NOTES.md containing your register derivation
```

Your notes must include:

1. The complete clock path before and after initialization.
2. The PLL equation with intermediate VCO frequency.
3. Every register address used.
4. Every field mask and shift used.
5. The reason for each polling loop.
6. A register snapshot proving XOSC stable, PLL locked, and muxes switched.
7. Your verification result.

---

# Acceptance criteria

The lab is complete when:

- [ ] no PAC or HAL is used for clock setup
- [ ] all MMIO accesses are volatile
- [ ] XOSC reaches `STABLE`
- [ ] `clk_sys` is moved off PLL_SYS before PLL reset
- [ ] PLL_SYS reset release is confirmed through `RESET_DONE`
- [ ] PLL_SYS reaches `LOCK`
- [ ] post-dividers are enabled only after lock
- [ ] `CLK_REF_SELECTED` confirms XOSC
- [ ] `CLK_SYS_SELECTED` confirms the PLL AUX path
- [ ] `clk_peri` is sourced from `clk_sys`
- [ ] the resulting clock is independently verified
- [ ] unsafe code is isolated inside the raw driver modules

---

# Stretch goals

1. Replace magic masks with a small `Field` helper.
2. Add timeout-based polling so broken hardware does not spin forever.
3. Return a typed `ClockError` instead of hanging.
4. Make a const-evaluated PLL configuration validator.
5. Support another valid frequency generated by `vcocalc.py`.
6. Add `PLL_USB = 48 MHz` initialization.
7. Extend the existing FC0 measurement to report its precise result over RTT or UART.
8. Compare generated assembly with the equivalent C implementation.
9. Replace read-modify-write with RP2350 atomic SET/CLEAR register aliases.

---

# Questions to answer after the lab

1. Is the crystal itself controlled by the Cortex-M33?
2. What physical circuit actually creates 1500 MHz?
3. Why is the CPU clock 300 MHz when the VCO is 1500 MHz?
4. Why is `LOCK` a hardware status bit instead of a fixed software delay?
5. What would happen if you reset PLL_SYS while the CPU is using it?
6. Why can a correct divider equation still violate PLL constraints?
7. Which peripherals depend on `clk_peri`, and what breaks if its assumed rate is wrong?
8. Why does FC0 measuring 300 MHz not prove that the overclock is stable?

---

# Official references

- RP2350 datasheet, especially:
  - Section 2.2 — address map
  - Section 7.5 — subsystem resets
  - Section 8.1 — clocks
  - Section 8.2 — XOSC
  - Section 8.6 — PLL
- https://pip.raspberrypi.com/documents/RP-008373-DS-rp2350-datasheet.pdf
- https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
- https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/hardware_pll/pll.c
- https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/hardware_xosc/xosc.c

Do not blindly copy the SDK. Use it to check your derived sequence after you
have worked from the datasheet.
