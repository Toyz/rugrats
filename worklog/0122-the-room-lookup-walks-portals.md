---
number: 122
title: The room lookup walks portals
date: 2026-10-16
area: engine
files: crates/rugrats/src/media/cine.rs, crates/rugrats/src/world/game/rooms.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T00:34:40-06:00
took: 4m
---

# 122. The room lookup walks portals

0x8002b660(pos, hint) finds the room a point is in, in three steps:

1. The hint room, if its bounds (+0x10, +0x20) hold the point.
2. Otherwise each of the hint's portals (+0x30 quads at +0x34, 48 bytes
   each). A portal whose corner 0 flags (+0x06) have bit 2 is skipped.
   For the others, the room it leads to is corner 1's flags (+0x0e); if
   that room holds the point, it is the answer.
3. Otherwise the first room whose bounds hold the point, else -1.

The port did steps 1 and 3. `Set::neighbours` now keeps each room's portal
rooms; LEVEL*.BIN's quads are dealt to the rooms in file order, as the
loader does. `Game::room_from` does all three steps, and `room_change`
uses it with the current room as the hint.

**Checking the portal data.** Every link in levels 0-31 is in range and
symmetric: if a leads to b, b leads to a. That confirms the reading of
+0x0e. The rooms' bounds overlap in levels 2 (19 pairs), 7 (13), 4, 11, 5,
3, 16 and 31. In those places the first-box answer and the portal answer
can differ.

**Sign 111.** Last entry's open question was the hub sign at (-3394, 932,
8498) that is in no room. It is more than 1500 outside every room's
bounds: a leftover placement. Level 2's real signs are 572 and 1225, in
room 21.

Test: rooms_are_found_through_portals takes each overlap of two rooms in
level 2. From each portal neighbour of the later room, the lookup gives
that neighbour's first portal room holding the point. The cases where
this differs from the first box are counted, and there are some.

**Still unknown:**

- Objects and NPCs still take the first box (`room_of`). The original
  keeps each object's +0x3b with its own hint.
