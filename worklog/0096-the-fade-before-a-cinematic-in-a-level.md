---
number: 96
title: The fade before a cinematic in a level
date: 2026-10-15
area: video
files: crates/rugrats/src/app/flow.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/app/play.rs crates/rugrats/src/audio/sound.rs docs/engine/sound.md
started: 2026-10-15T22:35:40-06:00
took: 3m
---

# 96. The fade before a cinematic in a level

95 left the screen's fade before a cinematic posted inside a level
(0x80018ad0). The port went straight to the cinematic.

Unless the cinematic's music byte has 16, the original:

- redraws the level as it stopped (0x8001886c: the camera and draw
  calls, no player or world update) for 127 dt;
- raises 0x800c3f2c by 256 / (127 / dt) a frame (21 at dt 10), to 255
  at most;
- the flip (0x8001d00c) draws it with 0x8001cf2c: a POLY_F4 over the
  whole screen (+-256 x +-120 about the centre), semi-transparent, draw
  mode E1 0x640, so blend 2: the screen less the grey.

The picture darkens to black over 13 frames. With the byte's 32 the song
starts its 60-tick fade down at the same moment (0x8006ef0c), and the
pause at the cinematic's start (95) catches it part way.

The port's `Boot::Fade` holds the level and its stage. Each game frame
it replaces its quad on the overlay with the next grey (`Vtx::FLAT |
SEMI`, page blend 2), then opens the cinematic with `Then::Resume`.
`Spu::fade_song` gives the song's fade-down.

**Still unknown:**
- whether events queued during the fade (the loop repeats while 0x800c3e9c refills) can post a second cinematic straight after; the port plays one and goes back to the level
