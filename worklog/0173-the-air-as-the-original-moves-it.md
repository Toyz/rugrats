---
number: 173
title: The air as the original moves it
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/ground.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/examples/air_check.rs, crates/rugrats/examples/ground_check.rs, crates/rugrats/examples/below_check.rs, crates/rugrats/tests/hle.rs
started: 2026-10-16T21:51:45-06:00
took: 39m
---

# 173. The air as the original moves it

The air is now the original's: its test of the move (0x80066014) and the
move itself (0x8005f1b4 with 0x8005ee78), proved against the original
called on the live player, and in play. All four scripts of
`tests/hle.rs` now match the original across (x and z) on every frame,
and the run and jump from the hub's start through its whole flight into
toy 403, the bounce off it and the landing (39 frames; it differed by up
to 241 from frame 28 before).

**The air test** (`ground::Ground::air`, `examples/air_check`: 2,800
moves in eight levels, 0 different):

- At or past the height field's floor under the body (+0x13c, which the
  floor test 0x800627a0 queries each frame: the hub's walked floor is a
  thing at 779 over the field's 1024): landed there.
- Across into the field's floor: a wall facing back (its normal the move
  turned back, made a unit).
- Else air probes the body's depth past the move along it, its middle
  and its sides (straight up or down, two only), each five segments (up
  the body's axis to its top, across at the top, down to its bottom,
  across at the bottom, down at the probe; met on that last, across
  again just under it): the objects' first (0x80065210, 0x8006592c),
  then the field's (0x80065578, 0x80065b3c). A face's kind
  (0x80067168): a wall, a slope overhead, or a floor to stand on.
- The bottom is the lower of the body's box and its lowest posed joint
  (+0x68, kept by UpdatePartRotations, 0x8004a5dc).
- A floor to stand on met moving down lands (0x80065cbc): the floor
  moved to the height there, the thing stood on taken.
- The middle's, right's and left's kinds pick the record kept and the
  answer (27 cases): clear, a wall (the way turned off it and its slant
  across, 0x800761c8), a slope overhead to slide along, or held with no
  way along.
- A segment of the field that does not stop leaves the record as it
  was (the probe from inside the floor after a floor met).

**The air move** (`Player::air_move`): the velocity's step, held at
+0x13c; across first, tested without the fall and had as left, then
the fall, tested and had. A wall met going up within 512 of where the
jump began (+0x670) stops the way across; met head-on (or past a right
angle) the body loses its rise and, as at a slant, its way across, and
the whole move is had once more that frame; the fall of the first is
had as well, so a bounce falls twice (toy 403: 377 to 589). Landed: the
state 10. A slope overhead turns the velocity along it. Once the flight
has met something (D_800c4bc4 flag 2) the stick no longer pushes and a
held jump no longer lifts.

**The take-off's height**: a moving jump leaves from the run's height
that frame; its own state's first keys are not added (the port flew 31
higher than the original, which decides whether the probes reach a
thing's top).

**Still unknown:**
- On the ground the original's y carries the animations' root motion
  (the run's bob, the landing's squash: 589, 778, 886 .. 923 .. 822); the
  port keeps it apart (`root_y`) and snaps to the floor.
- +0x68 comes from the port's posed parts in floats, not the original's
  integer posing (0x8004a5dc); exact only where the body's box is lower.
- The objects' callbacks met in the air (0x8005ed2c: type 711, class 11
  but 1177, characters of class 16) and flag 0x4000 things that cannot
  be landed on; flag 8 of D_800c4bc4.
- The ground floor's following with objects (0x800627a0) and the eased
  step, still the port's own.
