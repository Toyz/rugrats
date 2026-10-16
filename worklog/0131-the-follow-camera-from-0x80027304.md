---
number: 131
title: The follow camera, from 0x80027304
date: 2026-10-16
area: engine
files: crates/rugrats/src/world/game/camera.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/rooms.rs, crates/rugrats/src/world/game/story.rs, crates/rugrats/src/levels/aisle_seven.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/tests/camera.rs, crates/rugrats/tests/play.rs
resolves: 100.1
took: 45m
---

# 131. The follow camera, from 0x80027304

The port's camera eased its yaw toward the player's heading and its eye
toward a point behind him. It is now 0x80027304, ported in camera.rs.
The work was done by a delegated agent and merged here.

**The eye.** It sits on a spring. Its velocity is integrated by
0x8004fbb4 with dt 10, has drag of |v| / 32 and a top speed of 2796, and
stops when the velocity reverses. It is pulled with 16 (10 when
floating) toward the goal 4096 behind, which 0x8002cd7c clips. Across,
the eye stays between 1024 and 4800 (7200 in level 7).

**The swing.** With the player facing the camera and nearer than 2049,
the eye swings aside (74 / -18; 37 floating). The side is picked by a
2048 wall test and the 4096 timer. Between 2049 and 4096 it backs off
along the player's axis.

**The height.** The goal is the look-at's y less the height. It keeps
256 off floor and ceiling (0x8002b240) and stays under ceiling hits over
the eye and the player. The push is 9 (4 floating, 18 in level 15),
settled within 64, with a band of 1512. The branch for "blocked at eye
height, clear lower" is in too.

**Probes and tests.** The wall probes (0x800263c4): 37 off a wall ahead,
up to 74 off walls to the sides. The view test (0x80026914): from 512
ahead of a running player, 18 off what blocks, plus 32 toward the goal.

**Settling and idling.** Per-axis sign bits, and the "still" bits for
each axis while the player stands. When idle, the orbit (0x800284f4) at
512 runs until the eye is at the distance and past the player's back.

**The look-at** (0x800de454). It looks ahead by speed x 150, scaled
nearer than 5192, and eases by 1/4. Its height catches up at 298 (223
floating) with a band of 512.

**The reset** (0x80026ddc). It uses the 8 offsets at 0x800b6e18. R1
takes the first clear one; a stuck camera takes the nearest.
`set_camera_yaw` places the eye (0x80025e90) and resets. The end of a
fixed or shaft camera resets too.

**The room clamp** (0x8002953c). An eye outside every room is drawn
clamped.

**The stick.** It is turned by the camera's matrix, ratan2(-m[0][2],
m[2][2]) as 0x8005be24 does. Under a fixed camera the stick now follows
that camera's view.

The look-aside (L1/L2/R2), `cam_hold`, the shaft camera and the fixed and
post cameras are kept.

**Checked against the original.** The oracle's recorded player path in
the hub was replayed into the port's camera from an R1 reset. These
windows matched frame by frame:

- 2784-2854: turning and walking, with the 4800 clamp;
- 2784-2834: walking toward the camera;
- 3004-3200: left, stop, the idle orbit back behind;
- 3204-3400 and 3404-3486.

Examples:

- the reset at 2782: eye (9301, -1589, 18229) in both;
- frame 3200: eye (8010, -1589, 16792), look-at (7267, 11, 20958) in
  both;
- frame 2856: (7647, -1589, 19173) against (7666, -1589, 19175).

tests/camera.rs replays two of those windows. Changing an expected value
makes it fail.

**Play tests.** Three needed different input, with the same assertions:

- Levels 18 and 15: behind Tommy is walled off in the height field, so
  the reset puts the camera in front, as 0x80026ddc would. `collect`
  now steers by the camera.
- Level 6: the camera turns about 15/4096 after the first frame's push,
  so the test presses R1 before the climb.

The oracle logs are in /mnt/data/claude/scratch/cam/ (run1-run5 and
their tables).

**Resolved from earlier entries:**

- 100.1: the camera's side-step tests.

**Still unknown:**

- 0x80090554, the line test against the room's polygons, is stood in for
  by the height field (`collide::segment`). That is the main mismatch:
  the hub start's side probe hits a doorway wall in room 35 that the
  height field lacks, and the oracle's view test hits polygons at 2836,
  2856 and 3490.
- Not ported:
  - flag 0x4000 (following another object);
  - flags 1 and 2 with 0x800c4a48 (nothing found sets them);
  - flag 0x80000 (the halving blend);
  - level 11's look-at snap with player +0x188 bit 512;
  - +0x188 bit 2048 (0x80062a08).
- Level 7's +0x148 aim uses the player's floor; this is not checked.
- The player's "moved" bit (+0x188 bit 32) is taken as x or z changing
  this frame.
- The camera always steps at dt 10. The oracle's lag frame (dt 15) at
  3488 is where they part.
