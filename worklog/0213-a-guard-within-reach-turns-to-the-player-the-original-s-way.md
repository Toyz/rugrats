---
number: 213
title: A guard within reach turns to the player the original's way round, with the turn in place; the trigger pass's five a frame found
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, docs/engine/gameplay.md, docs/levels/level-05.md
started: 2026-10-17T22:02:11-06:00
took: 37m
---

# 213. A guard within reach turns to the player the original's way round, with the turn in place; the trigger pass's five a frame found

A guard within reach turns to the player as the original's does, the original's way round and with the turn in place. What still holds the original's robot back is the trigger pass, five a frame.

**Turning within reach.** A guarding robot 1200 from Angelica, in the oracle:
- The original's turns 672 a frame while it stands within reach (+0x184 moved into +0x176, so speed 0). That is the stick's 512 and the body's 160, the turn every character's steer takes.
- The port's guard stood still within reach. It now steers to the player at speed 0 (`Npc::steer` with the body).

**The rest of the original's attack, traced** (robot 2000, +0x688 0x200a, mode 11):
1. Frames 1-4 after Angelica is put beside it: no update at all (its height holds at -9068, its heading at 0). Guess: it is scheduled only once the trigger pass meets her (its enter callback 0x80049e18, +0x48).
2. Frames 5-8: it turns 0, 672, 1344, 2016, 2048, its base track state 63 (logical 53, the turn in place to the left), its hover bringing it down (-9032 to -8339).
3. Frame 9: the attack (mode 5): level with her box (0x80083ff0). +0x688 bit 512 sets +0x24 to +0x2c (the snap to 2048) and asks animation 13 first.
4. Frame 10: the bite (state 49), mode 1. Frame 11: the damage taken.
5. Frames 10-29: mode 1; 30: mode 0; 31: mode 11; 32: the next attack. One every 23 frames.

**The turn's side and its animation (0x800820e0).**
- The difference to the wanted heading loses a turn from 2049 and gains one under -2048, so a way straight behind (2048) turns to the positive side. The port's `wrap` made it -2048 and turned the other way. `Npc::steer` now normalizes as the original does.
- Standing (+0x18c 0) with a turn to make, 0x800820e0 sets +0x688 bit 0x10000. That is what makes 0x80059204 answer -1, and so the stand (0x8005d11c) asks logical 53, the turn in place (state 63). A guard turning within reach now asks it.

With both, the port's robot turns 672, 1344, 2016 as the original's does, its base track state 63 through the turn, but from frame 1 where the original's waits to frame 5. It then sits at 2053 (no snap) and bites on frame 7, then on its cooldown (23 frames).

**Why the original's waits: the trigger pass.** 0x80072d38(0) tests five entries of the trigger list a frame, going on from its cursor (0x800c4c20).
- The loader (0x800454dc) pushes each placed object whose init set an enter callback (+0x48) on the front of the list, in LP order.
- Level 5's list holds 51 entries (sound objects 1139, movers 1156, the sliding doors 843, the robots, others), with the cursor at the 25th.
- Robot 13002 is the 43rd, so it is scheduled on the fourth frame, which is the original's wait.
- The port tests every character every frame. The gameplay doc now has the list; which inits set +0x48 is not mapped.

**Still unknown:**
- The trigger list in the port: which inits set +0x48 (0x80049e18, 0x8004d0f0, 0x8004d138, 0x8004cfa0, 0x8004cfd8, 0x8004f228, 0x8004d174, 0x80083f24 and the level scripts'), and the cursor at a level's start.
- The attack's snap (+0x688 bit 512) and animation 13, and the bite cycle as modes 5, 1 and 0 for the guards (worklog 212).
- Other characters standing and turning (the stand's turn in place for all, not only guards).
