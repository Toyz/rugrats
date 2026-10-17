---
number: 205
title: A pie's hit flashes its mark for a frame as the original's does; COL tile byte +5 is the shade
date: 2026-10-17
area: render
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats-data/src/col.rs
started: 2026-10-17T17:51:51-06:00
took: 7m
---

# 205. A pie's hit flashes its mark for a frame as the original's does; COL tile byte +5 is the shade

A pie leaves the original's mark where it hits, and COL's tile byte +5 has a name.

**The mark (0x8008652c).** A flight whose record has +0x2a set (the pies' 284, from 0x80061c98's desc +0x46) calls 0x8008652c at a hit:
- It sounds 46 at the player (worklog 201).
- It turns a sprite to the face met (0x8008e6c8): angles (x, y) = (-asin(-ny), ratan2(-nx, -nz)) of the normal.
- It adds the sprite to the flat-sprite pool (0x8008e76c, or 0x8008e72c with +0x8a): sprite 284, a square of half side 900 (+0x8c, desc +0x68) at the point met.
- Its grey is the COL tile's byte +5 there, or 128 when what it met was no tile (0x800869cc).

The pool's entry has +0x08 0 (0x8008e240), and the pool's frame (0x8008e670) frees any entry without +0x08 bit 1 after drawing it (0x8008e694). So the mark is a flash: drawn the frame of the hit, and gone. `Game::marks`, made in `fly` and drawn with the flights through the sprite quad (`Sprites::quad`, as golf's aim line, 0x8008e508).

**COL tile +5** is the shade: what a body's brightness (+0x3a) eases toward from the record under it (+0x8c +5), and a mark's grey. `rugrats-data::col` names it.

**Still unknown:**
- What sets an entry's +0x08 bit 1 (an entry that stays): the golf aim line is made every frame, so it needs none; no persistent user is known.
- Tile byte +4 and +7.
