---
number: 138
title: Running bobs with its root keys
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:06:10-06:00
took: 3m
---

# 138. Running bobs with its root keys

The run plays table states 3 and 4 in turn (0x80056484), each with its
own root keys. The oracle's body, run from the hub's start with the stick
held, bobs:

746, 739, 756, 761, 769, 780, 751, 731, 720, 753, 773, 778, ...

and comes back to the floor (779-780) each round.

With worklog 137's `root_y` the port drifted up by 11 a round. Each time
the state changed, the last frame of the animation before was dropped.
`Player::root_step` now:

- keeps the base track as it last saw it (`RootSeen`: state, frames,
  frame, root y so far and whole);
- adds the rest of an animation that reached its last frame when the next
  begins, or when a loop starts round again.

The jump's crouch is the exception. Its last part is not taken, as the
body leaves the floor when state 6 comes; the oracle goes 868, 868,
then 814.

Port against oracle, over three rounds: 747/746, 739/739, 756/756,
761/761, 769/769, 779/780, 750/751, 730/731, 720/720, 753/753, 773/773,
778/778, ...

Test: running_bobs_as_the_original holds 18 frames within 2. The jump's
test (worklog 137) still holds.

**Still unknown:**

- Walking (states 1 and 2) and the other characters' bobs were not peeked.
