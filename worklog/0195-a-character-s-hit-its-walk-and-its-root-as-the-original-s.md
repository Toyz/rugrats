---
number: 195
title: A character's hit, its walk and its root as the original's; damage to characters posted
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/levels/mr_friend.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T08:11:33-06:00
took: 31m
---

# 195. A character's hit, its walk and its root as the original's; damage to characters posted

The rest of a character's hit and the rest of its body follow the original. Mr. Friend now takes a ball, staggers, loses his health and chases on, and the ball rolls, on the original's place every frame.

**Mode 1's wait.** At its animation's event 0x2000 (and not for 1776), mode 1 waits 300 + (rand & 3) x 300 dt (+0x69c, by 0x8002fb7c) with no body update at all. This is in `npc::step` before the step, as `Npc::wait_left`.

**Mr. Friend's hits.** His callbacks (0x800833a4 for a kick, 0x800834b0 for a thing's touch) give his line, ask the stagger (logical 8 at 2048, table state 30), and put him in mode 1 with the callbacks off until it ends (`Npc::staggered`). This is now the follower's own cycle (`Bite::Recover`). The port used to keep its own `Mode::Hit` timer.

**Damage to a character.** 0x800780ac with a target of object flag 0x4000 posts the target. The event (0x80077864) takes its health straight down at the next frame's start; with no such target it takes the player's. `Game::post_npc_damage` queues it. A thrown thing therefore staggers Mr. Friend at once and costs him its +0x6d a frame later. When mode 1 finds his health at 0, he dies (mode 10).

**Walking and running.** A character (object flag 0x4000) walks (logical 1) under the fifth speed of its table row (0x80059008, 0x800590d4: 17, or 11 in the slow row) and runs from it. The port ran it at any speed.

**The root, across and up.**
- Another body than the player's moves by its root keys scaled by (hierarchy +0x4c << 12) / -561 (0x80055904), each toward zero, before the step's share is taken.
- A state whose word has 32 also moves it across. Mr. Friend's stagger steps him back 22, 13 a frame.
- The body takes that move after its stick, so the steer reads the place before it.
- The height (`RootCurve`) is now in these integers, with the player's first-frame blend, base + (key - base) frac. It was a float scale and blend, a unit off at times.

**The aim at a character** takes its height as its +0x1c has it, with its root. That was the last unit in the ball's launch.

**Oracle.** `level_9_ball_at_friend_as_the_original`: Tommy picks up ball 2 and throws it at Mr. Friend, who comes for him. Given the original's Friend once (`sync_friend`: place, animation, the logical animation it was asked as, health, mode), it checks every frame:
- the ball's places for all 77 frames of the roll;
- Friend's tracks, place, height, heading and health;
- Tommy's place and tracks;
- the camera.

All 15 scripts pass.

**Still unknown:**
- Which animations carry event 0x2000 besides the stagger.
- Level 14's Mr. Friend (every hit staggers, no aim): kept as the port had it.
- The other levels' things that strike characters still take health at once (their hooks: 0x8007eb68 level 2, 0x8007e308 level 3, 0x8007eafc level 16).
- The player's damage while +0xf8 is set (0x80077864 skips it): what +0xf8 is.
