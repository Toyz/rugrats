---
number: 105
title: The camera after a cry
date: 2026-10-15
area: render
files: crates/rugrats/src/world/game/story.rs crates/rugrats/src/world/game/camera.rs crates/rugrats/tests/play.rs
resolves: 40.3
started: 2026-10-15T22:59:41-06:00
took: 2m
---

# 105. The camera after a cry

40.3 left the camera after a cry. While Tommy cries, player +0x188 bit
64 holds the camera. When the hold ends (0x8002953c, 0x800c4a3c bit
0x40000000), the camera is reset:

- normally by 0x800290a4: the follow camera, the player's room, the
  level's height (0x800c3f5c: 1024, 512 in levels 4 and 15, -1000 in
  7);
- in levels 9, 20 and 21 instead by 0x800291f8(camera, player, +0x4a,
  0), its own kind again, so a room's fixed camera (kind 3) stays. The
  level 9 doc saw +0x4a = 3 before and after the cry.

The port kept whatever camera it had. `cried` now drops the room's
fixed post and puts back the level's height, except in 9, 20 and 21.
The test `the_camera_after_a_cry` checks two cases: level 9 keeps a
kind 3 room's camera after a cry, and the first other level with such a
room gives it up. `Game::fixed_camera` reports the post for tests.

**Still unknown:**
- whether the next CameraSwitch in the same room sets the fixed camera again (the port waits for a room change, as 0x80077c88's "same kind" skip would)
