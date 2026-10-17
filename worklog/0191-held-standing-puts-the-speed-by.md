---
number: 191
title: Held standing puts the speed by
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T05:27:52-06:00
took: 17m
---

# 191. Held standing puts the speed by

Standing with the way held puts the speed by. The stand state's handler (0x8005d11c) probes ahead (0x80064a34, mode 1) when the stick would run. If the probe holds (1), the body stays standing and its speed (+0x184) goes into +0x176 and is set to 0. The port kept standing but kept the speed.

Hooks on the stick handler's branch points (0x8005be24: the turn over 1024 at 0x8005c7e8, the speed set at 0x8005c8c4, the dead stick at 0x8005c8d8) showed the original setting the speed on the frames after the running jump's landing. The stand handler then zeroed it: toy 403 is beside him, so the probe held until he had turned away. With the speed 0 the follow camera rests (its standing branch) as the original's does.

The follow camera's heading now matches the original's on every frame of eight scripts, which assert it. The slide's end and the running jump's landing are added.

The harness's `put` also writes the body's matrix (+4) from the heading, as 0x80020d10 makes it. The original rebuilds +4 only as it turns, so a put used to leave it at the old heading.

**Still unknown:**
- The seat script: on the first frame from a standstill the original's way (+0x178, the body's matrix at +4 turned onto (0, 0, 4096)) points 45 degrees off the heading it was put at, even with +4 written, and its speed is 0. Something else rebuilds +4; the follow camera parts from there.
- Level 6's room camera (the port reports no heading from a fixed camera).
