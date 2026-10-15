---
number: 50
title: Level 18 playable: Egg Hunt
date: 2026-10-15
area: play
files: crates/rugrats/src/level18.rs, crates/rugrats/src/main.rs, crates/rugrats/src/game.rs, docs/levels/level-18.md
started: 2026-10-15T12:23:17-06:00
took: 23m
---

# 50. Level 18 playable: Egg Hunt

A delegated map, docs/levels/level-18.md, traced Egg Hunt and checked it
in the oracle. The port now plays it.

**Level 18** (`rugrats::level18`):

- EpisodeStart (0x80079670): 600 and 99 hidden, 170 s, song 0. The
  loader has put 35 eggs (template 496) on group 0's spots and the gold
  egg (1219) behind the couch.
- Each egg counts down by icon 143; the 35th ends the clock next tick
  (0x80051b50). The gold egg shows icon 245 and earns the fourth bar.
- The clock's end: all eggs plays cine 24, won (3 bars, 4 with the gold
  egg). Eggs left play cine 23 after the one try is used without a cry.
  The hub follows with the level recorded played. Cine 23 is now in the
  endings table, lost.

**Also:**

- The "already earned the reptar bars" note now comes after the start
  cinematic and its card, as the oracle showed for 13 and 18. The
  puzzle-piece note still comes first.
- No hint cards in activity play.
- In activity play, the gold egg is not placed (0x80045fa0).

Checked: the clock running out at frame 5099 asks for cine 23.

**Still unknown:**

- What hurts Tommy here (Damage is on).
- The camera kinds 7 and 9 in rooms 16 and 15.
