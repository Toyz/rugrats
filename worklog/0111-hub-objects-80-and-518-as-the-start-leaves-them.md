---
number: 111
title: Hub: objects 80 and 518 as the start leaves them
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/hub.rs crates/rugrats/src/world/game/mod.rs
started: 2026-10-15T23:19:45-06:00
took: 2m
---

# 111. Hub: objects 80 and 518 as the start leaves them

Two of 54.2's leftovers, from docs/levels/level-01.md:

- Cine 7's end hook (0x800732d4 case 7, also run without the cine by
  0x800746bc) sets object 80's y to 746. Object 80 is template 896, the
  door to level 6, LP 69 at y 748.
- EpisodeStart (0x80078e7c) makes object 518 (LP 248, template 1149)
  template 1130 with that template's model, room 3. It stands outside
  room 3 and was never seen, so this changes nothing on screen.

`Game::edit_placements(id, f)` changes the placements of a registry id
and rebuilds the scenery and its boxes. The hub's start applies both.

**Still unknown:**
- the signs' sparks (an effect flight; with the effects work)
- object 518's room 3 (the port's scenery is not kept by room)
