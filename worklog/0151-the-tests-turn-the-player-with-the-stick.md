---
number: 151
title: The tests turn the player with the stick
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/golf.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T04:05:41-06:00
took: 8m
---

# 151. The tests turn the player with the stick

`face` set the heading. It now turns with the stick as a player does,
and three kinds of aiming each use their own input:

- Standing, free (`turn_to`): a way's heading is the camera's turned by a
  multiple of 512, and a turn over SNAP_TURN (1024) goes round in place
  to it while a smaller one snaps (0x8005be24). A tap of Down turns the
  body round in place; the follow camera swings after it about 12 a frame
  (levels 3, 6, 11), so its heading passes every value mod 512, and the
  way within 8 of the wanted heading is pressed then. Where the camera
  does not swing (a fixed camera, or the hub standing), the nearest way:
  up to 256 off, as a pad player gets there.
- Frozen (`steer`, the pie toss and level 2's throws): the stick only
  turns the body, 160 a frame, and letting go stops it; the way on past
  the wanted heading is held while more than 80 off. The pie toss and the
  goose loop now aim this way frame by frame, with Triangle on top.
- Golf (`golf_aim`): Left and Right step the aim 25 a frame, 8 with
  Square; with no common factor the presses reach the solver's heading
  exactly. The course still plays at par.

Port fix found by it: in golf's aim (state 14) Left took 185 a frame, the
frozen player's stick turn (160) on top of golf's 25. The original's
0x8005be24 has its own case for state 14 that only steps the target by
25 or 8 (docs/levels/golf.md, Aiming; seen 25 a frame). `Player::script_aims`,
set by golf while walking in and aiming, keeps the stick off the target.

**Still unknown:**
- Whether the original's follow camera swings after a turn in place in
  the hub as it does in the levels (the port's does not there).
