---
number: 235
title: Doors scheduled by their triggers as the original's, level 26's roar synced; the sweep's sounds and flights all the original's in 24 levels
date: 2026-10-19
area: engine
files: crates/rugrats/src/world/game/doors.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/golf.rs, crates/rugrats/examples/npc_check.rs, docs/engine/doors.md
started: 2026-10-19T00:25:55-06:00
took: 1h 15m
---

# 235. Doors scheduled by their triggers as the original's, level 26's roar synced; the sweep's sounds and flights all the original's in 24 levels

Doors now run by their trigger schedules, level 26's roar keeps the original's time, and the npc_check sync is wider. With that, the oracle sweep at SYNCAT=1 has no sound or flight differences left in 24 of the 28 levels it runs.

**Doors.** The original schedules a door's think in two ways:
- **Class 8** (0x80049e18 in, 0x80049e5c out, +0x9c): scheduled while its trigger entry is in. So it starts when the five-a-frame pass reaches it. The port had polled the player's distance; level 3's hinged 160 then started eight frames early and closed (sound 54) early.
- **An automatic door** (template flag 0x40000000, golf's 770: enter 0x8004f6d8, +0xcc): scheduled on its enter, which leaves the "in" bit clear. So the pass enters it again each time round and never leaves it: it cycles for good once the player has come near.

`Game::door_entered` does both from the trigger pass; a door off the list keeps the distance test. Level 24's 770 sounds 136 and 135 on its cycle exactly as the original does (doors.md had reckoned 136 never plays).

**Level 26's roar** (0x80085168, 0x800c4ce4) was right but never synced; `Script::set_countdown` lets tools set the count (golf's roar, the hub's barks, level 10's lines).

**npc_check:**
- doors are matched by the nearest object of their template (+0xc0 is not every door's start);
- a door's `moving` is synced from +0xcc or +0x9c;
- the level's countdown is synced.

**The sweep** (FLIGHTS, SOUNDS, SYNCAT=1, 300 frames):
- 0 flights and 0 sounds off in levels 1-7, 10-18, 23-31.
- Levels 20 and 21 want CINES=1 (the round starts on cine 63's end): level 20 then matches.

**Still unknown:**
- Level 21: Angelica heads 20-25 units off the original from the first frame (she aims at the box in the player's hand; the hand's place there?), so her lunge (sound 121) comes 6 frames early.
- Level 9 under SYNCAT=1 (its own sync, level-09.md).
- The sound sources (0x8006fe04, voices 15-16, kept on and turned by distance each frame, 0x8006f88c): they need voice-addressed key on, volume and key off from retro_rt's SPU, which is waiting on approval to change retro_rt.
- The port-only sound after npc_check's put (the tool does not settle its player).
