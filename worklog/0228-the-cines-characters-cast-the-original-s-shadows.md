---
number: 228
title: The cines' characters cast the original's shadows
date: 2026-10-18
area: render
files: crates/rugrats/src/media/cine.rs, crates/rugrats/src/world/game/shadow.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T13:26:54-06:00
took: 11m
---

# 228. The cines' characters cast the original's shadows

The in-engine cines' characters now cast shadows as the original's do. All 3473 shadow quads of level 1's start cine match the original's.

**In the original.** play_cine (0x8007424c) draws through the same room render (0x80030b80), and the characters through their own draw, so the shadow (0x8003414c) is drawn there too. Only the opening cine's init (0x8008f078) turns shadows off (0x800c3fc4), until its end (0x8008f308). Booted with cinematics, level 1's cine 7 makes 3473 shadow calls with the switch on.

**The floor in a cine.** It is the one under where the stream puts the character, not the one under its placement. Characters 15, 16 and 1776 are placed upstairs (floors -1279, -1490, -876), but the original draws their shadows at the hall's 1024, where cine 7 moves them. So the cine's mover refreshes +0x13c. No object top was met in this cine.

**The port.**
- `shadow::quad` and `Quad::triangles` are free of `Game` now, and `world::game::shadow` is public.
- `media::cine::Shadows` holds the GTE tables, the database's COL and TEXPOS 261.
- `Set::shadow_quads` poses each character from the frame's keys with the port's integer posing (`pose::posed`). It finds the floor under the posed place and builds the quad. `Set::shadows` draws them, and `Cine::tick` adds them to the frame, but not in the opening (cine 0).

**Oracle.** `level_1_cine_shadows_as_the_original` boots level 1 with its cinematics and reads every quad at 0x80034518, keyed by the cine (0x8007424c's argument), its frame (0x800c4c34) and the object's id (+0x38). It compares them with the port's from the same stream frame: 3473 the same, none off.

**Still unknown:**
- Which call in the cine's mover refreshes +0x13c (the floor query or the settling).
- Whether any cine puts a character over an object, whose top 0x8006c118 would take (the port's cines have no object solids).
- Which objects lie on a room's second list with +0x00 0x4000, against the port's "no character".
- Who reads floor flag 8.
