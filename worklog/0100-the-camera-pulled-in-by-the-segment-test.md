---
number: 100
title: The camera pulled in by the segment test
date: 2026-10-15
area: render
files: crates/rugrats/src/world/game/camera.rs crates/rugrats/src/world/collide.rs
started: 2026-10-15T22:45:07-06:00
took: 2m
---

# 100. The camera pulled in by the segment test

The follow camera (0x80027304) keeps its eye in front of walls by
walking the segment from the aim point (the player less 768 in y) to the
eye goal with 0x8002cd7c. When that returns 1 (stopped), the eye goal is
the point where it stopped. The port stood in for this with a 32-step
march stopped at 90%; `clear_eye` now calls `collide::segment` (99) and
takes the point. `Segment` derives Default.

Seen in the hub running into the patio doors and past the gates: the eye
stays in the open.

**Still unknown:**
- the camera's side-step tests (0x80027304's two further 0x8002cd7c calls, 2048 to the side, flags 0x800c4a3c 4 / 8 / 0x10000) are not ported
