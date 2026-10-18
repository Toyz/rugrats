---
number: 223
title: A state left in a step keeps what the step spent on it, the next gap is taken from the height as the step began, and the oracle can run without lag frames
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/animator.rs, crates/rugrats-hle/src/lib.rs, crates/rugrats/examples/npc_check.rs, docs/engine/gameplay.md
started: 2026-10-18T10:02:20-06:00
took: 1h 40m
---

# 223. A state left in a step keeps what the step spent on it, the next gap is taken from the height as the step began, and the oracle can run without lag frames

The oracle can run as a console that never takes a third blank over a frame, and with it every playable level's characters match the original's in the height sweep but level 20's lunge start and level 21 after its knock. Two root rules came out of it: a state left in a step keeps what the step spent on it, and the next first frame's gap is taken from the height as the step began.

**The oracle's lag (`Hle::frame_steps`).** The oracle gives a frame a vertical blank once it has run 600000 instructions without a VSync call. Frames past that take three blanks, and the game's time (0x800c3e98) goes 15, then 0, a frame. Level 5 lagged every two or three frames and level 2 nearly every frame. The sweep tool skipped those frames and synced after them, so the oracle's two-blank steps cut the original's frames midway. Level 5's robot 17 looked a frame apart, and level 2's follower Chuckie was never compared at all.
- `Hle::frame_steps` (default 600000) is now a field.
- `npc_check` with `FAST=n` sets it (1200000 is enough; with no limit a loop that waits on the blanks never ends). The original then runs at dt 10 every frame, as the port does.

**A state left in a step keeps what the step spent on it (0x80055904).** The idle's fidget (0x800563bc) takes over at whatever frame's end, not only at the idle's last frame. The step had already spent its time on the idle's frame, and the original adds that frame's key times that time. Level 5's robot moves -37 for the last 10 of frame 1310's 20 before its fidget begins. The port moved the body only by a state's last frame.
- The animator now records where the base track's spending left a state before a begin in the same step (`Animator::left_from`).
- `RootCurve::step` adds the left state's height there less what it had (`Animator::root_height_at`).

**The gap is taken from the height as the step began (0x80055b3c).** The gap is the feet, the floor (+0x148) and the key less +0x1c. The step's sum (+0xda) goes into +0x1c only after the step (0x8005cb48), so the left state's rest is not in it.
- A first frame that spends time in the same step takes its gap there, and the rest lands on top. Level 2's Chuckie ends fidget 23 (frame 1088, +2) into the idle's 1022 and rises 3 over that frame, two units above the gap's target.
- A first frame begun as the step ends (the robot's fidget, its whole 25 to go) takes the gap on the next step, with the rest in the height by then.
- `RootCurve` keeps such a rest apart (`rest`) and adds it to its model.

**The sweep with FAST** (YCHECK, 300 frames from frame 0):
- No character is off in levels 1-7, 10-18 or holes 23-31.
- Level 9 matches from its sync at frame 20.
- Level 20 (PLAYERSYNC, CINES) is off on 3 frames and level 21 after its knock.

The REALIGN option, an extra blank after a lag, is gone: it changed nothing.

**Still unknown:**
- Level 20's three frames at a lunge's start (158-160): the original's heading turns by 352, then 160 a frame back. The lunge start (0x80084df0) falls through to the follow's speed and stick, and mode 8 (0x80084fc4) steers through the lunge (home past half the range with bit 2, else the player). The port's lunge does not steer.
- Level 21 from its knock: the player's own throw moves it within the port's frame, which the tool's placing of the player cannot follow.
- The view test's push from inside an object's box (the stale stack normal, worklog 221).
- Whether the real console lags where the oracle's instruction budget does: the budget is the oracle's, not a measured speed.
