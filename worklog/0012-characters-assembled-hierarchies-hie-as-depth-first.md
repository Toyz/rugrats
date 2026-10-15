---
number: 12
title: Characters assembled: hierarchies (HIE) as depth-first skeletons
date: 2026-10-14
area: format, render, decomp
files: crates/rugrats-data/src/hierarchy.rs, crates/rugrats-viewer/src/main.rs
started: 2026-10-14T20:19:07-07:00
took: 2m
---

# 12. Characters assembled: hierarchies (HIE) as depth-first skeletons

The characters stand up in the port's viewer (`rugrats-viewer 22 --hier`):
Chuckie, Angelica and the rest of DB22's cast, each assembled from its
parts at rest.

A hierarchy is a part count, 0x32 bytes of byte lists, then 40-byte parts
from +0x34: a depth (1 the root), 16 run-time bytes, the joint's offset
from its parent (s32 x, y, z), and a model index. The parts are depth
first, so a part's parent is the nearest earlier part one level up. Chuckie
(hierarchy 1) is hips, chest, head, two three-part arms from the chest and
two three-part legs from the hips.

The layout came from `draw_character` (0x8003205c): it reads part i's
model at hierarchy + 0x58 + 40 i, which put the parts at +0x34 with the
model as their last word. Read from +0x58 as a first guess, every part had
its neighbour's offset. The format page is `docs/formats/hie.md`.

**Still unknown:**
- the hierarchy header's byte lists
- how 0x8004a5dc turns the animation into each part's matrix
