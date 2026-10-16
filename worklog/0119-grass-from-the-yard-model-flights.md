---
number: 119
title: Grass from the yard: model flights
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T00:19:08-06:00
took: 8m
---

# 119. Grass from the yard: model flights

Floor flag 16 (COL tile byte 4 bit 4) throws up a model, 0x80050b5c. It
uses model 1008, looked up through 0x80044b74; it is a tuft of grass.

The flag is on these floors:

- the hub's yard: y 1408, x -2048..51968, z 24576..44800;
- levels 4, 10, 11, 12, 13 and 18.

Level 4 is left out by the callers.

The flight's launch:

- kind 2, flags 0x10502 (gravity 0x500, world collision);
- the model's radius;
- speed 100, within 2048 of up;
- 2000 ms, bounce 1000, spin 3000.

**Who calls it.** The stand state's handler 0x8005d11c and the walk/run
state's handler 0x8005e1f4 both call it, from two places. Each flips its
own coin first: the high half's bit 0 of an LCG. The coins live at
0x800c42e0 and 0x800c42ec, seeded 0xa8c0 and 0x2fdd as read on the
original. The grass starts at the position of hierarchy slot 16's part,
512 up.

**Checked on the original.** Tommy was put at (-1536, 708, 30000) in the
hub, with UP held. 0x80050b5c ran from 0x8005e4c4 on about every second
game frame, and once from 0x8005d80c while standing. The screenshot shows
small dark green tufts at his feet.

**Model flights.** Flights gain `Skin::Model` (record +0x84 from
descriptor +0x50). They also gain the two other angles: +0x20 spins by
axis +0x4e and +0x22 by +0x50, while +0x24, the roll, spins by +0x4c.
`Flights::models` hands kind 2 to the game. The game draws each one
through `models::model_triangles`, turned by `Trig::euler` of (+0x20,
+0x22, +0x24) as 0x80087afc does through 0x80020ee0.

Test: the_yard_throws_up_grass puts Tommy at the same spot and has him
walk for 40 frames. At least five model 1008 kind 2 flights are in the
air.

**Still unknown:**

- State 9's handler 0x8005d030 throws grass 1024 up in levels 2 and 17
  when +0xf4 is 0. Which port state this is has not been worked out.
- The descriptor leaves +0x14 and +0x18 unset (stack garbage); the port
  starts the angles at 0.
- Kind 4's stretch (0x80087afc with a3 1: +0x92 times 10 as y scale) and
  kind 1 (0x8006d948) are still not drawn.
- The port's picture of the grass has not been compared with the
  original's; the port has no way to start at a given spot.
