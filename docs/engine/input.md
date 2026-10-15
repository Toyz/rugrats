---
title: The input module
status: partial
discs: US
covers: US SLUS_006.50:0x8001acc0, US SLUS_006.50:0x8001baf8, US SLUS_006.50:0x8001b028, US SLUS_006.50:0x800cda18
worklog: 6
---

# The input module

The game reads the pads through the BIOS driver (InitPAD/StartPAD fill two
receive buffers at 0x800cdbb0, 12 bytes a port, every vertical blank) and
keeps its own state at 0x800cda18: per port 21 words.

```
word 0       the port's status
words 1-16   button histories: shifted left once a sample, the button's
             current bit in bit 0 (bit order: Select, L3, R3, Start, Up,
             Right, Down, Left, L2, R2, L1, R1, Triangle, Circle, Cross, Square)
words 17-20  the sticks (127 for a digital pad)
```

`0x8001acc0` samples both ports: a port whose status byte is not 0 is left
alone; otherwise its button word is inverted and each bit shifted into its
history; an analog pad's stick bytes are stored and set flags at 52 and 203.

`0x8001baf8(state, mode)` finds a button over both ports, 1-based, the
second port's answer replacing the first's: mode 0 pressed this sample
(history & 3 == 1), 1 released (== 2), 2 held (bit 0), 3 not held.

`0x8001b028(port, map, ...)` binds the game's actions to buttons through
one of six controller maps at 0x800b44ac ("Controller Map %d"): each starts
with a pad type byte (0x41 digital), then a slot index per action; the
action pointers land at 0x800c496c onward.

## The pad driver

The game uses libpad's direct driver, not the BIOS's InitPAD/StartPAD:
0x80099e08 installs its handlers and the two receive buffers (0x800cdbb0,
0x800cdbbc) and talks to the pads through the serial port at each
vertical blank. The oracle models no serial port, so the buffers never
fill and it cannot be given input yet.

## Unknown

- The action list and the six maps' contents.
- The status word's meaning beyond 0 = present.
