---
number: 57
title: Doors: class 8 swinging
date: 2026-10-15
area: play
files: crates/rugrats/src/game.rs
started: 2026-10-15T15:35:33-06:00
took: 1m
---

# 57. Doors: class 8 swinging

Class 8 doors are now Doors with a Swing, from docs/engine/doors.md "Class 8".

- **Template fields:** ease +0x18, waits from flags 0x10000000 (16, shut) and 0x20000000 (8, open), trigger radius +0x34, damage +0x32. The first target is the open end.
- **Running:** they think only while the player is within the radius (2D from the placement), matching the enter/leave scheduling.
- **Each think:**
  - A wait counts down by 136.
  - Then the speed is set to 256 + ease x rsin(((angle - shut) << 11) / (open - shut)) / 4096, and the hinged step and stop apply.
  - At an end the target flips, and the door waits 4096 if its flags name that end.
- **Blocked:** Damage +0x6d to the player at most every 25 frames (51 VSyncs), but not in the hub. Level 3's 159/160/595/602 hurt 15.
- **No Triangle use.**

Checked by watching DB6 (990/991) and DB3 (159/160/595/602) swing open and shut while standing near; no panics in levels 2, 3, 6.

**Still unknown:**
- Not checked against the oracle: the speeds and waits are computed from the code only.
- DB2's 672/1216 pairs (no end sound 54) were not watched.
