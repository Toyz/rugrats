---
title: The input module
status: partial
discs: US
covers: US SLUS_006.50:0x8001acc0, US SLUS_006.50:0x8001baf8, US SLUS_006.50:0x8001b028, US SLUS_006.50:0x800cda18
worklog: 6, 16
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
0x800cdbbc). 0x80098178 queues a vertical-blank handler with the BIOS
(SysEnqIntRP(2, 0x800e7f30): func1 0x80097f6c tests I_MASK and I_STAT bit
0, func2 0x80097fd4 does the work). Each exchange (0x80098288) selects the
port (SIO0 control 0x1003, or 0x3003 for port 2), writes a byte, clears
I_STAT bit 7, waits for /ACK to rise (STAT bit 7, 0x80098b34),
acknowledges (control | 0x10), waits for the answer (STAT bit 1,
0x80098bc4), then polls I_STAT bit 7 for the pad's next /ACK with a
timeout on root counter 2 (0x8009a928, 0x8009a948). A digital pad's
answer lands in the buffer as 00 41 lo hi.

The oracle models SIO0 that way (`rugrats-hle` sio.rs) and runs the BIOS
interrupt queues. With a pad in port 1 the boot goes from the logos to the
memory card check ("There is no Memory Card in Memory Card slot 1") and
on X to the title menu (Start Game, Load Game, Audio; Start Game: Main
Game, Activities, Training). With no pad it went from the logos to the
opening cinematic.

## Unknown

- The action list and the six maps' contents.
- The status word's meaning beyond 0 = present.
