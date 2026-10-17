---
number: 175
title: Jumps as the original's, from press to landing
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/app/flow.rs, crates/rugrats/src/app/mod.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T23:09:24-06:00
took: 30m
---

# 175. Jumps as the original's, from press to landing

Jumps are now the original's from the press to the landing, in all
three coordinates while in the air; seven scripts are played against the
original, all the same across on every frame.

- **The held jump's lift has a clock** (D_800c4288): 1024 at the launch,
  a timer step (0x800c3f20, 136 at 10 a frame) less each frame Cross is
  held on the way up; any other frame ends it. The port lifted for as
  long as the body rose. The frame the clock runs out is split
  (0x80061330 has itself again): the lift for the rest of the clock's
  time (72 * 300 / 4096 = 5) and the remainder without it (64 * 300 /
  4096 = 4), the moves added. The hub's held jump: 7 lifted frames,
  then -370 to -269, as the original.
- **The time in the air for the long fall** (D_800c428c) counts every
  air frame once the lift is spent, from the launch (or the walk off a
  ledge), not only the way down (a guess, gone). A long fall (193) does
  not hurt in the hub, level 2 or the circus, nor as Spike (the
  player's type 9, character 6).
- **The standing jump leaves a frame after its crouch**: the state after
  the crouch (6) comes with the launch, the first step in the air the
  frame after, as the moving jump's (worklog 173). The original's height
  takes the crouch's root a frame after the animation (+0xda): 868, 868,
  814, then 664.
- **The animation's root is rounded toward zero** per part (0x80055904),
  as each frame's delta is: the crouch's last part is -54, not -55.
- **The take-off's own air step lifts** if Cross is held.
- Level 11's ring 2 is now jumped with Cross 7450 into the ramp (7300 to
  7600 make it; 6700 did with the unclocked lift).
- The fallback screen when a screen's files fail to load is `Blank`, not
  the "front end not ported" it was named for: the front end is.

`tests/hle.rs` adds the held jump; all seven scripts (the hub's run, run
and jump into toy 403, standing jump, held jump, run at the seat;
level 18's slide; level 11's Spike) match the original in x and z on
every frame, and in y through every jump's flight.

**Still unknown:**
- On the ground the original's height carries the root's motion a frame
  late (the crouch, the landing's sink to 923, the run's bob); the port
  keeps it apart and draws it at once.
- The follow camera after a bump from rest (worklog 174).
