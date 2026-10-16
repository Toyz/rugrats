---
number: 150
title: Level 6's boxes are carried up the steps by play
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T03:06:43-06:00
took: 59m
---

# 150. Level 6's boxes are carried up the steps by play

Level 6's win put four Reptar boxes in a column by hand. It is now played:
`box_route` fetches five boxes, carries each up the steps laid so far with
the stick, and puts it down (Circle) at the foot of the next step: the -2048
step from the floor, the -3072 block, the -3584 block, the -4608 ledge, and
on that ledge against the -6144 block. A box is 506 high and a climb while
carrying is at most 521, so each step of 1024 needs a box at its foot to be
taken in two climbs with the next box in the arms; the last box lets Tommy
onto the -6144 block (1030 from its top), from which he drops onto the
switch platform (-4736). Throwing a box up a 1024 step does not work: it
peaks 133 short and bounces off the face. The boxes used (LP 197, 198, 97,
94, 151) are the five reached from the start on foot; the other three sit
on shelves 1536 above their floor (a box under them is the way up).

A port fix found on the way: the carry's climb limit applied to whatever
was carried. 0x80058da4 limits it only for an object without bit
0x1000000 (class 13, `Thing::keep`: level 5's remote). `Player::carry_climbs`
is set at the grab.

The reachability check (worklog 149) grew to judge this:

- Each put is judged by the floor the body comes down on there, not any
  floor under it (a shelf over a reached floor no longer passes).
- Carrying (not a kept thing), no jump and climbs under 522; the body's own
  `climb_max`.
- Running jumps: 8 directions from cells at an edge, with a run-up of two
  cells, along `jump_shape`, the port's air (crouch frame, `moving_jump`'s
  launch, gravity 14 less 5 rising with Cross held, drag 32/4096). Tommy's
  matches the port's run to the crouch's root keys (54 higher at the first
  air frame): 849 up at 1288 along, level again at 2142. A landing needs the
  feet over the floor at that frame (in a wall the port puts the body back
  over the floor it came from).
- Objects within half a cell of a cell's middle count as its floor: level
  11's stands are narrower than a cell.

The check then caught two test placements: `win_4` put Spike at Cynthia's
y, under the floor for Spike's body (now on the floor there), and the old
level 6 column.

**Still unknown:**
- Whether the original's put-down stops a box against another box (in the
  port a box put down into another passes into it), so whether a column
  of boxes can be built.
- Whether this is the route the original intends; the original's full box
  route was never played on the oracle.
- The running jump in the flood is Tommy's run speed (17) for every body.
