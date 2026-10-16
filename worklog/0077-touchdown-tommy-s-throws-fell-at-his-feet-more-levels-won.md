---
number: 77
title: Touchdown Tommy's throws fell at his feet; more levels won by play
date: 2026-10-15
area: bug, test
files: crates/rugrats/src/world/game/things.rs crates/rugrats/tests/play.rs
started: 2026-10-15T20:22:25-06:00
took: 14m
---

# 77. Touchdown Tommy's throws fell at his feet; more levels won by play

A play test for level 21 found that no throw ever left Tommy's hand. So Touchdown Tommy could not be won.

- **The bug:** the flight's wall test (worklog 59, after 0x80060810) took back any step whose point was not in the open. The bottle leaves the hand with its bottom a little under the field's level floor, so the first step counted as a wall: horizontal speed 0, and the bottle dropped where it was. The catcher missed it, and Angelica's lunge took the round.
- **The fix:** blocked is now what the original names, a step into a wall or a higher column. That means out of the field, no tile, or the new column's floor above the current one. Level ground under a low thing no longer stops it. Level 6's ledge (a higher column) still stops a box.
- **Won by play in tests/play.rs** (each with the player put in place where needed; the levels' rules decide):
  - level_5_outer_space_is_won: room 30's cine 38 (the float), then the garage, room 38, cine 40.
  - level_6_toy_palace_is_won: room 11's cine 46 hands over to Tommy, the four boxes stacked to the platform, Triangle at the switch: cine 47.
  - levels_20_and_21_are_won: a policy that eats (Circle) while Angelica is away and, when she comes near, faces the baby farthest from her and lobs (Triangle). Cine 64 wins No More Cookies and cine 50 Touchdown Tommy, the latter only after the fix.
- **New helpers in the tests:** enter_room (walk into a room, the cinematic it plays) and play (a policy choosing buttons each frame).

**Still unknown:**
- The original's wall test against the room geometry (0x80060810 with 0x8002cd7c) is still approximated by the COL columns.
