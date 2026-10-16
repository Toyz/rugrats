---
number: 118
title: Landing sounds and the object in focus
date: 2026-10-16
area: audio
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/flights.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T00:14:24-06:00
took: 5m
---

# 118. Landing sounds and the object in focus

The landing (state 10, animation 4) has its own entry at 0x8005b624. After
it starts the animation it picks a sound:

1. If the object in focus (player +0x1e8) is template 505, it plays sound
   102 unplaced (0x8006f26c). The only 505 is level 11's LP 39 at (3681,
   1087, 40474), flags 0x0a.
2. Otherwise, if the object stood on (+0xf8) is template 494, it plays 43
   at the player. These are level 3's LP 103, 104 and 107 (shape 809, at
   about y -2100).
3. Otherwise, on a water tile (+0x8c byte 4 bit 0) in level 4, it plays 57
   at the player.

The player now keeps two things for this:

- `support`: the box whose top is its floor.
- `landed`: set as the landing begins.

Each box carries its object's template. Static objects, things,
breakables and movers set it.

The object in focus is worked out by 0x8006af4c. It is a template flag 8
object in the player's room list. Its cylinder must meet the player's
(0x8006abbc), and its model must meet a vertical segment (0x8006a714). The
segment runs from the root part down by the larger y extent of the
player's bounds. `boxes::focus` stands in with the model's boxes: they
must cover the player's (x, z) and span from the root to the feet.

Test: landing_in_water_splashes_aloud drops Spike 1500 above level 4's
water nearest the start. Sound 57 is heard; a check against 58 fails, so
the test runs. The water test's spot search is now shared as `water_spot`.

**Still unknown:**

- What 505 (level 11) and 494 (level 3) look like in play; neither sound
  has been heard on the original yet.
- Whether +0x8c is still the floor tile when standing on an object above
  water (the port asks the tile under the feet either way).
- Focus uses boxes, not 0x8006a714's face test and 0x8006abbc's cylinder.
