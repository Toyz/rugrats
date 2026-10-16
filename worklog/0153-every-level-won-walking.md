---
number: 153
title: Every level won walking
date: 2026-10-16
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-16T08:20:50-06:00
took: 5m
---

# 153. Every level won walking

`levels_walked` now wins every level with the player walking (stick,
Square, R1, Cross) to each place the level tests used to put it, all
sixteen in one run (levels 2-6, 9-13, 15-18, 20, 21, about 30 s).

On the way:

- Pickups (`collect`): the nearest left first, walked to and run at; one
  overhead is jumped for (Cross held under it). Level 18's 35 eggs then
  fit its 170 s clock.
- Level 6's boxes: off a box the walk climbed onto before taking it up.
- Doors: only a shut one at rest is planned through; an open or swinging
  one stands where it is, and a door is used only when it holds the walk
  up (the walk opened level 18's refrigerator in passing and was caught by
  its swing).

Twelve walks still end in the old put (the room entries count twice):
level 5's elevator ride (room 30) and float (room 38), which the walk does
not model; level 2's maze to room 69 and the court (room 44); one box each
in level 6, one place in levels 9 and 17.

**Still unknown:**
- Why level 2's maze and court are not walked although the flood reaches
  them (geese, the gate's swing, or the yard's slopes).
- Rides and the float in the walk.
