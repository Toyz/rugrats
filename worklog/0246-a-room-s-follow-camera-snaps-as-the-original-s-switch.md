---
number: 246
title: A room's follow camera snaps as the original's switch
date: 2026-10-23
area: play
files: crates/rugrats/src/world/game/rooms.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/story.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-23T05:25:29-06:00
took: 1h 12m
---

# 246. A room's follow camera snaps as the original's switch

A room's follow camera now switches as the original's: a camera byte different from the last runs the follow setup, which snaps the camera behind the player at the new height. Before, the port only changed the height and let the eye ease toward it at 9 a frame.

**What the original does.**
- **CameraSwitch** (0x80077c88): on the room's entry, flag 4, the room's camera byte is compared with the last one switched to (D_800c4410). 0x80077070 (a level's start) and 0x8007737c (a cine's end) set D_800c4410 to 1.
- **A different byte** is passed to 0x800291f8. Byte 6 becomes kind 1 with height 200; 7 and 11 become 500; 8 becomes 1600; 9 becomes 2000; 1, 10 and 12 become the level's own (1500); byte 13 passes a fifth argument of -1.
- **0x800291f8 sets the height** (D_800c3f5c).
- **Then, for kind 1:**
  - with the camera in mode 1 (+0x4a, just set up), nothing more;
  - else, but for the fifth argument -1, 0x80025e90, the follow setup: the eye 4096 behind the player at his height less 768 and the height, the camera in mode 1, then the reset (0x80026ddc).
- **Kinds 6 to 11 of 0x800291f8** do nothing but the height. CameraSwitch never passes them.

**Seen.** Level 1, the player put in room 19 (byte 6) from room 23 (byte 8). The original's eye stands 200 over the look-at at once, at (24281, -189, 20153). The port's stood 1024 over it and would have taken about 90 frames to come down.

**The port** (`Game::room_change`):
- It keeps the byte switched to last (`camera_byte`): 1 at a level's start and after a cine.
- It keeps whether the camera was just set up (`camera_set`): from a level's start, or after the camera frame's own setup or the room's, until the next camera frame.
- A new follow byte, not 13, with the camera not just set up, now runs `set_camera_yaw` (0x80025e90 and the reset) after the height is set.
- At a level's start the camera was just set up, so the start room's byte sets the height alone, as on the original. Level 1's start room is 23, byte 8: the original's eye stays 1024 over the look-at then.

**Checked:**
- Level 1 room 19 after a put: the eye's height is the original's on every frame, (24309, -189, 20153) at rest against the original's (24281, -189, 20153).
- The hub's run, turn and jump tests, which a snap at the level's start had broken in a first try.
- Every test file, and the hle suite.

**Still unknown:**
- That place in level 1 as the player runs: the original's eye swings out in z (to 18659 in five frames) and its heading with it, the port's does not. Its flags there (0x2100000, 0x300000) are the pushes' signs. The likeliest cause is the view test's push along a stale stack slot holding an object's address (gameplay.md, Camera), which the port cannot know.
- The eye's x at rest after the snap: 28 off (24309 against 24281).
