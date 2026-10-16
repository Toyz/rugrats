---
number: 164
title: Slides, one bump, air control and the analog stick
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/input.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T13:41:40-06:00
took: 2h 3m
---

# 164. Slides, one bump, air control and the analog stick

Running into things now slides, bumps and stands as the original does,
the stick pushes a jump along in the air, and the port reads a
DualShock's analog stick.

**Slides and the bump (0x8005d11c, 0x8005e1f4, 0x80065120).** The stand,
walk and run handlers share one rule for a step the probes find held up:
+0x130 is the angle between the step turned back and the face's normal
(0x800657d0 through 0x800761c8). Under 384 (220 as Spike, +0x15c 6) the
thing is met head-on: the climb test, else the run's bump (state 9); from
384 to a right angle the step goes along the face instead, at its whole
length (0x80065120 turns +0x178 by 0x800763d0's tangent, rcos 1024 being
0), if the probes then find that way open, or meet a face there at 512 or
more (held, no bump); else the head-on path. A wall's hit has no thing
(+0xf4 0): the bump without damage. The oracle, level 18, Tommy put at
(28664, 12748) facing 1433, R1, Up held: along x 30866 from z 11166 (door
213's box at a slant), along x 31335 from z 6039 (a cell's side), then
head-on into the corner's other wall at (31335, 5529): state 9 for 13
frames, then state 0 with Up still held, no bump again. The port matched
none of it (it bumped on the door every 14 frames, for ever); it now
follows within 8 throughout (`a_run_slides_along_walls_and_bumps_once_as_
the_original`). Standing, a held-up step leaves the body standing, and the
stand handler's climb test runs from a standstill (a box pushed at from
beside it is climbed).

**Air control (0x80061330).** The stick in the air, turned by the camera
into the body's own frame with its sideways part halved, pushes against a
drag of 32/4096 of the speed, both a unit of time, the step v dt + a dt^2
/ 2. The port had a drag of 32/4096 a frame and no push. The oracle,
level 6's shelf, put at (40334, -4341, -34944) facing 1024, R1, Up held,
Cross: steps across 111, 122, 132, 142, 152, 160, 168, ... 226 a frame,
where the port's fell 106 to 90; fitted, the push at full is 12.55 a unit
with the drag as coded (rms 1.1 a frame). The port is now within 4 a step
(`the_stick_pushes_a_jump_on_as_the_original`). Cross at the shelf's edge
jumps from its height (the oracle); the port's crouch had dropped the
body 3072 to the floor under it.

**The analog stick.** `Pad::stick` carries the left stick (DualShock in
analog mode), each axis taken over the D-pad's where larger, as
0x8005be24 does. The walking tests steer with it: an exact heading, a
light push to turn in place without a step.

The walking campaign (every level won by play) needed its own fixes for
the slower, truer movement: straight runs through pickups, three probes
in the plan with the middle one alone as a fallback, a frame-counted
stall, a repeated fall off the way banned, the jump shape with the push.
All 16 levels walk with no fallback; level 17's nickels take 3818 frames
of the clock.

**Still unknown:**
- The air push's scale: the code's default branch (D_800c4bc4 0, as the
  oracle reads it) gives 9.3 at full, the flight fits 12.55; the body's
  matrix at +0x24 may scale it.
- The body's height in the air: the oracle's +0x1c leaps about 1057 as a
  running jump leaves the floor, the port's root-y convention about 260;
  the drawn arc matches either way (worklog tests), collision may not.
- The COL's side normals are taken along the axes, from which axis alone
  holds the step; a cell's diagonal halves are not given their own.
- The analog axes' scale ((b - 128) x 32, full 4064 one way, 4096 the
  other) is not checked against 0x8001a888's type 2 conversion.
