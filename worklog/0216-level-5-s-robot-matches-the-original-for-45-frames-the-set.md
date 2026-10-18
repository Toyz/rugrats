---
number: 216
title: Level 5's robot matches the original for 45 frames: the set up's lift, a root on its keys, the guard's goal out of range, guards bite through modes 5, 1 and 0
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md, docs/levels/level-05.md
started: 2026-10-18T00:20:53-06:00
took: 1h 34m
---

# 216. Level 5's robot matches the original for 45 frames: the set up's lift, a root on its keys, the guard's goal out of range, guards bite through modes 5, 1 and 0

Level 5's robot matches the original for 45 frames from Angelica's arrival: its wake, its turn, its hover's height, both bites and the player's health. Four fixes got it there: the set up's lift, a root that moves on its keys, the guard's goal out of range, and the bite cycle for guards.

**The set up's lift (0x80055170).** The original sets every character's idle up at load (0x80082478, 0x80055f7c), with no ask. The tracks sit at the second frame, and 0x80055170 applies the first frame's keys. Each column's lift key adds its dy, as it is, to the height. A settled body is then put back on its floor (0x800452f8). An unsettled one keeps it: level 5's robots take four columns of -311, from the LP's -7824 to -9068.
- `Animator::init` now owes that lift (`take_lift`).
- `Npc::set_up_idle` sets the idle up as the original does, adding the lift to an unsettled body's place. The port used to ask the idle (phase 0), so each character's first step began it anew.

**The root moves on its keys (0x80055904).** Each step adds the frame's key times the change in its time gone. Only a state begun on its first frame with its whole time takes the gap (+0xde): floor (+0x148), feet and key, less the height.
- `RootCurve` now goes on from where the body is for a state set up and not begun (`RootCurve::set_up`).
- An unsettled body's keys start at the floor under it (`off`), so the robot's hover comes down to -8357 as the original's does. The port had held it 621 higher, at its own place plus the key.

**The guard's goal (0x80085220).** Out of range of home or 2800 high, a guard with +0x688 bit 2 goes home only when +0x690 or more from it. Short of that, it aims at the player with no distance, so it stops and turns to face them. The port went home past 600 or stood. The height test reads +0x1c, place and lift, before the body's step, as the AI runs before 0x8005a068; the port read it after, which put the bite a frame early.

**Guards bite through modes 5, 1 and 0** as followers do (0x800846e4: animation 37 and the damage, mode 1 until the animation is done, mode 0, back to mode 11). The guard's cooldown and `bite_wait`, the port's stand-in for that cycle, are gone.

**The reach (+0x690)** is 1024 from the init (0x80081bac), where the port had 600. Level 2's geese have 2000 (0x8007914c), which the port had missed; they now walk home past it.

**VectorNormal of nothing.** A zero vector's shift is -1, which `srav` takes as 31. The port's debug build panicked on it (`Trig::normal`). The nearest usable object's squared distance is now taken in i64; a far object overflowed i32 in the play suite.

**Oracle.** `level_5_robot_wakes_as_the_original` now asserts 45 frames of the robot's heading, its height (+0x1c), its four tracks with their phases and the player's health. All match: the wake on frame 5, the turn, the attack on 9, the bite (state 49) on 10, the damage on 11, mode 0 on 30, mode 11 on 31, and the next bite's damage on 34.

`Play::boot_with` hands the machine to a setup before the boot, for hooks that watch the load.

**Still unknown:**
- Which guards have +0x688 bit 512 instead of 8: the snap to the wanted heading, then animation 13 at 2048 and mode 1 with no mode 5. The port gives every guard mode 5; the robots' 0x200a and the geese's 10 have bit 8.
- What 0x80055170's channel test skips: a lift key whose channel the hierarchy does not map. The port takes every lift key.
- A follower's chase reads its height after its root's step (the guard's now before).
