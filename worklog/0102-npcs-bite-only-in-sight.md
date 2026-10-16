---
number: 102
title: NPCs bite only in sight
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/npc.rs
started: 2026-10-15T22:49:27-06:00
took: 1m
---

# 102. NPCs bite only in sight

In mode 4 (follow) 0x80084084 tests the way from the NPC to its target
with the segment test (0x8002cd7c, ported in 99) before anything else.
Only when it gets through (returns 0) does it go on to the attacks:

- within +0x68c with +0x688 bit 128: the lunge (mode 8, state 28);
- within half of it, with the heights overlapping (0x80083ff0) or
  touching: stop and, with bit 8, bite (mode 5).

A wall or floor between them (return 1), or an NPC not in the open
(return 2), skips the attacks for that frame. The port's follow mode bit
whenever it was close. Now the bite asks `collide::segment(npc,
target) == 0` too. The level 2, 9 and 16 bite tests still pass.

**Still unknown:**
- the lunge (mode 8 from this test) is only ported per level (levels 20-21's Angelica); other followers with +0x688 bit 128 do not lunge
