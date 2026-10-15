---
number: 42
title: Level 11 playable: three rings
date: 2026-10-15
area: play
files: crates/rugrats/src/level11.rs, crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats-data/src/model.rs, crates/rugrats/examples/lp_dump.rs, crates/rugrats/examples/play_script.rs, docs/engine/level-11.md
started: 2026-10-15T07:56:58-06:00
took: 49m
---

# 42. Level 11 playable: three rings

A twelfth delegated map, docs/engine/level-11.md, traced Circus
Angelicus and checked each ring in the oracle. The port now plays it.

**A finding on models:**

- DB11's bone (template 1017, shape 1423) names a model its table does
  not have. model() then answers offset 0, the first model stored, and
  that is what the bone draws.
- The port had dropped such collectibles (`find != 0`). They now take
  `get`'s answer.
- `Models::has` tells a real id from a miss. Pickables and movers use
  it. Class 0 scenery keeps the constructor's offset 0 rule (0x800494d8
  clears the draw).

**Level 11** (`rugrats::level11`), the ring step machine of 0x8005a7cc:

- Ring 1:
  - Spike at (25252, 751, 995) and the 110 s clock. Tommy runs the
    28-post loop.
  - The bone starts at group 2. Each one eaten moves it to the next
    group, 2 .. 7 and round; twelve play cine 54 with tries 3.
  - Spike's moving jump here is -1700 with factor 20 (0x80060f1c), high
    enough for the stands.
  - The clock out plays cine 53, then a try and the ring again.
- Ring 2:
  - Cine 54's end swaps to the tricycle (char 34); cine 55, then ring 2
    with the speed bar.
  - The ramp's zone records the spot. Cross there launches heading 1050
    at 40 x speed x 4, 10 or 16 by the distance (6000, 7500 bands).
  - Landing on the net plays cine 57, else cine 56 and a try.
- Ring 3:
  - Angelica on the tower with a fixed camera. The meter rises 6 a frame
    to 220 and back. The second reading starts at 220 less the first.
  - Cine 39, then Chuckie is launched: heading 2944/3072/3200 and
    forward 2176/2816/3328 by the readings.
  - Good both ways plays cine 62, won; else cine 59/60/61 by tries.
- No cry in any ring (0x80080044): a try, then the ring again.

Checked in scripted runs: a bone eaten and moved to group 3's stand; the
clock running out at 3300 frames, cine 53, two tries, ring 1 again.
Rings 2 and 3 run their code paths but were not scripted end to end.

**Still unknown:**

- The 878 zone's true extent (a box from the seen points here); the net
  test (a 2500 square here).
- The ring 3 air drag at a quarter; the seesaw's tilt; the meter's bar
  drawing.
- The standing jump's no-pickup rule (0x800c4bc4 bit 1) is not ported.
