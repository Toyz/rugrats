---
number: 97
title: Cinematics fade out at their end; the overlay's semi-transparent shapes are drawn
date: 2026-10-15
area: video
files: crates/rugrats/src/media/cine.rs crates/rugrats/src/ui/hud.rs crates/rugrats/src/app/play.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/stage.rs crates/rugrats-render/src/renderer.rs crates/rugrats/examples/opening_shot.rs
started: 2026-10-15T22:38:16-06:00
took: 4m
---

# 97. Cinematics fade out at their end; the overlay's semi-transparent shapes are drawn

Two fixes.

**The cinematic's end fade** (0x8007424c, docs/engine/cinematics.md).
For each frame shown with fewer than 13 frames of the stream left, unless
the music byte has 1, 0x800c3f2c rises by 25 (to 255) and the flip draws
the screen fade (0x8001cf2c). Bit 1 is set for cines 0 (the opening), 3,
4 and 39. Cine 64 measured in the port: the mean brightness holds at 169
to frame 163, then 144, 120, 98 ... and 0 from 174 of 176.

`hud::fade` is the screen fade (a POLY_F4, draw mode 0x640, the screen
less the grey). `Boot::Fade` (96) and the cinematic share it.

**The overlay's semi-transparent shapes were never drawn.** The
renderer has a list for them (`set_overlay_semi`), but nothing filled
it. The opaque overlay pass skips semi-transparent triangles, so 96's
fade drew nothing either. Two changes:

- `Stage::overlay_semi` picks them out. The app and the shot tools
  (opening, stage, level, play_script, card, screen) now hand them over.
- The renderer draws them after the opaque overlay, so a fade covers the
  HUD as the original's flip covers everything.

Nothing else on the overlay is semi-transparent today. The title's
added layer goes under the world, so nothing that drew before changes.

**Still unknown:**
- the overlay's semi-transparent and opaque shapes keep their own order (all opaque, then all semi), not the original's ordering table; nothing yet mixes them
