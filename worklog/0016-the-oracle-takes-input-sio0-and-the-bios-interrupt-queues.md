---
number: 16
title: The oracle takes input: SIO0 and the BIOS interrupt queues
date: 2026-10-14
area: tooling, input, engine
files: crates/rugrats-hle/src/sio.rs, crates/rugrats-hle/src/hw.rs, crates/rugrats-hle/src/lib.rs, docs/engine/input.md
started: 2026-10-14T21:19:13-07:00
took: 13m
---

# 16. The oracle takes input: SIO0 and the BIOS interrupt queues

The game reads its pads through libpad's direct driver, so the oracle's
BIOS pad buffers never reached it. Two pieces fix that:

- `sio.rs` models the controller port with a digital pad in port 1:
  data, status, mode, control, baud; the pad answers FF 41 5A lo hi and
  pulls /ACK after each byte but the last. The driver clears I_STAT bit 7,
  waits for /ACK to rise, acknowledges, takes the answer, and only then
  polls I_STAT bit 7 for the pulse. With the harness's clock still within
  a slice, the pulse lands at that I_STAT read (or the slice's end). A
  pulse that landed at once was cleared before the poll, and the driver
  gave up after the first byte.
- The BIOS's interrupt queues (SysEnqIntRP/SysDeqIntRP) are kept and run
  before libetc's dispatcher, each entry's func1 then func2. libpad's
  vertical-blank handler is queued there (priority 2) and tests I_STAT
  bit 0 itself, so it must run before libetc acknowledges it.

The buffer at 0x800cdbb0 now reads 00 41 ff ff, and ef ff with Triangle
held. With a pad present the game behaves differently: after the logos
it shows the memory card check and then, on X, the title menu (Start Game,
Load Game, Audio; Start Game leads to Main Game, Activities, Training).
Without a pad it had gone from the logos straight to the opening
cinematic, which is the order the port follows today. That order has to
change once the front end is ported.

**Still unknown:**
- where the opening cinematic plays with a pad in (after Main Game?)
- the memory card's BIOS calls, which the oracle answers as no card
