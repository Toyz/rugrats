---
number: 110
title: Spike: getting up, the toy in his mouth, the flinch, the can, the bump
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/hub.rs crates/rugrats/src/actors/npc.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/src/levels/grandpas_teeth.rs crates/rugrats/tests/play.rs
started: 2026-10-15T23:17:16-06:00
took: 2m
---

# 110. Spike: getting up, the toy in his mouth, the flinch, the can, the bump

Part of 54.1, from docs/levels/level-01.md ("Spike" and "The fetch
game"). The hub's Spike lacked several things the original does:

- **Getting up** (0x80083f24): Tommy within 30000 wakes him with
  animation 25 (mode 1, back mode 2), so he gets up and then follows.
  The port went straight to following.
- **The toy in his mouth** (0x800601e4 for char 6): picking a toy up
  gives hierarchy slot 0x24 its model: 678 1212, 679 1214, 680 1213.
  Putting it down (0x80062ac4) gives back his own 79. The port only hid
  the toy, so he carried nothing.
- **The flinch**: a toy that hits him (0x800511a8, char 6) plays
  animation 8, then he goes on.
- **The can**: 716 thrown starts the fetch (0x80060b44 lists 678-680
  and 716), but he never takes it (0x80084cd0: not 716) and gives up
  there.
- **The bump**: bumping into a wall while fetching or bringing (state 9
  for char 6 in level 1, 0x8005b614) gives up (mode 14), putting a
  carried toy down where he stands. `Npc::bumped` records a step taken
  back off a wall or an object.

The one-shot animations play out through the sequencer (91); the
following waits until they end.

The test `hub_spike_fetches_a_toy` follows the oracle's run. Tommy
throws toy 679 from (44000, 700, 36000) facing +z. Spike carries it in
his mouth (1214), brings it back, puts it down (speech 190), and his
mouth is 79 again.

**Still unknown:**
- the mode 2 test "his goal equals Tommy's position exactly" (the port starts a fetch from any following Spike)
- object 80's y = 746, object 518's template swap, the room 4 climb rule and the signs' sparks (54.2)
