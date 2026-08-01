# RP2350 Clock Register Worksheet

Fill this out before implementing each phase.

| Symbol | Base + offset | Address | Field | Bits | Mask | Encoded value |
|---|---|---:|---|---:|---:|---:|
| XOSC_CTRL | | | FREQ_RANGE | | | |
| XOSC_CTRL | | | ENABLE | | | |
| XOSC_STATUS | | | STABLE | | | |
| XOSC_STARTUP | | | DELAY | | | |
| RESETS_RESET | | | PLL_SYS | | | |
| RESETS_RESET_DONE | | | PLL_SYS | | | |
| PLL_CS | | | REFDIV | | | |
| PLL_CS | | | LOCK | | | |
| PLL_PWR | | | PD | | | |
| PLL_PWR | | | VCOPD | | | |
| PLL_PWR | | | POSTDIVPD | | | |
| PLL_FBDIV_INT | | | FBDIV | | | |
| PLL_PRIM | | | POSTDIV1 | | | |
| PLL_PRIM | | | POSTDIV2 | | | |
| CLK_REF_CTRL | | | SRC | | | |
| CLK_REF_SELECTED | | | selected source | | | |
| CLK_SYS_CTRL | | | SRC | | | |
| CLK_SYS_CTRL | | | AUXSRC | | | |
| CLK_SYS_SELECTED | | | selected source | | | |
| CLK_PERI_CTRL | | | ENABLE | | | |
| CLK_PERI_CTRL | | | AUXSRC | | | |

## Expected final snapshot

```text
XOSC stable:             yes
PLL_SYS locked:          yes
PLL_SYS REFDIV:          1
PLL_SYS FBDIV:           125
PLL_SYS POSTDIV1:        5
PLL_SYS POSTDIV2:        2
clk_ref selected source: XOSC
clk_sys selected source: AUX → PLL_SYS
clk_peri source:         clk_sys
```
