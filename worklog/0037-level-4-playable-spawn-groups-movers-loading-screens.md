---
number: 37
title: Level 4 playable, spawn groups, movers, loading screens
date: 2026-10-15
area: play
files: crates/rugrats/src/level4.rs, crates/rugrats/src/game.rs, crates/rugrats/src/animator.rs, crates/rugrats/src/collide.rs, crates/rugrats/src/main.rs, crates/rugrats/src/screens.rs, crates/rugrats/src/progress.rs, crates/rugrats-data/src/placement.rs, crates/rugrats/examples/play_script.rs, crates/rugrats/examples/screen_shot.rs, docs/engine/level-04.md
started: 2026-10-15T04:59:19-06:00
took: 22m
---

# 37. Level 4 playable, spawn groups, movers, loading screens

A seventh delegated map, docs/engine/level-04.md, traced 7 Voyages of
Cynthia and checked it in the oracle. The tracer found the oracle loads
Tommy unless the character byte is poked: main takes it from the
previous level. The port now plays the level.

**Framework:**

- The playable character by level (0x80018598): Spike in 4 and 11. His
  body, his animations, turn 298, the camera aiming 400 up.
- Spawn groups (LP kind 7, 0x800c4bac): x in the template word, then y,
  z, the group, a running threshold (100 ends a group), a post link.
  0x8004fe14: r = (rand & 255) % 100, the first unused spot above r, then
  r = 0, five rounds.
- Collectibles come in LP +0x46 copies. With a group (+0x48, used when
  +0x4a is set) they go to its spots. Copies 0 is one object where it
  stands: DB04's decorations (template 1141, kind 16) have group 0 but
  must not take Cynthia's spots.
- Kinds 4 and 14 heal by their +0x6d (template +0x32): bones 63.
- COL floor flags (record +4); a room hook for scripts (0x8007f498);
  hint cards a level shows (0x80044434).
- Movers (template class 11, 0x8004ee00): per axis a step to the range;
  with template flag 0x10000000 back and forth, else once. They are
  armed by the trigger radius and carry a player standing on top.
- The loading screen before each level, the "already earned" notes, and
  "Returning to Main Menus" on quitting.

**Level 4** (`rugrats::level4`):

- Spike starts on LP 260. Cynthia is hidden as scenery and drawn at a
  group 0 spot. Her LP +0x42 door is taken away (0x80047e30).
- Rooms 26 and 21 set phases 2 and 3 and the respawn. From phase 2,
  water floors (flag 1) cost 5 standing or 3 moving every 36 VSyncs.
- Room 40's script 2: the 130 s clock, card HINT15, the portrait (sprite
  287), phase 4.
- Rats prowl and bite for 3 (sound 168).
- Within 2000 of Cynthia: cine 48, won. The clock out: a cry, a try, the
  clock again and Cynthia at an unused spot.

Checked in scripted runs:

- Cynthia starts at spot 2; dropped into room 40, phase 4 with the clock
  and the portrait.
- Water costs about 8 health a second there; next to Cynthia's spot,
  cine 48.
- The rafts in room 22 run back and forth.

**Still unknown:**

- Spike's fall and other states have no frames in DB04 (bind pose); what
  states 10-13's ids 902/903/901/900 are.
- One-shot movers' trigger (0x8004ed00's bit 256 logic, level 5's
  template 855); movers pushing the player (0x8006baa8).
- The splash particles, the water footstep sound, the ambient speech and
  the running water emitter.
