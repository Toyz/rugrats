---
number: 243
title: An object's size and bound as 0x80046570 makes them
date: 2026-10-23
area: play
files: crates/rugrats/src/world/boxhit.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-23T02:46:11-06:00
took: 42m
---

# 243. An object's size and bound as 0x80046570 makes them

An object's size (+0x86) and bound cube (+0x78..+0x84) are now computed as the original's 0x80046570, not taken from the model's radius.

**0x80046570** (`boxhit::bound_size`). Over the object's model boxes (+0x88's list) it takes the least corner and the greatest corner, each axis over all the boxes. +0x86 is the floor square root (0x8001f4ec) of the larger of their two squared lengths. With no boxes the bound is the cube of 1024's (1773: the characters' cube; their +0x86 of 8192 is set after, by 0x8004822c, 0x800596e8 and 0x80081bac). +0x7e is the same across x and z alone. Then the bound is the cube of +0x86 about the object's place.

0x8004691c, every object's set-up, calls it. The collectibles' set-up (0x80048a44) first writes ±375 into their model's shared box list, which gives their 649.

**Checked** with stack_check OBJBOUNDS=1, every object of each level's room lists against its template's model's `bound_size`. Characters, collectibles and the few below are left out:
- level 1: 238 of 256 the same;
- level 3: 273 of 311;
- level 6: 155 of 178;
- level 12: 205 of 240.

**Where the port used the model's radius for these, it now uses `bound_size`:**
- this frame's solids' bound and size (0x8006a714's screen);
- the focus test (0x8006af4c);
- a thing in flight as the characters' tests see it;
- the nearby test for things, doors and placed objects.

For things the two agree, so worklog 242's things are unchanged.

**Left as the model's radius:**
- the draw's cull (0x80031b84);
- the particle and touch radii, which are the model's own.

**Checked:** every test file, and the hle suite.

**Still unknown:**
- A few objects hold a smaller +0x86 than their set-up gives, by room, though only 0x80046570 writes the cube:
  - level 3's 883: 5930 in rooms 0 and 35, 6172 in rooms 9 and 18;
  - level 6's spinner 1028: 1167 in room 14, 1432 in room 1;
  - level 1's 248 and 1130, level 3's 623, level 12's 960 and 1070.

  The smaller ones are the length of the box's greatest corner alone (1167 for 1028's (718, 557, 733)), as if those objects were set up when their box's least corner was different. The port gives them all their set-up value.
- The level 12 objects of template 0 (649) are collectibles' cubes on objects stack_check reads as template 0.
