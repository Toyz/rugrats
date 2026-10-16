---
number: 126
title: Characters in no room are not drawn
date: 2026-10-16
area: render
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/rooms.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T01:01:27-06:00
took: 7m
---

# 126. Characters in no room are not drawn

A check for characters drawn in their bind pose (no part has a
rotation), 90 frames into each level, found some. The ANM of their level
has none of their animations:

- the hub's Angelica (id 600);
- level 4's Tommy, Chuckie and Angelica (1492, 1776, 600);
- level 5's char 36 (142);
- level 6's Reptar (507);
- level 7's Tommy, Chuckie, Phil, Lil, Stu and Didi (1492, 1776, 608,
  609, 15, 16).

All are cinematics' actors (the level docs: cines 38, 47 and 48).

The original draws objects only from the object lists of the rooms
(+0x48, +0x4c). An object goes into a list by its room +0x3b, which comes
from 0x8002b660 at load. An object placed in no room is in no list and is
never drawn. The hub's Angelica, at (21733, -706, -13634), and all six of
level 7's are outside every room. The port drew every character wherever
it stood.

`Game::npc_rooms` now keeps each character's room. It starts from the
character's place and is kept up each frame with the portal lookup
(worklog 122), hinted by the last room; the last room found stays when
none is. A character with no room is not drawn.

Left as they are: level 4's three are in room 56, level 5's 142 is in
room 30, and level 6's Reptar is in room 9, the room play goes on in. The
original draws those if their rooms are seen, so they stand in their bind
pose there as well. That has not been compared.

Test: characters_in_no_room_are_not_drawn. The hub's Angelica and level
7's babies have no room; some of the hub's characters do.

**Still unknown:**

- Whether the original updates +0x3b for a character with none, if
  something moves it into a room (the port finds it then).
- How level 6's Reptar (507) looks in room 9 on the original. The oracle
  could not be brought there. Level 4 does not finish loading under the
  probe either.
