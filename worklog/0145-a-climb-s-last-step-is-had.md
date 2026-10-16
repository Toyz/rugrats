---
number: 145
title: A climb's last step is had
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs
started: 2026-10-16T02:32:13-06:00
took: 2m
---

# 145. A climb's last step is had

When the climb animation runs out, the animator moves on to the next
state within that step. The climb's last step of root motion was lost:
Chuckie ended at -173, where the oracle shows -193.

The climb now keeps the animation's whole root motion (`Climb::whole`,
turned into the world at the climb's start). When it ends by running out
(`finish_climb(.., true)`), the body is placed by the whole motion. Its
height against the top goes into the ground offset `root_y`, for the
floor to take back, rather than snapping.

The frame where the climb hands over, against the oracle:

- Chuckie (level 10): -193, the oracle's -193;
- Tommy (hub): -125, the oracle's -125. That is 779 less the 900 band's
  904.

All play tests pass.

**Still unknown:**

- After the handover the original settles Tommy from -125 to the floor's
  -70 over about seven frames (-111, -115, -110, -93, -88, -80, -69).
  The port keeps a height above the floor until the run's keys move it,
  going -157, -165, -148. How the original's floor test brings a body
  down onto the floor is not traced.
