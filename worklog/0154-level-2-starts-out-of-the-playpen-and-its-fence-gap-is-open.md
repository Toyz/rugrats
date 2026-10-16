---
number: 154
title: Level 2 starts out of the playpen, and its fence gap is open
date: 2026-10-16
area: play
files: crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T08:25:50-06:00
took: 14m
---

# 154. Level 2 starts out of the playpen, and its fence gap is open

Walking level 2 by play found two things that kept a player from it:

- Cine 1 (the intro) ends with Tommy and Chuckie out of the playpen
  (LP 283, template 261: a floor 155 up, walls 1360 above it) the loader
  puts them in. The port played the cine and left them inside. Seen in
  the oracle with the intro played (the probe's `--ret 0x80074958=0`
  skips it): Tommy at (-5334, -245, 10864) heading 2881, Chuckie at
  (-5562, -354, 11757) heading 1885; a fresh start (no checkpoint) puts
  them there.
- The board in room 43 (id 335, template 1200, an object) is out until
  cine 3 (0x80078bc4 at EpisodeStart, shown by cine 3's start). The port
  looked for it among the characters, never found it, and left it across
  the gap in the fence, the way to room 69. `Game::hide` now does it.

Walk: a step across a thin box between two cells (`Box3::crosses`) is no
step, except out of a box the walk stands in; with no way at all (Spike
come down on the fence rail after its jump) a run on ahead and the plan
again. Level 2 is walked whole.

Eight walks still end in a put: level 5's elevator ride and float, two
boxes in level 6, a place each in levels 9 and 17.

**Still unknown:**
- Whether other levels' start cinematics leave the player elsewhere than
  the loader's spot (the port places them only where a level says).
