---
number: 174
title: The characters as the collision tests see them
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/src/world/game/camera.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T22:31:13-06:00
took: 38m
---

# 174. The characters as the collision tests see them

The other characters now meet the player as the original's collision
tests see them, and two more scripts are played against the original.

**Characters** (`Game::frame`, `boxhit::Exact`): every character object
in the original (the player's own, type 11, too) has a bound cube of
1773 (+0x78) and size 8192 (+0x86, which the code had guessed at 200).
Those whose template has flag 0x2000 (level 3's creatures, 495, 590, 711
...) are met by their posed parts (0x8006a318), the rest (the babies) by
their hierarchy's one box (0x8006a008: header +4 .. +0x10, turned by
their heading). The port hands both to `Ground`: the box from the
hierarchy, the parts from the pose as it draws them (each part's
rotation transposed, its joint, its model's boxes). An NPC's heading is
now its placement's angle (`Trig::placed_angles`), not a float of the
quaternion's.

Level 3's run past the tipped toy keeps the creatures out of it: with
their parts in the way, the port's creature wanders into the path by the
fourth phase where the oracle's does not (the route walkers are the
port's own).

**The bump keeps its speed for its first frame** (+0x184 is zeroed by
the bump's state as it runs, the next frame): the follow camera, which
takes the player as moving by the speed (0x80027304), sees it so. The
slide's camera matches on 66 of its 70 frames (50 before).

**Two more scripts** (`tests/hle.rs`): the standing jump from the hub's
start, and a run into the hub's seat (template 130) and the bump. Both
the same as the original's in x and z on every frame.

**Still unknown:**
- After a bump from rest (the seat) the original's follow camera holds
  still where the port's goes on, and its look-ahead parts by a few
  units: other branches of 0x80027304. The heading the stick sets
  follows the camera, so it differs there too.
- On the first frame from a standstill the original's speed is 0 when
  the way already points off the heading (the seat, 45 degrees) and 17
  when the stick turns the body (the hub's start): the stick handler
  (0x8005be24) is not yet ported to that detail.
- The creatures' wandering (route walkers, posts) is the port's own.
