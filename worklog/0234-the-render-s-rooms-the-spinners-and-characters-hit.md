---
number: 234
title: The render's rooms, the spinners' and characters' hit callbacks, the flights' marks and splashes, the shared count
date: 2026-10-19
area: engine
files: crates/rugrats/src/world/game/visible.rs, crates/rugrats/src/world/game/emitters.rs, crates/rugrats/src/world/game/toys.rs, crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/media/cine.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T20:52:28-06:00
took: 3h 33m
---

# 234. The render's rooms, the spinners' and characters' hit callbacks, the flights' marks and splashes, the shared count

The hit callbacks beyond the class 10 objects, the render's walk of rooms and portals, the flights' marks and splashes, and the shared sound count now all run in the port as in the original.

**Hit callbacks on other objects.** Any template with +0x09 gets +0x50 and the +0x98 record (`emitters::record`).
- The spinners (0x8004e2ac) call it each think. Their think also changed:
  - it runs only while scheduled by the trigger (the port had used its own 2D distance);
  - and only while drawn in the last six frames (+0x60);
  - the angles turn by the VSyncs since the last think (+0x64), not two a frame.
- Callback 1 (0x80053604, levels 3 and 16): model 932 along the spinner's turning angle, and sound 150/178 off the shared count.
- The characters (0x80084084) call it at each think's start, from where they are then:
  - callback 11, the level 5 robots' glow (its own LCG, % 250);
  - callback 8, level 2's template 20.
- Character 23 sounds 51 at its event 0x8000000.

**The render's rooms** (`world::game::visible`, 0x80034058 -> 0x80033ac4):
- the camera's room, the portals (now in `Set`), the stamps;
- the 65000 cut, the near-portal box (0x800307f8);
- the z-0 cut and projection (0x8003311c, 0x80033278);
- the first room's list, and the depth bands.

It gives the spinners' draw stamp (0x80031b84 against the room's rectangle) and runs the kind 16 collectibles' think from their draw (0x800324a0: level 4's 1141 sound 146).

npc_check VISDBG=1 compares the rooms and rectangles each frame. With the follow camera synced (new `Game::sync_follow`), all frames match in 24 levels; levels 20, 26 and 28 differ only through the tool's camera inputs.

**Flights meeting the world:**
- They call their +0x80. 706's drops carry the object's +0x5c, the table 0x800b7ea4 by template +0x0b. 2 is the splash 0x800529cc, five sprites 275 about the normal; 1 is the pie's splat.
- They make their mark in the pool:
  - 8 marks, newest first, the oldest taken when full;
  - +0x66's marks last, the rest are drawn once;
  - 209, 212 and 218 run sequences, and +0x65 makes them semi-transparent.

**The shared count 0x800c4ba0:**
- the sliding doors' think (0x8004e458) takes dt off it;
- the level starts set it: 1, 7 and 29 100; 3 and 24 1000; 4 and 23 500;
- kind 16 collectibles use it.

**Smaller fixes:**
- Level 16's sparklers' trigger radius is the template's 15000. The set-up's 2500 is overwritten later.
- Their sparkles, the hub signs' and the lobster's turn by the object's matrix.
- The class 10 enter beats a placement's speech line.

**VSync.** The thinks read the count the frame began at (`Game::vsync_lag`, 2). The oracle's emulated timing reads one later in some runs; the tools copy that from an emitter's or spinner's +0x64 parity.

**Tools:**
- npc_check syncs all placements' trigger states, the spinners, the callbacks' tenth seed (0x800c41b8), the ambience count and the follow camera.
- It also has EMITDBG, SPINDBG, VISDBG, PORTALS, TRIGLIST and MARKDBG.
- `emitters_launch_as_the_original` now also covers level 3's and 16's spinners with level 16's sparkles (482 launches), and level 4's drops with their splashes. All 9 places are the original's.
- Level 5's robots are all the original's in npc_check (739 launches, 15 sounds). The Both harness lacks the characters' body sync.

**Still unknown:**
- The marks' face bits (0xc080be00 / 0xc0802e00 by the tile) and their exact drawing (0x8008e508).
- The original's world test runs per room, so a drop crossing rooms can leave two marks where the port leaves one.
- Which VSync count the thinks read on hardware.
- Level 20's, 26's and 28's cameras in npc_check (the tool's put and player state), level 3's hinged door closing early (sound 54), and the port-only landing sound after npc_check's put (the tool's player is not settled).
