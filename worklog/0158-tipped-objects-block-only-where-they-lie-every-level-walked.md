---
number: 158
title: Tipped objects block only where they lie; every level walked
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T09:53:41-06:00
took: 52m
---

# 158. Tipped objects block only where they lie; every level walked

The walking campaign (`levels_walked`) found a port bug: an object turned
other than about y blocked the body along an endless strip. `Box3::depth`
and `crosses` took a point into the box's own frame and tested its own x
and z only. For a box turned about y that is its outline from above; for
one tipped on its side (level 3's templates 526 and 650, things lying on
the floor) the axis that now lies across is the box's own y, which was
never tested, so the box covered every point along it. Tommy, in level 3
at (116857, 779, 3723), could not run west at all: box 650, at z -28159,
32000 away, sent each step back (traced at the step's resets). Level 6's
wheels (spinners turned about z, flat in their own plane) did the same
along x.

`Box3` now carries an `Outline`, made where its box is (`Outline::of`, in
`world_boxes`): for a box turned other than about y, the convex hull of its
eight corners seen from above; two points for one edge on. `depth` is then
how far inside the hull's edges (or the distance to the segment), and
`crosses` clips the step against the hull's edges (or meets the segment).
A box turned about y keeps its rectangle as before. Made once a frame, not
per test: computed per call it took the play tests from 65 s to 346 s.
`a_tipped_object_blocks_only_where_it_lies` holds Tommy's run and every
tipped box of level 3 covering its own place and not far off.

The walking campaign now walks every place the tests go, none put:

- Level 5: `ride_elevator` walks to the deck beside the elevator (level
  5's sliding doors, template 843, are passed by the plan: they open as she
  comes within 5000), onto its middle with the stick, rides it down
  (ridden, it turns at either end and goes up again), and steps off at the
  bottom toward the lower deck. `float_to` floats into room 38: a way over
  the open cubes (`afloat`), each cube's middle gone to with the stick
  across and Cross up, Circle down.
- The walk: a body down off the way (off a ramp's side) plans again at
  once; 120 frames on the ground with no place passed is stuck (back and
  forth by a wall had reset the count); up or down a slope the body is
  turned to the way's heading and the camera put behind (R1), then aimed
  at the place 600 on along the line (the stick's eight ways drifted off
  level 6's ramp, 400 wide); a place a step was held out of is banned from
  every side; with no way, off ahead, back, and to either side in turn; a
  plan's end within two cells goes on to the spot's own place before the
  last stride; the planner's room for the body is tested at its middle and
  half its width as well.
- Level 3: the boss may bite Tommy twice on the way to the switch: back to
  the aisles, and room 36's cine 86 again on the way in; the test plays it
  and goes on.

`levels_walked` (LV picks levels; WALKLOG, WALKTRACE, WALKPATH trace) now
ends "not walked (0)".

**Still unknown:**
- How the original tests an object's box turned other than about y
  (0x80065210, 0x8006a714): the outline from above is the port's guess.
- Whether the original's level 5 elevator also turns at the bottom while
  ridden, or stops.
