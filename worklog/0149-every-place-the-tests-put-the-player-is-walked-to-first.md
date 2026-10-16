---
number: 149
title: Every place the tests put the player is walked to first
date: 2026-10-16
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-16T02:44:04-06:00
took: 23m
---

# 149. Every place the tests put the player is walked to first

The level tests put the player where play happens (`put`, `enter_room`).
To show those are places play can reach, `the_tests_places_are_reachable_on_foot`
wins every level with a check on: before each put, a flood from where the
player is (`on_foot`: 256 cells over the COL and the objects' tops, steps
under 384, a climb of 1033 or the moving jump's rise, drops of any height,
blocked by objects' boxes but not doors) must reach the spot; before each
room entered, some floor of the room. Floating (level 5's state 21) uses
`afloat`, a 3D flood of open COL cells, since Cross and Circle move the
body up and down without gravity.

What it found, and the fixes:

- Level 3: room 19 (the cart ride, cine 9) lies behind a ledge 2048 high
  at z -8192..-7424 between rooms 20 and 19, too high to climb. On the
  oracle Tommy put at (45500, 779, -8704) and walked at it stops at
  z -8592, as in the port: the original blocks it too. `win_3` now takes
  the balloon (id 81, room 25, Triangle: cine 8), which is reachable.
- Level 10: Chuckie went back to the playpen at the last baby's height
  (upstairs, in the air). He now returns to his start in the pen.
- Level 12: the refrigerator's door faces -x; `use_at` stood Spike at its
  z - 600, half inside it. `use_from` takes the side; the fridge is used
  from x - 600.

Nothing in the port blocked a route the original allows.

**Still unknown:**
- What the cart ride in room 19 is reached by in the original, if anything
  (HINT09 "Press @ for a fun ride to the bakery" has no caller found).
- The flood is coarse (256 cells, one body point): a gap narrower than the
  body would pass it.
