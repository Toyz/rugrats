---
number: 19
title: The title's sign and buttons, from the original's cameras
date: 2026-10-14
area: ui, render, decomp, test
files: crates/rugrats/src/front.rs, crates/rugrats/src/gte.rs, crates/rugrats/src/stage.rs, crates/rugrats/src/cine.rs, crates/rugrats/tests/gte.rs, crates/rugrats/examples/stage_shot.rs, docs/engine/front-end.md
resolves: 18.1, 18.3
started: 2026-10-14T23:19:48-06:00
took: 6m
---

# 19. The title's sign and buttons, from the original's cameras

The title now matches the oracle's picture apart from the babies, who were
not on screen in the compared frame.

- The camera's projection comes from its field of view (0x8001dfb8). The
  port computes it with libgte's rsin and rcos from their table in the
  executable, and a test calls the original for every field of view.
  The opening's 955 gives the 178 measured before; the front end's 512
  gives 386. `stage::view` builds the view and projection for both.
- The sign is model 1038 at the origin, seen from (0, 30, -500), with the
  original's vertex wobble every other frame (0x8003f200).
- The trademark is a raw textured quad (0x8003f0f0).
- CANCEL and ACCEPT are font-2 labels with shadows, each beside a cube
  (model 334) seen by a second camera at (0, 0, -500). The cube eases its
  angle toward a half turn or a quarter (0x8003f7f0). Its matrix comes from
  0x80020ee0's Euler formula, now in `gte::Trig::euler` and checked against
  the original (with MatrixNormal stubbed). The cubes are drawn in the
  main camera's space, shifted by the cameras' offset, which is exact
  since both share rotation and projection.

`stage_shot` renders the title headless after a number of ticks. A zoom
on the top strip against the oracle's frame lines up to the pixel row.

**Still unknown:**
- the babies
- the menu's input, sounds and submenus
- whether MatrixNormal changes the cubes
