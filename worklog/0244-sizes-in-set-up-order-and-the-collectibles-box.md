---
number: 244
title: Sizes in set-up order, and the collectibles' box
date: 2026-10-23
area: play
files: crates/rugrats/src/world/fixups.rs, crates/rugrats/src/media/cine.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-23T03:28:39-06:00
took: 43m
---

# 244. Sizes in set-up order, and the collectibles' box

Worklog 243's leftover objects are resolved: an object's size is taken at its own set-up, before the set-up's changes to shared model boxes. The port now keeps each placement's size as the original's set-up order leaves it.

**Found.** A hook on 0x80046570 during the boot (stack_check SETUP=template, `Play::boot_with`) showed level 6's first spinner 1028 sized with its model's box at y ±557. The next two were sized with -1000. A stop on the write (RUGRATS_STOPW at the box) found the writer: 0x8004691c's template cases at 0x80047890 set 1028's first box top to -1000, after that object's own 0x80046570. Each object of the model set up after it is sized with the change, each one before it keeps the old size. The port already makes these box changes (`fix_up`: 1028, 883, 381, 248 and the rest), but sized every object after all of them.

**The collectibles.** Their set-up (0x80048a44) makes the first box of their model the cube of 375 for every kind but 0 and 5 (kind 12 takes model 0 and changes that), then sizes them again: 649. Other objects sharing the model and set up after take 649 too: level 12's objects on model 0. The port never made this change, so collectibles' models kept their boxes.

**The port:**
- `fix_up` makes the collectibles' change in LP order.
- It records each placement's size before that placement's own changes (`Set::sizes`); a copy takes its source's.
- An object's boxes carry their size (`Obj::size`), which the solids' bound and size take.
- The nearby and focus tests take the placement's size (`Game::size_of`).

**Checked.** stack_check OBJBOUNDS=1 matches each object to its placement by place and template. Every object of levels 1, 3, 6, 12 and 16 is the original's, but:
- the characters (1773 cube, +0x86 8192, set after);
- level 3's two 623s. Their record is 100's with 623's flags (worklog's fix_up rename), so the tool finds no placement of 623 and compares with 623's model; the port's own size is 100's, the original's 886.

Every test file, and the hle suite.

**Still unknown:**
- Whether a collectible's ±375 box matters to anything but the sizes: the port's collectibles are taken by distance (0x800514b0, the model's radius plus the player's).
