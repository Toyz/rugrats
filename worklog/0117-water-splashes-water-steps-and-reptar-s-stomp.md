---
number: 117
title: Water splashes, water steps and Reptar's stomp
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T00:01:33-06:00
took: 13m
---

# 117. Water splashes, water steps and Reptar's stomp

Walking or running over a water floor (COL tile byte 4 bit 0) now throws a
splash each frame, as 0x80050770 does: a kind 5 flight, flags 0x10705,
sprite 275, half-size 100, speed rand % 150 capped at 100, within 2048 of
straight up, spin 3000, bounce 1000, 1000 ms. Standing on a mover or on an
object above the floor is not water.

The walk's step event (frame event bit 256) picks the step sound:

- Reptar (character 34): 108 on every step.
- Spike in water: 57 every fifth step in level 4, 176 in level 16.
- Anyone in water outside the hub (and Spike in the hub's water too): 27.
- Otherwise none. The general footstep table 0x800c4ec4 is all zero while
  playing (checked on the original), so a dry step makes no sound.

In level 7 (Reptar 2010) Reptar's step puts down a puff (0x80050c48): kind 5, flags
0x30705, sprite 397, half-size 64, speed 100, between parts 3 and 7 at the
height of part 7.

Test: walking_in_water_splashes puts Spike on the water floor nearest
level 4's start and runs him for 20 frames; splashes appear.

**Still unknown:**

- Level 4's landing sound 57 on water (the jump's end) is not played yet.
- Floor flag 16 dust (0x80050b5c) in other levels is not ported.
- Splash flights do not collide with objects (kinds 1, 2, 4 still open).
