---
number: 214
title: The trigger list is the original's in every level and its pass runs five a frame; the loader's skipped objects are not made
date: 2026-10-17
area: engine
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/media/cine.rs, crates/rugrats/src/levels/hub.rs, crates/rugrats/src/levels/let_there_be_light.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T22:39:31-06:00
took: 1h 4m
---

# 214. The trigger list is the original's in every level and its pass runs five a frame; the loader's skipped objects are not made

The trigger list is the original's in every level, and its pass runs five a frame as the original's does. The loader's skipped objects are no longer made, which removes level 5's two phantom movers and level 11's two extra bone stands.

**The list (0x800c4c24), built as the loader builds it.** 0x800454dc makes each placed object in LP order and pushes it on the list's front (0x8001a1c0) when its init gave it an enter callback (+0x48). `Game::trigger_list` does the same. The inits that give one:
- classes 2, 8, 9 (the pie pool's last pie, which the class init returns), 10, 14, 16 and 17;
- class 15 but level 4's template 1189;
- class 11 but a template of flag 0x40000000;
- class 3 of template flag 0x40000000, or template 843 but id 8801 (0x8004845c);
- characters 23-26, 33, 35 and 39 (0x80081bac);
- an LP speech line (or template 613);
- level 16's templates 111 and 441, and level 1's objects with a door's level (the signs), within 2500;
- level 4's id 34 (Cynthia) within 2000. 0x8004691c's switch is on the level, not the class;
- in level 12 only, a battery's copies (collectible kind 12, 0x80048a44: 0x8004cf58 within 1000), copies 1 on then copy 0 (0x80049af4).

The radius is template +0x34, else 15000. Levels add to the front at their start:
- level 12's Shadows, in id order (0x8007b7c8, `Game::push_trigger`);
- level 2's geese (0x8007914c);
- the hub's Spike (0x80083a94), whose own enter and leave wake him and lay him down (`Game::push_watch`, `Game::trigger_in`).

`trigger_lists_as_the_original`: the port's list against the original's, template by template, in order, in levels 1-7, 9-18, 20 and 21. All match.

**The pass (0x80072d38(0)).** Five entries a frame from where the last pass stopped (0x800c4c20), back to the front at the end:
- one within its radius by the game's distance (0x8001f57c) and not in is entered: a character is scheduled (0x80049e18);
- one in and out of range is left: unscheduled (0x80049e5c).

The port tested every character every frame. It now runs the pass: `Game::trigger_pass`, `trigger_next`. In the oracle, level 5's robot 13002 is the 43rd of 51 with the cursor at the 25th, so it is woken on the fourth frame after Angelica arrives. That is the wait worklog 213 traced.

**The hub's Spike** wakes and sleeps by his entry, five a frame and by the game's distance, where the port woke him on any frame within 30000 straight.

**What the loader passes over (0x800455d0).** The port knew only the templates (1242, 1041, 1267, 834, 382, 625), in the cinematics' code. The full rule:
- those templates;
- level 16's templates 391-395;
- level 10's template 403;
- template 1211 at (38650, 884, -13520) or (7900, 884, -2993).

The set's load now gives such a placement a kind no loader reads (`NOT_LOADED`), keeping the indices. Level 5's templates 1041 and 1267 had been made as movers, and level 11's two 1211 stands as scenery.

**Still unknown:**
- The cursor at a level's start in play (the oracle's runs start from wherever the boot left it).
- What the other entries' enters do in the port: the doors (843's 0x8004d078), the speech lines (0x8004d0f0), the signs, the batteries (0x8004cf58). The port still runs each of those by its own distance every frame; only the characters' schedule comes from the pass.
- Level 5's robot wake compared in the oracle with the pass's cursor given the original's.
