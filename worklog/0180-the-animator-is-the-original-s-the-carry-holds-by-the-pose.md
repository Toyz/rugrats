---
number: 180
title: The animator is the original's; the carry holds by the pose as drawn
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/actors/player.rs
started: 2026-10-17T00:18:49-06:00
took: 47m
---

# 180. The animator is the original's; the carry holds by the pose as drawn

The animator was a reconstruction: each track restarted from its first frame with no memory of the frame before, upper states were a flag, and the pose was a float slerp rebuilt from scratch each frame. It is now the original's (0x80056834 and the routines under it), checked against the oracle's track records (+0xa8, four of 12 bytes) frame by frame. `examples/throw_check.rs` plays a pick-up and a throw in level 1 on both and prints the tracks, the hands and the thing.

How the original runs its tracks:

- Each track is state, frame before, frame showing, time left, phase (0 to start, 1 playing, 2 done, 3 held) and way back. A call to 0x800572a8 only names the state and sets the phase to 0. The base track takes the state when its entry has a base column; otherwise track 1 takes it as an upper state.
- A track starts (0x80055560) at its column's first frame and keeps the frame before, so the first frame blends from whatever showed last. The port used to blend from the first frame onto itself.
- Time runs down by dt * speed / 4096 (0x80055d40). On an animation's first frame after a shorter frame, the time left also loses cur / prev durations on every step. With that rule the throw's release and end come a frame sooner; the oracle's pick-up and throw release agree.
- At a column's end the state's word decides (0x8005562c): 4 ping-pong, 2 loop, 1 complete (phase 2, frame -1, confirmed in the asm), 8 hold (phase 3). When the base completes (0x80056484), it goes to the next half or turn, else to +0xe6's state.
- Track 1 follows the base (0x800561b0, 0x80056088: same offset into its own column, same time and phase), except while it holds an upper state (19, 26, 50, 51, 53; 0x80056708) under a base without flag 64. When an upper state completes (0x8005678c), track 1 goes to +0xe7's state, or with 17 ("as you were") follows the base again. Tracks 2 and 3 follow track 1 where its state has their column, else the base.
- Standing in state 0 runs the fidget timer down: 1200 dt, then 23 and 24 in turn (0x800563bc).
- The samplers (0x800552bc) blend each part from the frame before into the frame showing by (dur - left) / dur, with 0x800202ec's integer slerp (acos 0x80021500 and rsin; straight lerp when cos >= 3841). The result goes into a pose kept between frames, so a part a frame does not key keeps its last turn.

The grab sets +0xe7 to 20 (0x800601e4; not for template 933, 886 in levels 20 and 21, or Spike), so when the pick-up ends the arms go straight to the carry. That frame they blend from the pick-up's last frame (619), as the oracle shows. Put-down and throw set +0xe7 back to 17.

The hands: the original's part matrices (+0x158) are made by the draw, and the draw runs before the update. So the grip and the carry (0x8005ff88) read the frame before's pose. The port now keeps the pose as last drawn (`Game::pose_before`) and the grip is integer: the midpoint halved toward zero, plus half the deepest box (0x8005fd58). The carried thing now sits within 1 to 3 units of the original's on every frame of the pick-up and carry. It used to lead by a frame and sit 40 to the side, and it snapped to the idle arms when the pick-up ended. A body swap (box toss, gold rush) or a scripted hand-over drops the kept pose; otherwise the new holder would carry the bottle where the old one stood.

The port's frame order still asks before it steps, so its tracks run one frame ahead of the original's. Its logic reads the step before, which is what the original's handler reads after its own step, so behaviour is unchanged. Two things had leaned on the old animator:

- The climb read its root move right after asking. A state asked and not yet begun now reads as at its start.
- The bump's root move kept running into the idle once leftover time carried over. The original adds x and z only for states whose word has 32 (`moves_across`).

All 9 oracle scripts match on every frame again. The play tests pass; the throw test now expects the original's frame numbers.

**Still unknown:**
- The pose matrices are still float (quat to matrix, the hierarchy's composition): the 1 to 3 units left between the hands and the original's +0x158.
- The bump is table state 28 in the original after a run (0x80056e10 asks 0x800590d4) and 32 in the port. The frames and words are the same.
- The game's own calls to 0x800572a8 are approximated by `set` (ask when the logical animation changes, or when its track does not show it). The original asks from each state's handler.
- A thrown thing's flight starts at its bound box's centre (0x80061f88: object + the midpoint of +0x78..+0x84) and moves the object from there. The port's thrown flight starts at the object's position.
