---
number: 137
title: Jumps crouch, land and settle as the original's
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:00:25-06:00
took: 6m
---

# 137. Jumps crouch, land and settle as the original's

On the oracle, a standing jump from the hub's start (Cross at VSync 2702,
the body's y peeked each game frame, with 0x800e2bf0's table state) goes:

1. **Crouch** (table state 5): 783, 794, 810, 827, 843, 859, 868, 868. The
   jump animation's root keys take the body 89 under the floor. The body
   does not leave the floor yet.
2. **Off at state 6.** 0x800562bc sets +0xe0 bit 31 on state 6's entry,
   and the jump's handler 0x8005ec9c turns that into flag 8 (in the air).
   From 868: 814, 668, 544, 442, 362, 304, 268, 253 (the top), 260, 289 and
   down.
3. **Landing** (state 8): 790, 886, 912, 923, 923, 908, 894. The land
   animation's keys sink the body 144 under the floor.
4. **Stand** (state 0): 868, 845, 822, 799, 779. The floor takes 23 a
   frame back.

The port left the floor on the press and drew neither sink.

Now:

- **The press** keeps the launch in `pending_jump` while the crouch plays.
  When the base track leaves table state 5, the crouch's depth goes into
  the position and the launch starts, with its first air step taken in
  that frame. The original's animation steps before the state's handler.
- **The base track's root y** goes, each step, into `root_y` on the ground
  (crouch, landing, standing, walking). The body is drawn under its
  position by it. Standing, walking or running, 23 a frame of any depth
  under the floor is taken back.

The port's frames against the oracle's, all within 1:

- crouch to the top: 783 ... 868, 868, 813, 667, 543, 441, 361, 303, 267,
  252, 259, 288, 339, 412, 507, 624;
- the landing: 886, 912, 923, 922, 908.

The oracle shows 625 twice just before the landing, a lag frame, where
the port lands a frame later.

Test: a_jump_moves_as_the_original holds the 21 frames from the press to
the fall within 2. All play tests pass with the later take-off. Among
them are level 11's standing-jump bone rule and the jumps in levels 7, 10
and 17.

**Still unknown:**

- The moving jump (logical 28) leaves at once in the port, as its states
  were not traced. Its table state's entry (41) also sets bit 31.
- The original also adds the root y in the air. The port leaves it out:
  added, the port's arc went far higher than the oracle's.
