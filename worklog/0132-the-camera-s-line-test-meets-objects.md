---
number: 132
title: The camera's line test meets objects
date: 2026-10-16
area: engine
files: crates/rugrats/src/world/game/camera.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/tests/camera.rs
started: 2026-10-16T01:28:02-06:00
took: 5m
---

# 132. The camera's line test meets objects

Worklog 131 took 0x80090554 for a test against the room's polygons. It
is not one.

0x80090554 walks the rooms along the segment, from the camera's room
through portals (0x8002ade0). In each room 0x80090340 tests the objects
in that room's two object lists (+0x48, +0x4c) with 0x8006a9cc, and keeps
the nearest hit. 0x8006a9cc skips:

- template flag 16;
- template flag 8;
- with the camera's 0x8000, characters (class 1 with a hierarchy).

It tests each remaining object with 0x8006a714. After that, the height
field (0x8002cd7c), and the nearer stop wins.

The port's camera now tests the frame's object boxes, `Game::solid`:
static objects, breakables, movers, spinners, walkers, doors; no flag 8,
no characters. `Box3::segment_hit` finds where a segment enters a box (a
3D slab test in the box's frame), with the normal of the face. The nearer
of that and `collide::segment` is the stop.

Checked against the original. Run 5's hub path (oracle frames 2782-2900,
/mnt/data/claude/scratch/cam/run5.tab) was replayed into the camera:

- with objects, every frame's eye and look-at match;
- with the height field alone, they part from 2836. At 2840 the eye is
  (10016, -1589, 18187) against the oracle's (9848, -1589, 18251).

That is the divergence worklog 131 put down to "polygons at 2836, 2856".

Test: hub_walk_on_past_an_object replays that stretch and checks 11
frames from 2838 to 2898. It fails at 2838 with the old test.

**Still unknown:**

- The object test 0x8006a714 is against the model, not its boxes.
- The rooms walked: the port tests every object box, not only those in
  the rooms the segment passes.
- Whether things at rest (pickables) are in the room lists the camera
  tests. The port leaves them out.
- The hub start's doorway hit in room 35 (worklog 131) has not been
  rechecked.
