---
number: 182
title: The carry and the throw's start in the original's integers
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T01:23:47-06:00
took: 17m
---

# 182. The carry and the throw's start in the original's integers

The carried thing and its throw now match the original's from the grab to the first thing the throw meets (`hub_pick_up_as_the_original`, ten flight frames held exact):

- The grip and carry are integer (0x8005ff88): the hands' midpoint less hand 15 turned into the body's frame by the transposed matrix, ApplyMatrixSV (0x8009fc38); the carry is hand 15 plus the grip turned back (0x8009fbe8), both by the pose as last drawn. The carry first moves the thing on the frame after the grab.
- At the release the carry runs first: mode 1 comes before the throw's handler. The oracle, stopped at the throw's descriptor (0x80061f88), had the toy at (4387, 98, 14140), where the hand as last drawn held it, not where the frame before had put it. The port used to let go from the frame before's place, 96 to the side and 159 short.
- The throw's velocity is integer: (0, -384, 1024) (D_800b7fe0) turned by the body's angles (0x80075ee8), made a unit (0x8007652c), times (170 << 16) / 6000 >> 12 (0x8008864c). With floats the launch was -651 down, not the original's -653, and the flight drifted a unit every few frames.
- The descriptor starts the flight at the object's position plus its bound box's centre (+0x78..+0x84). The oracle's toy has a cube of 483 about it, so the centre is 0 here. Its radius is the larger of the first box's y extents (279). The flight writes its position into the object's (+0x30 of the record points at +0x18).

`examples/throw_check.rs` gained hooks on 0x80061f88 and 0x8008864c (HOOKS=1), the bound and flight record (BOUND=1) and the part quaternions (QUATS=1).

**Still unknown:**
- After ten frames the original's toy meets a thing in the air (frame 72 of `throw_check 1 8`) and bounces. The port's thrown flight tests no things (0x80086668: things first through the rooms along the step, 0x80086340 -> 0x8006a824, then the world, the world's hit kept only when nearer on all three axes), and its world part is still the port's own wall test, not 0x80087594's.
- Things whose template +0x30 is 1 fly by 0x80086ef0 instead, not yet read.
- The put-down's flight (0x80062ac4: the same descriptor along (0, 0, 4096) at speed 10, after 0x80060810's wall test) is still the port's slide.
