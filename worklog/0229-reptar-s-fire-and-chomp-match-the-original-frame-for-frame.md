---
number: 229
title: Reptar's fire and chomp match the original frame for frame without lag frames
date: 2026-10-18
area: test
files: crates/rugrats/tests/hle.rs, docs/levels/level-07.md, docs/engine/gameplay.md
started: 2026-10-18T13:38:21-06:00
took: 9m
---

# 229. Reptar's fire and chomp match the original frame for frame without lag frames

Reptar's fire and chomp, which level 7's doc listed as running twice as fast in the port, match the original frame for frame. The difference was the oracle's lag frames.

**The measure.** The doc's 70 and 110 VSyncs for states 30 and 29 were read off an oracle that took frames of dt 15-20 in the city (the instruction budget, worklog 223). With `Hle::frame_steps` raised, Reptar put at (25279, -703, -1418) facing 2048 runs as follows:
- Circle: state 30 (animation 51) for 40 frames, from the frame after the press.
- Triangle: state 29 (animation 49) for 60 frames.

Both play at the written 2048 (template 8, `Animator::written_speed`), as the port asks them.

**Oracle.**
- `level_7_fire_as_the_original`: 51 frames.
- `level_7_chomp_as_the_original`: 101 frames.

In both the tracks, the height (root and all) and the place are the original's on every frame. The Unknown item is gone from level-07.md, and gameplay.md's list no longer has the floor flags and the shadow as unknown.

**Still unknown:**
- Whether anything can hurt Reptar in normal play.
- The flames' count against the original's (the port breathes one a frame from the event, as the code says; not counted in the oracle).
