---
number: 124
title: More sounds where they happen
date: 2026-10-16
area: audio
files: crates/rugrats/src/world/game/doors.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/src/levels/mr_friend.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/src/levels/aisle_seven.rs, crates/rugrats/src/levels/cynthia.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/src/levels/reptar_2010.rs
started: 2026-10-16T00:52:14-06:00
took: 3m
---

# 124. More sounds where they happen

A survey of the port's unplaced `play` calls against the original. For
each sound number, it checks whether the original calls 0x8006f644 (at a
place) or 0x8006f26c (unplaced). That is the literal-argument calls,
plus the `a0 = N` loads before a call.

**Placed in the original, played centred in the port; now placed:**

| sound | where | at |
|---|---|---|
| 54 | a hinged door's close, 0x8004f868 | the door |
| 36 | the puck's kick, 0x8004d1a8 (template 438) | the puck |
| 100 | Mr. Friend's end, 0x80084084, level 9 | him |
| 52 | a bite, 0x80084760 (levels 3 and 16) | the biter |
| 168 | a bite, 0x80084760 (level 4) | the biter |
| 148 | the swat, 0x800857fc | the swatter |
| 88, 91 | what Reptar takes, 0x8005ac4c | the thing |
| 43 + rand & 15 | box toss's crunch, 0x80057950 via 0x8006f2fc | the player |

The crunch uses program 14 (13 in level 20). Box toss's other eat sound,
123, stays unplaced.

**Unplaced in the original too, so unchanged:**

- doors 14, 15, 170, 135, 136, 16, 17;
- collectibles 28-31 and 113, and 69 and 109 (0x800514b0);
- the goose's 34, the circus's 124 and 125;
- golf's 118-120, Reptar's 89, level 12's 161, the hub's 102.

**Still unknown:**

- Mr. Friend's hit sound 19 and the goose's honks (32 + rand & 3): their
  calls take the number from a register that was not traced.
- The crunch's program (14, or 13 in level 20) against the port's effect
  table: the port plays effect 43 + n through its usual mapping.
