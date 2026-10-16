---
number: 108
title: No distance shading or lighting in play
date: 2026-10-15
area: render
files: docs/formats/level.md
resolves: 10, 11.2, 14.3, 15.4
started: 2026-10-15T23:04:29-06:00
took: 6m
---

# 108. No distance shading or lighting in play

Several entries left the "wall shading" open: the depth cue the room
renderer's code has, and the lighting of gouraud faces. Implementing the
depth cue in the port's shader showed the hub's back wall going black,
where the oracle's frame of the same view has it fully lit. So the
oracle was asked directly.

- The far colour is black: 0x8009fd38 (SetFarColor) is called only with
  (0, 0, 0), from 0x8001de38 and 0x8002a65c.
- The room renderer's depth cue (0x80011194: DPCS with IR0 = (SZ - 8192)
  >> 2 for each vertex past 8192) never runs. Hooks on 0x80011194 and
  the DPCS branch 0x800111ac never fired, in a hub run to frame 2562 or
  in levels 2 and 12 to 2700, while the room draw 0x80010e14 ran 7,400,
  9,669 and 1,764 times.
- The model path's lighting (0x80010710, NCS) never runs either: 0 hits
  against 47,330 calls of the model renderer 0x80010000 in the hub.

So the original draws rooms and models in play without distance shading
or lighting, as the port does. The shader change was reverted, and
docs/formats/level.md says so where it describes the depth cue.

**Still unknown:**
- what path would reach 0x80011194 (perhaps none in this build)
