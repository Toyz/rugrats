---
number: 170
title: The ground probes as the original runs them
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/ground.rs, crates/rugrats/src/world/boxhit.rs, crates/rugrats/src/gte.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/examples/ground_check.rs, crates/rugrats/examples/ground_at.rs, crates/rugrats/examples/boxhit_check.rs, crates/rugrats/examples/hier_boxes.rs, crates/rugrats/tests/hle.rs, crates/rugrats-hle/src/play.rs
started: 2026-10-16T19:17:22-06:00
took: 55m
---

# 170. The ground probes as the original runs them

The ground move's probes against the height field are now the
original's, in its integers (`actors::ground`), proved against the
original called on the live player (`examples/ground_check`: 2,700 of
2,700 spots, levels 1 and 18, the return, floor delta, wall normal,
tangent and slant all the same):

- **Three probes** ahead of the body (0x80064a34): the middle at the
  body's depth (the larger of its box's z extents) past the step, and one
  to each side at its half width (the larger x extent). The offsets are
  the way's own frame: ratan2 of the way, its heading matrix
  (0x80020d10, then MatrixNormal) and ApplyMatrixLV - not a float
  rotation: the slant and tangent were off by 1 to 3 before.
- **The floor under each** (0x80067448), asked from the body's top: no
  record is no floor (0xffff0000); above the tile's ceiling, the floor on
  top of it. **The wall on the way** (0x80064178): up across at half the
  rise or just over the higher floor, down from 1024 ahead to 1024
  behind. **Its kind** (0x80064444): level, a rise, a step to climb, a
  small drop, a step down, a drop past the limit, or held (a normal too
  steep, a step past the limit, no floor).
- **The decision** on the three kinds: had, slid along the held probe's
  wall, or stopped (a stop leaves no floor to climb to). The slide's
  tangent (0x800763d0) is ((n.a) n - a) of the way turned back, made a
  unit, the slant the arc cosine of the dot (the libgte acos table); the
  slide (0x80065120) sets the way along it and probes again, kept when
  clear or met again at 512 to 1024.
- **The run's handler** takes a slant under 384 (220 for Spike) as
  head-on: the climb test from the held probe's floor delta, measured
  from the floor the probes used (a thing stood on: level 6's last box
  climbed onto the -6144 block from its top, not from the floor under
  it), climbable within the body's reach, too tall from 1024 to 3071
  (message 16), the bump otherwise.
- **The step limit** is the character's (0x800c430c by size class
  0x80062eac): 750 for Spike and Reptar (characters 6 and 21), else 256;
  Spike's body box is 1400 deep (0x800450ac).

libgte's VectorNormal (its rsqrt table), acos and asin (their table),
ratan2, MatrixNormal and ApplyMatrixLV are in `gte::Trig`, exact.

The objects' side has begun the same way (`world::boxhit`, proved by
`examples/boxhit_check` on 100,000 random cases each, in a fresh oracle):

- **The slab test** (0x80068e68): a segment against a box from 0 to its
  size in the box's frame, by the start's outcode: inside (the start), one
  axis out (that face), two out (the face the segment reaches first by a
  cross product's sign), all three out (the z face, then x, then y; a
  flat box's far face reads as its near one). Each plane crossing
  (0x8006876c, 0x80068914, 0x80068ac8) works in 20-bit differences with
  truncating division, kept within 0..4096 of the segment and the box's
  other extents. A miss on one or two axes out still writes the face's
  normal, which a later box's start-inside hit then carries.
- **An object's boxes** (0x80069c40): the segment less the object's
  position, turned by its angles' matrix (0x80020ee0, MatrixNormal),
  less each model box's low corner; the first box met, its point and
  normal turned back. A character's body (0x8006a008) is the same with
  its hierarchy's one box.

Both are not yet wired into the ground move: the player still meets
things through the float boxes of before.

**Still unknown:**
- The object probe (0x80063d3c): the nearby list's order (0x80062ed4),
  each box tested at its own middle height clamped to the body, the
  height field's wall nearer cancelling it (0x80063c3c), the thing stood
  on skipped; the object branches of the floor and wall probes
  (0x80067448, 0x80064178 through 0x80067284) and the run's pre-probe
  (0x800645d8). Until then level 18's slide meets door 213 and level 1's
  jump from the start meets toy 403 unlike the original.
- The hierarchy objects' parts (0x8006a318: one matrix a part from
  +0x158, or built by 0x8004a5dc) and the bounding cube that screens
  every object first (0x800698fc: half side the largest of the bound's
  extents at +0x78).
- The port's things carry float rotations; the original's angles and
  integer positions are needed for the exact tests.
