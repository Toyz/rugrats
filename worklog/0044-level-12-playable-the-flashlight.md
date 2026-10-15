---
number: 44
title: Level 12 playable: the flashlight
date: 2026-10-15
area: play
files: crates/rugrats/src/level12.rs, crates/rugrats/src/game.rs, crates/rugrats/src/npc.rs, docs/engine/level-12.md
started: 2026-10-15T08:50:30-06:00
took: 18m
---

# 44. Level 12 playable: the flashlight

A fourteenth delegated map, docs/engine/level-12.md, traced Let There Be
Light and checked it in the oracle. The tracer found moving between the
house's floors in the oracle needs the room byte and +0x13c/+0x148 poked
too. The port now plays the level.

**Framework:**

- Level 12's Tommy is hierarchy 44 and its Spike 45 (0x8005981c,
  0x80048284). The player's LP is found by that body.
- A cine can drop the carried thing where the player stands.
- A guard's bite window is now the reach + 600 (at least 1200): the
  Shadow monsters hover about 1500 up.

**Level 12** (`rugrats::level12`):

- Cine 66's end: Spike hidden. Tommy at (29611, -5365, 15428) with the
  flashlight (template 933) in hand: 120 charge on gauge 360/361.
- Shadow monsters (3333-3340) guard 10000 with reach 1024 and bite 20.
- Holding Triangle shines: 1 charge a frame, sound 161 on the press.
  A monster within 4096 ahead and 700 across is struck (sound 110).
  18 frames later it comes back at a random group 10 spot, not marked
  used (0x80083be0, 0x8004fe14).
- Batteries are taken within 1000 by the trigger pass: +30, clamped to
  the gauge's max.
- Triangle by the refrigerator (601, 602) drops the light and plays
  cine 67. After card 705, Spike waits at a random group 0 spot.
  Triangle by him plays cine 68, won.
- The gauge's max is 100 after picking the light up again. A cry has no
  respawn.

Checked in scripted runs:

- A monster in the light, struck and away.
- The charge falling one a frame while Triangle is held.
- Triangle at the refrigerator asking for cine 67. Spike then at spot 2
  (19440, -5808, -2999), as in the oracle.

**Still unknown:**

- The light's true width and its moving reach (0x8009134c); the beam is
  not drawn.
- The hand models 1690 / 1310; Circle putting the light down (generic,
  not checked); level 12's room 4 climb refusal.
