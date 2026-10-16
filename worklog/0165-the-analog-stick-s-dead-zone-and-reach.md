---
number: 165
title: The analog stick's dead zone and reach
date: 2026-10-16
area: input
files: crates/rugrats/src/input.rs
started: 2026-10-16T15:46:58-06:00
took: 1m
---

# 165. The analog stick's dead zone and reach

The analog stick's bytes now become axes as the original's do. 0x8001a888
updates the twelve axis objects at 0x800cdac0 each frame; an analog one
(type 2) takes its byte less 127, lets 24 either side of the middle count
for nothing, scales what is left back to the whole 127 (x 127 x
0x13e22cbd >> 35, about 1.233), then to 4096 (x 4096 / 127) and clamps.
The port had taken (byte - 128) x 32, with no dead zone, so a stick at
rest drifting a few counts walked the body, and full travel was 4064 one
way. `stick_axis` is the original's sum; its test checks the dead zone
and both ends.

The digital axes (type 4, two buttons) were read on the way: one button
gives -127 or +128, both or neither 127 (the middle), then the same scale,
so the D-pad is +-4096 on its axis, as the port has it (worklog 164's
open question on the analog scale is this one).

**Still unknown:** nothing
