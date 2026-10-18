---
number: 215
title: Level 5's robot wakes on the original's frame and turns as the original's: the trigger pass late in the frame, the turn in place held
date: 2026-10-18
area: engine
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T23:43:28-06:00
took: 37m
---

# 215. Level 5's robot wakes on the original's frame and turns as the original's: the trigger pass late in the frame, the turn in place held

Level 5's robot wakes on the original's frame and turns as the original's does: the trigger pass runs late in the frame, the turn moves nothing, and its animation is held.

**The pass runs late (0x80018c38).** The main loop calls 0x80072d38(0) after the objects' updates and the draw, so a character the pass schedules first moves the frame after. The port ran the pass before the characters' step, one frame early. It now runs after them, before the ambience.

**The turn within reach.** A guard standing within reach:
- **turns in place, nothing else.** The stick's 512 and the body's 160, but no body step (`steer` without the body). Before, the body's step dropped the robot and slid it toward Angelica as it turned.
- **plays the turn in place as long as it turns** (logical 53, state 63). It is held (`hold`), so the stand's ask at the step's end does not replace it, then let go.
- **snaps to the wanted heading.** The body's turn stops at it: 2048 exactly, where the stepped body had overshot to 2053.

**The tools' sync** gives an unsettled character the original's lift: its height (+0x1c) less the port's place, which the sync keeps (worklog 212).

**Oracle.** `level_5_robot_wakes_as_the_original`: Angelica put 1200 short of robot 13002, the port's pass given the original's place in the list (the entry after 0x800c4c20's node). The robot:
- holds still the original's 4 frames;
- turns on frame 5 (672, 1344, 2016, 2048);
- its heading and its base track (state 63 through the turn, then 0) are the original's every frame to the attack.

**Still unknown:**
- The robot's hover lift once it wakes. The original's closes the first frame's gap from -1244 toward its key (-1208, -939, -669, -515, -478). The port's root jumps to about -1120 and stays, so the robot is too high to be level with Angelica (0x80083ff0) and its first bite comes late.
- The guard's attack cycle as modes 5, 1 and 0 (animation 13, the bite 49), worklog 212.
- Other characters standing and turning (the turn in place for all).
