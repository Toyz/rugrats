---
number: 88
title: The boot order and the opening's ending
date: 2026-10-15
area: engine
files: crates/rugrats/src/media/ending.rs crates/rugrats/src/media/cine.rs crates/rugrats/src/app/flow.rs crates/rugrats/src/app/mod.rs crates/rugrats-data/src/cine.rs crates/rugrats/tests/boot.rs crates/rugrats/examples/cine_sound.rs crates/rugrats/examples/cine_unknowns.rs docs/engine/cinematics.md docs/engine/front-end.md
resolves: 15.3
started: 2026-10-15T21:22:59-06:00
took: 19m
---

# 88. The boot order and the opening's ending

Two reports: the opening played after the demo instead of at boot, and the
milk squirt at the end of the opening was missing.

**Boot order.** main (0x800185f8) shows the legal still and the three logos,
then plays the opening (0x80074958(-1, 0)), then enters the chooser. The
chooser (0x80030158) loops title, demo, and the opening again only when the
demo ran to its end: 0x8008b448 returns 0 then and 1 when a button skipped it,
and a skip goes straight back to the title. The port now does the same:
the third logo leads to the opening, a demo that plays through leads to the
opening, a skipped one to the title (app/flow.rs). `movie()` reports whether
the movie was skipped.

**The opening's ending.** Cinematic 0 has one timed call, 0x8008ea70, for
stream frames 1184 to 1413, with the frame counter as its argument. Read
from the code and checked against oracle peeks (counter 1218 at vblank 9220,
1246 at 9250):

- Camera A (0x800e54d8, unturned) starts at (2250, -2450, -5000) and moves
  (-75, 81, 160) a frame up to frame 1213, toward the milk (model 948) at the
  origin, which turns 273 a frame about z. The spurt starts small at the
  bottle's tip and fills the screen white.
- Once A's x is 0 or less, camera B (0x800e5588, at (0, 30, z)) brings the
  sign (model 1038) in, z from -17048 by 600 a frame to -500. Then A's z is
  -300 and B's field of view of 512 sets H 386 (SetGeomScreen) for both.
- From the next frame the text shows: "Search For Reptar" (font 1, y 60) and
  "Loading" (font 0, y 85), green (100, 200, 0) with shadows (1, 1, 1) two
  pixels down and right, and the trademark (0x8003f0f0).

This is the title card 15.3 asked about. The squirt sound is in the stream's
XA soundtrack already (a peak near 40.0 to 40.5 s, found with
examples/cine_sound.rs), so nothing extra plays it. The port's frames now
match the oracle's: spurt, white, sign, text.

Two fields of the cine records were named while looking: every frame's
+0x02 is 1380 and every placement's +0x16 is 0 (`unknown_02`, `unknown_16`).

tests/boot.rs now drives the program by state, not by fixed frame counts, and
checks the untouched boot goes legal, logo, opening, title, demo.

**Still unknown:**
- what the frame's +0x02 (always 1380) and the placement's +0x16 (always 0) mean
- whether the original draws the milk and sign through the ordering table with the scene's depth or, as the port does, over it
