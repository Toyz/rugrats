---
number: 13
title: Animation: quaternion keys by channel, pinned to the original's matrix
date: 2026-10-14
area: format, render, decomp, test
files: crates/rugrats-data/src/anim.rs, crates/rugrats-render/src/models.rs, crates/rugrats-render/tests/against_original.rs, crates/rugrats-render/src/mesh.rs, crates/rugrats-viewer/src/main.rs
started: 2026-10-14T20:21:12-07:00
took: 4m
---

# 13. Animation: quaternion keys by channel, pinned to the original's matrix

A database's animations are three files. ANM indexes them (first frame,
frame count; 0xffff none), FRM lists frames (a duration the loader
multiplies by 5, and a range of keys), ROT holds the keys: two words
packing a quaternion as four signed fields (14, 14, 14 and 13 bits) and a
5-bit channel, or, with bit 29 set, a lift that moves the object up or
down. The hierarchy maps channels to parts through the bytes at its +0x14.
This came from the two samplers, `anim_set_frame` (0x80055170) and
`anim_blend` (0x800552bc), and `UpdatePartRotations` (0x8004a5dc, named by
its own error message), which builds each part's matrix from its
quaternion and its parent's.

`rugrats-viewer 0 --anim 1,239` poses DB00's Tommy at each of the 50 key
frames of animation 239, a walk. The first convention tried was the
textbook one; Tommy walked convincingly with it, which is why it was not
trusted. `quaternion_matrices_match_the_original` calls the original's
quaternion-to-matrix (0x8001fc48) in rrt-emu on five quaternions and
compares: the game's matrix is the transpose of the textbook's, and the
port now uses it (all within 8/4096).

On the way: DB00A.RAW is 262144 bytes, not 294912, though load_rooms
uploads 576 x 256 for every database; the port's VRAM load now stops where
the data ends. And DB22's FRM and ROT are absent from the file table, so
the opening cinematic must pose its characters from its stream, not from
animations.

**Still unknown:**
- the frame duration's unit and run-down rate
- which animations belong to which character
- what fills VRAM past DB00A.RAW's end in the original
