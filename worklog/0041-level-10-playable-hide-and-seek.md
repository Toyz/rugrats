---
number: 41
title: Level 10 playable: hide and seek
date: 2026-10-15
area: play
files: crates/rugrats/src/level10.rs, crates/rugrats/src/npc.rs, crates/rugrats/src/game.rs, docs/engine/level-10.md
started: 2026-10-15T07:33:07-06:00
took: 24m
---

# 41. Level 10 playable: hide and seek

An eleventh delegated map, docs/engine/level-10.md, traced Chuckie's
Glasses and checked it in the oracle. The port now plays the level.

**Level 10** (`rugrats::level10`), hide and seek with Chuckie:

- Cine 44's end hides 13-16 and 600.
- EpisodeStart (0x8007ee54) puts Phil, Lil and Tommy at random free
  group 0 spots (0x8004fe14). Each spot's link is the first post of its
  route home. Post 120 gets flag 32.
- Triangle or Circle beside a hiding baby, none running, tags it
  (0x800831a4). Its spot is freed and it runs its posts at row 1's 11.
- Chuckie in the playpen (template 1032; the oracle's points fit
  1390 x 2070 about (5926, 21212)) catches a running baby: cine 75, 76
  or 77. All three caught plays cine 78, won.
- A baby at post 120 first (GoalPost, 0x8007f150):
  - the phase goes up;
  - cine 41, 42 or 43 plays, then the baby hides again at a new spot;
  - a try goes without the cry, back to the start;
  - the fourth phase is the cry and the last try.

Checked in a scripted run: Phil tagged, running his posts home (55 ..
119, 120) in 919 frames, cine 42, then hidden again elsewhere with two
tries left.

**Still unknown:**

- The playpen's exact test (0x8006abbc, 0x8006a714); the tag reach
  (900 here).
- Chuckie's head without glasses (model 1237 on the head part); the
  hiding and tagged animations (46, 47); the taunts and footsteps.
- The climb limit of 905 in level 10.
