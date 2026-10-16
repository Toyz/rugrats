---
number: 93
title: Idle fidgets
date: 2026-10-15
area: engine
files: crates/rugrats/src/actors/animator.rs crates/rugrats/tests/play.rs
started: 2026-10-15T22:21:50-06:00
took: 2m
---

# 93. Idle fidgets

Standing still now fidgets as the original does (0x800563bc, run each
frame from 0x80056834 for the base track):

- In table state 0, +0xf2 counts down by dt from 1200 (0x80057244 sets
  it with +0xf0 = 23). At 0 the state becomes +0xf0, and +0xf0 swaps
  between 23 and 24.
- In any other state, +0xf2 is set back to 1200.
- States 23 and 24 complete (end word 1) back to standing.

So a body that stands still for 120 frames plays one fidget, stands for
another 120, then plays the other. The oracle showed both (hub frame
2550: 24; level 3 frame 2880: 23). It applies to the player and NPCs
alike, since it lives in `Animator::step`.

The port skips a fidget the body's table has no frames for. The original
would play an empty animation; none of the bodies tried lacks them.
The level 3 sequence test now checks 23, 0, 24.

**Still unknown:**
- what the flags 0x20000000 (states 0, 23, 24) and 0x80000000 (6, 41) on +0xe0 are read for
