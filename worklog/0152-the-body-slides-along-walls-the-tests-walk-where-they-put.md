---
number: 152
title: The body slides along walls; the tests walk where they put the player
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/mirrorland.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T04:13:46-06:00
took: 4h 7m
---

# 152. The body slides along walls; the tests walk where they put the player

Port fixes, each found by playing the levels with the stick:

- Walls (0x80066014): the ground step is tested at three points ahead of
  the body, its reach past the step straight on and its half width to
  each side, at a step's height. A wall there turns the whole step along
  whichever axis is open, at the full speed (the slides 0x80061a64 /
  0x80061b28); a ledge low enough to climb is left to the climb. Before,
  the step was a point test and a step into a wall was undone, so a run
  along a wall at an angle stuck. Oracle runs (`running_into_a_wall_
  slides_as_the_original`, `spike_stops_short_of_a_wall_as_the_original`):
  Tommy at 45 degrees into the hub's wall at x 12288, either side, holds x
  11924 and slides 170 a frame; straight at it he stops at 12020; Spike in
  level 11 stops 479 short at 200 a frame. The reach both ways is the
  hierarchy box's x half width (header +4/+0xc: Tommy 250, Spike 315); the
  box's z (Spike's 1000) is not it, though 0x80066014 reads +8/+0x10 for
  the reach.
- In the air across a place with no floor record: only the across part is
  undone and the fall goes on (0x8005f1b4 moves across, then down). The
  whole step was undone, so the body could hang in the air for ever
  (level 12's stairs, level 3's aisles).
- The jump's first air step (in the root-key code) met no wall, so a jump
  taken beside a wall put the body inside it, where it could never land.
- Mirrorland's balloons were in the world from the start, so one taken
  before its box was opened never counted and its colour could not be
  finished; they now wait out of the world until the box places them
  (0x8007ccb4 takes them out of the loader's lists).
- `Game::hide` drops the object's boxes from this frame's lists at once;
  `Game::bodies` keeps every box the player met this frame.

Tests: `walk_to` plans the way (`Ground`: cheapest-first over 256 cells,
steps, climbs, hops onto tops, running jumps along the body's own
`jump_shape`, the body's width against the COL, doors, things and
characters as they stand, a 1024 grid of the boxes) and plays it with the
stick, Square, R1 and Cross, planning again where a move failed in play
(banned) and using a shut door it comes to. With `levels_walked` (ignored,
by hand) every `put` and `enter_room` walks. Level tests changed to play as
a player would: level 3 takes the balloon; level 6 carries five boxes up
the steps; level 10 races the babies on Chuckie's floor first and plays
on after a lost race; level 11 eats the bones with running jumps through
them; level 4 follows Cynthia to her newest spot; Mirrorland collects
nearest first; `face` and the aims are the stick's. The campaign test
plays every cinematic it came to (36, 15952 frames) to its end.

**Still unknown:**
- Why 0x80066014's reach reads the box's z (+8/+0x10) when Spike's stop
  fits its x; whether the side probes are x too.
- The original's case table for one side probe alone (read as asymmetric,
  seen symmetric in the oracle).
- How the slide follows an angled wall (the port turns along an axis).
