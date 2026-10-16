---
number: 89
title: The box toss arrow and lob
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/box_toss.rs crates/rugrats/src/ui/hud.rs crates/rugrats/tests/play.rs docs/levels/level-20.md
resolves: 46.1
started: 2026-10-15T21:41:53-06:00
took: 12m
---

# 89. The box toss arrow and lob

Worklog 46 left two stand-ins in levels 20 and 21: a small marker for the
arrow over the target, and a lob fitted to land on the baby.

**The arrow** (0x80031a28). It is drawn for the object in 0x800e2d3c (the
target) in levels 20-21, from the character draw (0x8003205c). It is not
a model: it queues a camera-facing quad (0x8008afc4, drawn by 0x8008a954)
with:

- face flags 0x800c4004 = 0xd0800000, read in the oracle: top bits 3 with
  bit 23 and not 24 or 29, so 0x800233f4, an opaque FT4;
- half-size 500 x 900 (0x800c4008), turned 2048, so the sprite points down;
- the texture at the level's TEXPOS table + 0x1110, entry 364 of the
  12-byte entries (the menu table 0x800b9ba4 if none is loaded);
- its centre max(|+0x7a|, |+0x82|) + 400 (0x800c400c) above the target.
  +0x78..+0x84 is the bounds cube 0x80046570 sets. Peeked in the oracle on
  level 20, it is +-1773 for Angelica and all four babies, so the arrow
  sits 2173 up;
- grey 208 while the flip counter 0x800c3f24 mod 10 is under 5, else 128.

`Sprites::billboard_lit` draws a billboard with that grey. The port's shot
shows the yellow arrow pointing down over the baby in front.

**The lob** (0x80057bc4). It runs from the target picker (0x80059b34) in
levels 20-21 each frame a target is held, and fills player +0x200..+0x20c:

- direction (0, -2048, 2048) turned toward the target and made unit, so
  45 degrees up;
- speed isqrt(s * 0x180000 / 4095) with s = 20 (d - 1024) >> 10, in
  unsigned 32 bits, d the flat distance. 4095 is 2 sin 512 cos 512 >> 12.

The throw (0x80060b44) passes those to the flight in place of the plain
throw's (0, -384, 1024) and 170, so velocity = dir * (speed * 65536 /
6000) >> 12 as for every throw. A new test lobs from 2500, 5000 and 8000
away; each reaches a baby and swaps the player into it.

**Still unknown:**
- the lob's other case: with 0x800c56a8 bit 64 (set by 0x8005f3a4, which also makes 0x800e5778 the target), direction (0, -4096, 100) at speed 100; when that happens in play
- the bounds of characters outside levels 20-21 (only those here were peeked)
