---
number: 157
title: Levels 9 and 10 walked: characters gone up to by stick
date: 2026-10-16
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-16T09:45:42-06:00
took: 8m
---

# 157. Levels 9 and 10 walked: characters gone up to by stick

The walking campaign (`levels_walked`) still fell back to putting the
player in level 9 (twice) and level 10 (once). Both were the tests' own
targets, not the walk: a place 500 past the character in z, at the
character's own y (a body's y, 253 above the floor), on the far side of
it, so the last straight stride ran into the character's body.

Level 9 is now played as Mr. Friend asks: he comes after Tommy, so the
nearest is steered at with the stick and kicked (Triangle within 1100 of
him, every fourth frame) once he is within 1000; one far off (46 waits at
home after cine 18) is gone up to first. Level 10's babies are gone up to
the same way, then tagged.

`up_to(g, id, reach)` goes up to a character as a player does: the plan
walks to any place within reach of it on its floor (not a place picked
beside it, which may sit behind a shelf: baby 608), then the stick steers
at it while it moves; twice. `beside` gives the floor place a distance
short of a thing on the player's side. Under WALKLOG, a last stride that
falls short says where it stopped.

Left walked by putting: level 5's elevator and float (room 30, room 38 and
the float's two places) and level 6's box 0 at (46657, -3313, -28502).

**Still unknown:**
- Whether level 5's float can be planned like the ground (it moves the
  body by its own rules) or needs its own steering.
- Why the running jumps by level 6's guard fall short of box 0's shelf.
