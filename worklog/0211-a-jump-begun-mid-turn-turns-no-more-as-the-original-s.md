---
number: 211
title: A jump begun mid-turn turns no more, as the original's
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T20:59:29-06:00
took: 19m
---

# 211. A jump begun mid-turn turns no more, as the original's

A jump begun mid-turn turns no more, as the original's: found while trying to put the crouch over an edge in the oracle.

**The heading holds from the jump's first frame.** Tommy turning in place (Left held from the hub's start), Cross pressed mid-turn: the original's heading took that frame's turn (1568 to 1408) and then held at 1408 through the crouch and the air. The port's jump state left the wanted heading where the stick last set it, and the body's turn ran on toward it each frame (to 0 by the landing). Now the crouch's start (`pending_jump`) sets the wanted heading to the heading. The stick in the air was already not read for the heading; the air's push is unchanged.

**Oracle.** `hub_jump_mid_turn_as_the_original`: Left 3 frames, Left and Cross, Left through the air, let go on landing. Every frame matches the original's: places, heights with the root, headings, camera and tracks.

**The crouch over an edge, tried again.** R1 puts the follow camera behind on both after a put, which made level 6's -2048 step usable where the hub's step was not (its room's camera does not follow, the original's stays at 0). The stick then goes the same way on both. But the put there sits on a different floor in the original (226 higher), and the run turned before it ran, so no crouch crossed the edge.

**Still unknown:**
- The crouch carried over an edge, frame by frame in the oracle.
- After a mid-turn jump's landing with Left still held, the original slides 40 across and turns a little slower than the port.
- Why level 6's put at (44500, -2293, 13300) settles 226 higher in the original.
