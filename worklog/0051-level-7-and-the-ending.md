---
number: 51
title: Level 7 and the ending
date: 2026-10-15
area: play
files: crates/rugrats/src/level7.rs, crates/rugrats/src/game.rs, crates/rugrats/src/main.rs, crates/rugrats/src/progress.rs, docs/levels/level-07.md
started: 2026-10-15T12:45:49-06:00
took: 58m
---

# 51. Level 7 and the ending

A delegated map, docs/levels/level-07.md, traced Reptar 2010 and the end
of the game, walking the whole route to City Hall in the oracle.

**Level 7** (`rugrats::level7`):

- The player is Reptar (char 9, 0x80018598): gait 10 and 10. He starts
  where cine 79 leaves him, (25162, -703, -9351).
- Buildings (template class 14, 96 in DB07) are kept apart from the
  scenery. When Reptar's body comes within reach of a model box they
  break (0x8004d7c4):
  - the model becomes shape + 1, the template t + 1;
  - the rubble still blocks unless its template has flag 8;
  - 1240 goes;
  - sounds by template, then 91 or 92.
- The seen reach was 1670 to 2240 from his origin; the port uses 1700
  past the boxes.
- Triangle chomps (animation 49) and Circle breathes fire (51), sound
  89, in any state. A landing plays sound 93.
- Templates 401 and 795 are taken by touch.
- Level 7 keeps the camera's -1000 for every room switch (0x80029288).

**The end:**

- Room 6's cine 85 "Reptar Dance": exit -1, nothing recorded.
- The teardown (0x80018264):
  - clears +0x22, +0x23, +0x30..+0x46 and both bar arrays;
  - plays movie 6, CREDITS.STR, which any button skips. Screens.md's
    "11" was the display mode.
- +0x30 = 4, and the game goes on in the hub at level 7's spot, with no
  Reptar Puzzle. The port now does all of this; it used to quit to the
  title.

Checked: Reptar starting in room 9 and running north, two lamps and two
more objects breaking on the way.

**Still unknown:**

- The real hit test (0x8006b6d0: one building broke 2400 away while he
  stood).
- The flames (they broke nothing in the oracle); the debris and
  explosions; the camera shake.
