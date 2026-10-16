---
number: 94
title: Pie splats
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/pie_toss.rs crates/rugrats/src/levels/aisle_seven.rs crates/rugrats/src/levels/training.rs crates/rugrats/examples/template_bytes.rs docs/levels/level-03.md
started: 2026-10-15T22:23:56-06:00
took: 6m
---

# 94. Pie splats

A pie's flight ends in a splat (0x8005282c). The pie's template, 564
(model 885), has +0x0b = 1 in both levels 3 and 16, so both toss
levels splat. `examples/template_bytes.rs` prints a template's record.

Each of the five bits is a flight (0x8008864c) with:

- flags 0x30705: kind 5, gravity 13, world collision;
- lifetime 2000 dt and bounce 1000;
- sprite 359 (TEXPOS + 0x10d4), half-size 200 x 200, face flags
  0xd0800000;
- speed min(rand mod 150, 50);
- a direction tilted rand mod 2048 and spun rand mod 4096
  (0x80088bac(pie, (4096, 0, 4096), 0, 2048)).

The two random numbers come from their own LCGs (0x800c41a4 and
0x800c445c, x 214013 + 0x269ec3), seeded as the executable has them.
Kind 5 flights are drawn by 0x80088388 through 0x8008afc4: camera-facing
quads like the box toss's arrow.

`pie_toss::Splats` spawns five where a pie stops: a wall, a hit or its
range. It steps them (gravity, a bounce on the floor, the lifetime) and
draws them with the pies. Seen in the port: the bits fly off the stall
the pie hit.

**Still unknown:**
- how 0x80085f44 turns the tilt and spin through the (4096, 0, 4096) axis and the pie's own vector; the port tilts from straight up
- the bits' colour (flight +0x6c, 0 in the descriptor as read; drawn at 128 here) and their roll (+0x24, desc +0x42 = 3391)
- how the world collision (0x80087594) bounces them; the port reflects y and damps all by 1000 / 4096
