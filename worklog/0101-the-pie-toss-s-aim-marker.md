---
number: 101
title: The pie toss's aim marker
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/aisle_seven.rs crates/rugrats/src/ui/hud.rs crates/rugrats/examples/play_script.rs
started: 2026-10-15T22:46:53-06:00
took: 3m
---

# 101. The pie toss's aim marker

Level 3's pie toss shows where a pie would go. While Tommy holds a pie,
the carry draw runs 0x8005fe44: the point (0, 1024, 24000) turned by his
heading and added to his position (0x800c42f8, 0x800c42fc). It is pulled
in to the first hit of the ray 0x80090554, from Tommy to that point:

- the world, by the segment test (0x8002cd7c, ported in 99);
- or an object of template 590 or 591, which are Larry's and Steve's
  bodies (hierarchies 10 and 11).

The player draw (0x80031db8) puts sprite 426 (TEXPOS + 0x13f8) there.
It is an FT4 at the texture's own pixel size centred on the projected
point, coloured (255, 0, 0) from 0x800c402c..34, opaque (0x800c4020 is
0).

The port's `AisleSeven::marker` computes the point; `Sprites::at_pixels`
draws a sprite at its own size. Larry and Steve are tested against their
bounds cube along the ray. Seen: the red mark on the back wall ahead of
the counter while a pie is in hand. `play_script` gains FINISHED=n (a
cinematic's end hook) to get into the toss.

**Still unknown:**
- the object test 0x8006a714 itself (the port uses each body's bounds cube for Larry and Steve)
