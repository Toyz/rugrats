---
number: 196
title: Characters run only when scheduled, by script or trigger
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/hub.rs, crates/rugrats/src/levels/mr_friend.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/src/levels/circus.rs, crates/rugrats/src/levels/chuckies_glasses.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/src/levels/let_there_be_light.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T08:42:38-06:00
took: 56m
---

# 196. Characters run only when scheduled, by script or trigger

Characters now run only when the original schedules them. In the original a character's think, its animation included, runs only from the scheduler (0x80072bf0). Something has to put it there with 0x80049e18:
- a level script. Pickles' Home schedules only Spike (0x80078e7c); level 9 schedules Mr. Friend (0x8007b078), and all three after cine 18 (0x8007cac4);
- or its trigger, +0x48, from the trigger list (0x80072d38), once the player is within its radius (+0xa0, from the template's +0x34, by 0x8001f57c). Out of the radius again, +0x4c (0x800820ac) drops it.

The trigger comes from the character's init (0x80081bac: characters 23-26, 33, 35 and 39) or from a script: level 2's geese (0x8007914c) and level 12's shadows (0x8007b7c8). The port animated every character every frame. The hub's babies in the other rooms, for one, never move in the original: their tracks are uninitialised memory that never changes.

The port now has:
- `Npc::scheduled`, set where the scripts schedule;
- `Npc::trigger` (with `Npc::radius`, the template's), checked every frame;
- a character a script sets going (a mode other than idle) counts as scheduled.

Unscheduled characters skip the whole update. Box toss's babies, the circus's Angelica, Chuckie's Glasses' hiders and training's Angelica are scheduled while idle, as their EpisodeStarts do.

npc_check prints a summary for each character: its character number, trigger, mode, frames off, and whether each side's tracks ever changed. With the cinematics played and 900 frames settled, every level's characters are now animated or still as the original's, except one in level 12.

**Still unknown:**
- Level 12: the original schedules 14 (Spike) at its start with Tommy (1492) as his target (0x8007bb84). The port hides Spike until later, and the character it pairs with the original's moving one is its 1492.
- The trigger list takes five entries a frame in turn (with the objects' and talkers' own); the port checks every character every frame.
- Characters animated on both sides but off: level 6's 506, level 10's and level 11's 1492, level 13's and level 20's racers and babies.
- Level 21 after 900 frames: the original's Angelica is still, the port's follows.
