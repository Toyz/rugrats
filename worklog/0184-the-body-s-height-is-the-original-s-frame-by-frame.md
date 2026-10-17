---
number: 184
title: The body's height is the original's, frame by frame
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T02:02:57-06:00
took: 1h 53m
---

# 184. The body's height is the original's, frame by frame

The body's height now matches the original's on every frame, root and all. Nine oracle scripts assert it (all but the pick-up, which asserts its carried thing's height instead, also exact). Until now the port kept the animation's root apart and the scripts compared x and z only.

What it took:

- **The original's order.** 0x8005a068 steps the animation (0x80056834) first and then runs the state's handler, whose asks start at the next step. The port asked and then stepped in the same frame, so its animations ran a frame ahead and its logic read the step before. The player now steps the animator, and takes its root, at the top of its frame; the handler's ask comes after. The animators' track records now equal the oracle's frame by frame through runs, jumps, landings, pick-ups and throws.
- **The take-off.** A standing jump leaves the floor on the step the crouch (state 5) gives way to state 6. A running jump (41) has no crouch: launched as its state is entered (0x8005b2d8 -> 0x80060f1c), it is in the air the step its animation begins. The launch heading is the way's from the frame before. The take-off no longer takes its own air step, because the handler's follows in the same frame.
- **The landing.** On the landing frame the body stays where the air left it (the root takes up the difference), and the landing animation's first frame blends from there.
- **The first frame's blend (0x80055904).** The body goes from where it was to the frame's root key by tz(D frac) of its time gone, in integers; a frame without a root key has key 0 (0x800557ec). The start (+0xde) is taken at the first step that spends time in that frame. If the animation began in the step the one before ran out in, that one's rest comes on top. If it began with no time left in that step, the start is taken the next step, rest included. The rest stays until the next start.
- **Asks per track (0x800572a8).** A base state asked again starts again: the stand's entry after a landing restarts the idle, and the root re-bases on any base start (`Animator::began`). An upper ask (a throw) leaves the base's last ask as it was, so the throw's end no longer restarts the idle or resets the speed it was played at. Standing with a thing asks for the idle on the base as well as the carry's arms, as the stand's entry does.

`Player::root_shown` (a stopgap that shifted the root a frame) is gone. The play tests that hold hand-sampled oracle heights (`running_bobs`, `a_jump_moves`, the climbs) are aligned by the press's own frame. The jump no longer needs its "crouch shows twice" exception.

**Still unknown:**
- The bump is table state 28 after a run in the original (0x80056e10 asks 0x800590d4) and 32 in the port: same frames and words.
- Before a script's press the oracle stands mid-fidget from its boot, so the track records differ until the first ask.
- The NPCs still ask before they step, the order the player had.
