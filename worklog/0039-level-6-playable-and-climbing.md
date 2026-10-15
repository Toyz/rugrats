---
number: 39
title: Level 6 playable, and climbing
date: 2026-10-15
area: play
files: crates/rugrats/src/level6.rs, crates/rugrats/src/player.rs, crates/rugrats/src/game.rs, docs/engine/level-06.md
started: 2026-10-15T06:27:17-06:00
took: 41m
---

# 39. Level 6 playable, and climbing

A ninth delegated map, docs/engine/level-06.md, traced Toy Palace and
checked it in the oracle. It also found why the port showed cine 46 at
level start: with Tommy as the player he lands on LP 15 in room 11,
whose cine is 46. Chuckie is level 6's player (0x80018598, also 10).

**Climbing** (state 6, 0x8005f310), for every level:

- Walking into a ledge or a climbable object (template flag 64, now on
  `Box3`) brings the player onto it.
- The ledge may be up to 1033 above the feet, 521 while carrying
  (0x80056e44's types 9-14, 0x80058da4).
- A COL ledge is found by asking the field at the feet less the reach.
  An object's box by its top.

**Level 6** (`rugrats::level6`):

- Chuckie starts where cine 45 leaves him; Tommy is hidden.
- Guards from EpisodeStart, all mode 11 with reach 1024 and bite waits
  as seen:
  - 46: range 5000, bite 15;
  - 506: range 15000, bite 6;
  - clowns 920-922: bite 8.
- Scary toys (templates 1227, 1028-1031 for 5 and 1155 for 2) hurt once
  per approach within their trigger radius; leaving clears it.
- Bursts 969 (every 50 frames, 18 within 3072) and 982 (25, 2048).
- Standing on a 1102 toy breaks it: sound 173, thrown up 2000, 5.
- Room 2 is phase 2. Cine 46's end swaps the bodies: Tommy is the player
  at (40738, -1267, 8097), Chuckie a character beside him. Tries 3,
  song 2.
- The switch's cine 47 wins.

Checked in scripted runs: a scare for 5 near the start, a burst for 18
in room 4, the swap after room 11's cine, Chuckie climbing onto a Reptar
box (512 up). Level 2 and 3 runs unchanged.

**Still unknown:**

- The scared animations (Chuckie's slots 0-4), the spinning toys and
  the walking robot toy (class 15 on posts 17-18).
- Climbing's refusals (levels 1 and 12 room 4, level 10 type 14) and
  the climb animation's motion; guard 46's stagger.
- The piano keys' notes, the ambience by character.
