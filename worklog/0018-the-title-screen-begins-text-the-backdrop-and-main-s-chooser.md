---
number: 18
title: The title screen begins: text, the backdrop, and main's chooser
date: 2026-10-14
area: ui, render, tooling, test
files: crates/rugrats/src/text.rs, crates/rugrats/src/front.rs, crates/rugrats/src/stage.rs, crates/rugrats/src/main.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/disc.rs, crates/rugrats/tests/text.rs, crates/rugrats/examples/stage_shot.rs, crates/rugrats-render/src/renderer.rs, crates/rugrats-render/src/mesh.rs, crates/rugrats-hle/src/gpu.rs, crates/rugrats-hle/examples/gpu_frame.rs, docs/engine/text.md, docs/engine/front-end.md
resolves: 16.1
started: 2026-10-14T23:01:18-06:00
took: 19m
---

# 18. The title screen begins: text, the backdrop, and main's chooser

Main's mode chooser (0x80030158) answers where the opening plays with a
pad in. It cycles: the front end with a 30-second idle countdown, then a
demo movie picked by `rand() % 3`, then the opening cinematic. So the port
now goes from the logos to the title. The demo and the opening come back
once the countdown is ported.

The text system is ported (`rugrats::text`): the fonts in PHOUSE.FNT at
VRAM (960, 256), the glyph table, centring (0x8002e144), the glyph sprites
(0x8002e67c), and the slides, bounces and wobbles (0x8002e2b4). Tests run
the original's three functions in rrt-emu and compare. The sprite test
reads the packets 0x8002e67c writes back out of its buffer. All three
agreed on the first run.

The title's first layers are in: the backdrop's two fading layers and the
menu's items sliding in. Two rendering pieces were needed:

- An ordered 2D layer under the world (`Renderer::set_under`). The
  backdrop sits at the back of the ordering table, under the 3D sign and
  babies, and its layers are one opaque and one added.
- The GPU's texture window, as a mode bit (`Vtx::WRAP`). The backdrop's
  quads are 256 pixels across over texels 0-255, through a 128-texel
  window. Drawing them as 128-pixel tiles instead overflowed the 8-bit u
  (128 + 128) and showed the pattern as stripes.

The oracle's GPU had no texture window either (GP0 0xE2), so its title
showed raw quadrants of different patterns. It does now, and its title
picture shows one tiled pattern, as the port's does.

Also: a Stage (VRAM, fixed and moving triangles, layers under and over,
camera, clear colour) that the cinematic and the title both fill, and
that main draws; the executable's strings and tables through `GameDisc`;
a `gpu_frame` oracle example that lists every primitive of a frame.

**Still unknown:** the sign (model 1038) and its camera; the babies; CANCEL/ACCEPT's button icons and the TM; the submenus; the front end's sounds
