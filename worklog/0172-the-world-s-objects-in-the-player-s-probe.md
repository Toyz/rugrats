---
number: 172
title: The world's objects in the player's probe
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/boxhit.rs, crates/rugrats/tests/hle.rs
started: 2026-10-16T21:20:53-06:00
took: 31m
---

# 172. The world's objects in the player's probe

The world's objects now meet the player through the original's own
probe (worklog 171) in play, and what was left of the float stepping on
the ground and in the jump is the original's integers. Played against
the original (`tests/hle.rs`), three of the four scripts are now the
same on every frame: the hub run into its wall (29 frames), level 11's
Spike run (11) and level 18's slide along door 213, into the corner and
the bump (70 frames, off by up to 28 before). The run and jump from the
hub's start is the same for its first 27 frames, through the whole
flight, to where the original meets toy 403 in the air.

- **Objects as solids** (`boxes::solids`, `boxhit::Exact`): every
  object's boxes are marked with the object as the original holds it:
  its integer position, its angles (placed ones from the quaternion,
  worklog 171; a door's swing added about y; a spinner's turn; a walker's
  heading; things and movers from their matrix, 0x8002114c), its model
  (radius for the bound and +0x86, boxes) and which box. Each frame they
  become the solids `ground_move` probes with (`ground::Ground`); the
  float boxes of those objects no longer stop the step on the ground,
  only the characters' do, and the air still uses them all.
- **The thing stood on** is carried between frames as its object and
  found again among the frame's solids (+0xf8, +0xfd).
- **A slippery floor** (flag 2) turns walls' ways off them.
- **An object met head-on** (+0xf4) is climbed only if its type says so
  (flag 64; message 29 if not) and is the bump's, its template's damage.
- **The way** (+0x178) is the body's matrix (0x80020d10 of the heading,
  MatrixNormal'd) applied to (0, 0, 4096), a little off unit (4103 long
  at 1430), not (sin, 0, cos) (0x8005c8d8); taken before the frame's
  turn.
- **The slide** scales the tangent to the old way's length (its floor
  square root, 0x8001f4ec, via 0x80076a74) before its VectorNormal; with
  the way's own length that is 171 a frame along the door, as the
  original, where 4096 gave 170.
- **Into the body and back** (0x80075f20, 0x80075ee8) are the
  MatrixNormal'd matrices and ApplyMatrixLV, not sin and cos products.
- **The launch** (0x80060f1c) is the way scaled to the jump's speed (its
  VectorNormal, 0x80076944) turned into the body, plus the rise; set at
  the animation's event, a frame before the port's first air step, so
  with that frame's heading (the run from the hub's start launches at
  (-1, -1000, 679), as the original).

**Still unknown:**
- The air's move across and its tests (0x80066014, 915 instructions,
  with 0x800658cc, 0x80065c5c, 0x800657d0, 0x80065cbc): the original
  meets toy 403 in the air where the port goes over it.
- The floor under after a move with objects (0x800627a0, 0x80065578)
  still runs on the float boxes, and the run's own probe of its step
  (0x800645d8) and the eased step (flag 0x80) are ported but not used in
  play.
- The characters still meet the player as float boxes (their posed
  parts, 0x8006a318, are not made by the port's animation).
- The nearby list's order, and objects whose model is not their
  template's (worklog 171).
