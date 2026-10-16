---
number: 135
title: A climb carries the body on to the ledge
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/animator.rs
started: 2026-10-16T01:49:32-06:00
took: 6m
---

# 135. A climb carries the body on to the ledge

Worklog 134 drew a climb as the body's position less the root motion
still to come. Two faults showed in level 6's box setup:

- The port's position at a climb is already one step into the ledge, so
  the drawn body jumped back about 156 as the climb began.
- The physics left the body at that step, while the original's root
  motion carries it on to the ledge.

Now the climb keeps where it began (`Climb::from`, the position before
the step) and its root motion so far in the world (`Climb::moved`,
updated after each animation step). The body is drawn at `from` plus that
motion, and down by the part of the rise still to come.

When the climb ends, the body is put where the motion got to. That is
what 0x80066e60 does with the +0xd8 / +0xdc the animation filled. The
port does it only when the spot is on the ledge (the COL floor or a box
top within a step of the top) and has room for the body (no box over the
top within its height). Without the room test, level 6's stacked boxes
put Tommy inside the boxes above the one he climbed.

The root motion of a body other than character 1 (Tommy, 0x800c5680) is
scaled by its root offset (hierarchy +0x4c) over 0x800c41dc. Both were
peeked: 1 and -561, in levels 1, 4 and 6. `Animator::character` now keeps
the character the tables belong to.

Checked in level 6's setup (Chuckie, scaled 0.63):

- the drawn root goes z 13410, 13433, ... 13653 and y -2402 to -2860;
- the body is then at 13653 and runs on;
- y settles 54 onto the top as the last frame is cut.

All 41 play tests pass, among them level_6_toy_palace_is_won and
level_6_box_stack_reaches_the_platform.

**Still unknown:**

- Whether the original also cuts the climb's last frame (the 54 drop), and
  how its own floor test meets it.
- A climb compared with the original's picture.
