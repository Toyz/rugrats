---
number: 237
title: The flights' second mark from the stop test, and npc_check's put on the floor
date: 2026-10-22
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/examples/npc_check.rs, docs/engine/gameplay.md
started: 2026-10-22T21:31:40-06:00
took: 35m
---

# 237. The flights' second mark from the stop test, and npc_check's put on the floor

The flights' marks are now the original's mark for mark, and npc_check's put no longer leaves its own artifacts.

**Two marks a hit.** The original made two marks where the port made one, about 57 apart. It is not the room walk: 0x80086668 walks rooms only for the objects and tests the world once. It is the stop test (0x80087414). That checks for a floor within the flight's radius under its backed-off place, through 0x80086668 again, which marks the floor it finds. The port's stop test now reports that floor hit too, and the flight marks it. npc_check MARKDBG=1 by level 3's drops shows the same eight lasting marks, in the same order, at the same places.

**npc_check's put** now puts both players:
- on the floor under the given place (PUTAIR=1 keeps the height);
- with the body's matrix (+0x04) and the speed cleared, as the oracle tests' put.

The port's player no longer fell and landed (the stray sound 11), and the camera stays with the original's. In level 3 by the 614 source the voices' turns are then the same on all 149 frames, and in levels 1, 3 and 4 by their emitters every flight and sound is the original's.

**Still unknown:**
- The marks' face bits (0xc080be00 on a wall, 0xc0802e00 on a floor, by the normal's y) and 0x8008e508's drawing; the port lays them as the pies'.
