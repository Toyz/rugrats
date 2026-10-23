---
number: 239
title: A launch's room, and the particles' object touches
date: 2026-10-22
area: play
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/mod.rs, docs/engine/gameplay.md
started: 2026-10-22T22:46:17-06:00
took: 33m
---

# 239. A launch's room, and the particles' object touches

Two more of the flights' "not here" are now done.

**A launch's room.** 0x8008864c begins by looking up the room holding the start (0x8002b660 with no hint). With none it makes no flight; else it keeps the start inside that room's box (+0x10, +0x20) and the flight remembers the room (+0x38). `Flights` now has the rooms' boxes (set at a level's load) and does the same.

**A flight's touch.** A flight of flag 0x40000 meeting an object runs the object's touch (+0x58), as a thrown thing's does: its parts (`touch_parts`, touches 1-7), and touch 1's sound off the shared count. The particles' object hits (`Flights::struck`) now carry the object's place and the point met for it.

The emitter places of levels 3, 5, 6, 16 and 29 are still all the original's (only the frames before the sync differ).

**Still unknown:**
- A flight's stick (+0x98 = the object met, unless a character): what reads it.
- The flight quads' screen-edge cull and their split into four (0x8008a954, face flag 0x8000000).
