---
number: 68
title: Level 11: a standing jump never eats a bone
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/player.rs crates/rugrats/src/world/game/goodies.rs
resolves: 42.3
started: 2026-10-15T18:09:23-06:00
took: 2m
---

# 68. Level 11: a standing jump never eats a bone

0x80060f1c, the jump launch, switches on the state less 3:
- case 0, the standing jump, sets 0x800c4bc4 = 1;
- case 12 sets 2;
- the moving jumps (25, 35) set 0, or 64 when fast.

0x8005ed2c, a touch while in the air, skips a level 11 bone (kind 14, phase below 2) while 0x800c4bc4 & 1 is set. So Spike only eats a bone from a standing jump once he lands on the stand (the stand and walk handlers).

- **The port:** Player::standing_jump is set at the launch (the standing branch, speed at walking or less) and cleared by a moving jump. Game::touch skips bones in level 11 while the player is in the air after one.
- **Air control** (0x80061330, read for this) is not ported, and need not be:
  - The stick's axis (-127..128) is reduced by >> 7, or by * 340 >> 15 with bit 64.
  - It is then divided by about 27.5 (the 0x4a90be59 magic) before the camera turn, so with a pad it rounds to 0.
  - The lift (-5 while Cross is held on the way up) is kept only while 0x800c4288 lasts and not after case 12 (bit 2). The port lifts while Cross is held and the jump rises.

**Still unknown:**
- 0x800c4288's start value (the lift's time); the port's lift has no limit but the rise.
