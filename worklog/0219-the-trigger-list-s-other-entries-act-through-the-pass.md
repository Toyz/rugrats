---
number: 219
title: The trigger list's other entries act through the pass: speech lines, level 5's doors, level 12's batteries, the signs and sparklers, Cynthia; the cursor starts each level at the head
date: 2026-10-18
area: engine
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/rooms.rs, crates/rugrats/src/levels/outer_space.rs, crates/rugrats/src/levels/let_there_be_light.rs, crates/rugrats/src/levels/cynthia.rs, crates/rugrats/src/levels/hub.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T05:21:17-06:00
took: 42m
---

# 219. The trigger list's other entries act through the pass: speech lines, level 5's doors, level 12's batteries, the signs and sparklers, Cynthia; the cursor starts each level at the head

The trigger list's other entries now act through the pass, as the original's enters and leaves do:
- the speech lines;
- level 5's sliding doors;
- level 12's batteries;
- the hub's door signs and level 16's sparklers;
- level 4's Cynthia.

The cursor at a level's start is settled.

**The cursor at a level's start (0x80072d38(1)).** The pass's other form walks the whole list from the front, entering every entry whose squared distance is under +0xa0. It sums three 32-bit products and compares them signed, so wrapped sums count too. It runs in three places, each before a loader fills the list, so it always walks an empty list:
- the level runner, before 0x800454dc (0x80017d98);
- main's own cinematics (0x80074858);
- the activity reset (0x80096b54).

It leaves the cursor null, and each level's first five-a-frame pass starts at the list's head, the port's index 0.

**The enters and leaves.** The pass calls an object's +0x48 or +0x4c and leaves whether it is in (+0x00 bit 0) to the call. `Game::trigger_pass` hands the port's to `Game::trigger_called`:
- **Speech lines** (0x8004d0f0): with the player in the object's room it says its line and is in; otherwise it stays out, to be entered again on a later round. 0x8004d160 lets it out. The talkers' own 2D distance each frame is gone. A talker with template +0x34 of 0 is now on at the list's 15000 rather than left out.
- **Level 5's sliding doors** (0x8004d078 opens with sound 71, 0x8004d0b0 shuts): `Script::trigger`. A beam's open (0x80051d44) sets the door in, so the pass shuts it when it next finds the player out of reach. The port's own reach test (5000 across, 4000 high, every frame) is gone.
- **Level 12's batteries** (0x8004cf58): taken (+0x58, +30, sound 109) and off the list (0x8001a810, `Game::trigger_remove`). The 3D distance each frame is gone.
- **The hub's door signs and level 16's sparklers** (0x80049eec, their think while in): shown while their entry is in, in the player's room. The hub's mirror, not on the list, keeps its distance.
- **Level 4's Cynthia** (0x8004d138, a use, 0x8004f78c): found when her entry is entered, tested at her hiding spot (`Script::trigger_at`, the object's +0x18 where the level moved it). A thing is tested where it now is.

**Oracle.** `level_5_door_opens_as_the_original`: Angelica put 4000 short of a door with floor before it, the cursor synced. The port's pass enters the door on frame 2. The original sets its +0x00 bit 0 in the same game frame, read on the oracle's frame 3: the oracle's frame ends between the objects' update and the pass (0x80072bf0 | 0x80072d38, traced every frame).

**Still unknown:**
- Levels 20 and 21's Angelica compared through her modes 4, 8 and the lunge: the sweep tool syncs only the post and follow modes.
- Level 5's robot 17 after the original's lag frame (dt 15): the tool pairs frames a frame apart from there.
- The crouch over an edge.
