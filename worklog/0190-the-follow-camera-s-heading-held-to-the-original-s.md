---
number: 190
title: The follow camera's heading held to the original's
date: 2026-10-17
area: render
files: crates/rugrats/src/world/game/camera.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T04:51:12-06:00
took: 37m
---

# 190. The follow camera's heading held to the original's

The follow camera's heading is now asserted in six oracle scripts (the hub's run, the playpen, the tap turn, the standing and held jumps, Spike at the wall). It matches the original's on every frame. In two others it is off on a few frames only.

- **The harness's own rounding.** The ±1 the scripts showed now and then on turning frames were the harness's. It read the original's heading off its +0x224 matrix with a float atan2, where the game uses its table ratan2 (0x800a0328). The port's rows equal the original's matrix on those frames; the heading is now read with the game's own ratan2 (`Game::heading_of_rows`).
- **The camera's aim height.** The aim point (0x80027304's local_150) and the look-at's height goal take the body's height as the original's +0x1c has it, with the animation's root (`pos[1] + root_y`). Through a landing's squash the original's look-at holds its height while the port's followed the bare position. The running jump's landing camera now matches but for the script's last frame.

`Game::camera_rows` gives the rows for tools; the CAMLOG line prints them beside the original's matrix.

**Still unknown:**
- The first frame after the running jump's landing: the original sets no speed from the stick (+0x184 stays 0, the way +0x178 is set, the stick read at 4096 in +0x1b0), so its camera rests where the port's pushes on. By the handler's code (0x8005be24) only a turn over 1024 from a standstill, or a dead stick, skips the speed. Neither fits on the face of it, so this needs a hook inside the handler.
- After a bump from rest (the seat; the slide's end) the original's camera holds where the port's follows.
- Level 6's room camera: the port reports no heading from a fixed camera, where the stick still reads the original's +0x224.
