---
number: 43
title: Slopes read as walls; level 11's rings 2 and 3
date: 2026-10-15
area: bug
files: crates/rugrats/src/player.rs, crates/rugrats/src/game.rs, crates/rugrats/src/level11.rs, crates/rugrats/examples/play_script.rs, crates/rugrats/examples/col_probe.rs
started: 2026-10-15T08:45:47-06:00
took: 5m
---

# 43. Slopes read as walls; level 11's rings 2 and 3

Scripting level 11's second ring found two bugs that every level had.

**Slopes read as walls.** The player's floor test asked the height field
one unit above the feet. On any rise the point was already under the
next floor, so the field answered "inside" and the move went back. Flat
floors hid it. The tricycle stopped at the ramp's foot (col_probe showed
a smooth slope, 1408 to -561 over 8600). The test is now asked from a
step (384) above the feet while on the ground: a slope or a step within
it is floor, a taller rise still a wall.

**Template flag 8 collided.** Flag 8 objects are skipped by the object
collision tests (0x80063d3c, 0x80065210 ..), as level 3's doc noted for
the pie toss. The take-off zone (template 878) and level 10's playpen are
flag 8 volumes; their boxes no longer stop the player.

**Level 11 checked further:**

- The stick's run and walk factors are now per player: Spike 20 and 10,
  the tricycle 20 (seen top speed).
- Ring 2's jump: the oracle landed at 26977, 32671 and 38690 for bands
  4, 10 and 16. That fits the ground speed carried on plus 12 x speed x
  the band (factor 12 at this speed), not 40 x speed x band. With it,
  Cross 36 or 40 frames up the ramp lands in the net and asks for cine
  57; 25, 32 and 44 miss (cine 56).
- Ring 3: readings 192 and 172 launch Chuckie at heading 3072, forward
  2816. He lands at x 4252 (the oracle 4056) and cine 62 follows.
- `Script::skip_to` and play_script's PHASE=n set a level up at a phase
  for tools; BOXAT and GOODIES list boxes and collectibles.

Level 2, 3, golf and level 6 runs give what they gave before.

**Still unknown:**

- The real floor test (0x800627a0) and the move test (0x80064a34).
- The jump's true velocity for state 26 (fitted here).
