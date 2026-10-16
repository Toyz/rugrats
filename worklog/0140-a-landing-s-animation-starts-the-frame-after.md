---
number: 140
title: A landing's animation starts the frame after
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:15:47-06:00
took: 3m
---

# 140. A landing's animation starts the frame after

0x8005a068 steps the body's animation (0x80056834) before the state's
handler runs. The handler is what notices the floor and sets state 10, so
the land animation's first step comes the frame after touchdown. The
oracle's standing jump spends seven frames in state 8 (790, 886, 912,
923, 923, 908, 894). The port spent six, because it set and stepped the
land animation in the frame it landed.

The port now sets the animation that frame but does not step it. The
landing then runs seven frames: 779, 842, 886, 912, 923, 922, 908. The
oracle's lag frame just before touchdown (625 twice) puts the two a
frame apart in phase.

The egg hunt's play test puts Tommy down 100 over each egg. With the
landing played out in full and held still (worklog 139) he needs longer
to reach the egg, so `collect` holds the stick 14 frames instead of 6.

**Still unknown:**

- Whether other states the handler enters (climbs, throws) also start a
  frame late; only the landing was looked at.
