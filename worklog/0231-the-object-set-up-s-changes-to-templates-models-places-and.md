---
number: 231
title: The object set-up's changes to templates, models, places and copies; golf's ball and the guards' bite by the full line test
date: 2026-10-18
area: play
files: crates/rugrats/src/world/fixups.rs, crates/rugrats/src/media/cine.rs, crates/rugrats-data/src/model.rs, crates/rugrats-data/src/placement.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/levels/golf.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/examples/fixup_check.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, docs/engine/gameplay.md
started: 2026-10-18T15:33:00-06:00
took: 2h 38m
---

# 231. The object set-up's changes to templates, models, places and copies; golf's ball and the guards' bite by the full line test

The object set-up's changes to the shared tables, the objects' places and the copies it makes are now the port's too. The templates and model boxes equal the oracle's in every level checked, and golf's door runs at the original's speed.

**What 0x8004691c does beyond the object.** Every object of a loaded placement runs it, in the LP's order, and it writes into tables all objects share:
- the template table (file 239, loaded once at boot at 0x80118a0c in the oracle);
- the models' boxes.

Per level (its switch at 0x80046e5c) it also moves some objects, renames one, gives ids, makes copies and starts sound sources. The full table is in gameplay.md, "What the set-up changes beyond the object". The main lines:
- **Flags.** It clears 64 (climbable) on 248, 381, 960 and the two renamed 100s. It sets 8 (no object test) on 125, 527, 1244, 61, 687 and 1189.
- **Template +0x24.** A pickable template with none gets 19, and 1164, 954 and 678 get their own values.
- **Template +0x1a.** The automatic doors' speeds become 50 (770 in level 24) and 60 (855 in level 5). The golf door's cycle was 94 frames in the port, against the original's 72.
- **Boxes.** It changes boxes of 381, 248, 883, 1070, 1028, 960, 425, 1185, 653, 1182-1184, 1264, 232, 712 and 1168.
- **Copies.** Level 15 gets eight blocking copies, level 3 three, levels 10 and 18 one each. Levels 3, 4 ("Created 2 oildrums"), 7, 11 and 12 get drawn copies.
- **Ids.** 1231 becomes 524 (one of level 5's cine actors) and 473 becomes 519. Level 5's TV robots are numbered from 13000.
- **Pickables.** 402 and 948 stop being pickable, and 1128 and 795 start.

Found with `npc_check` TEMPLATES=1 and the oracle's new write watch (`RUGRATS_STOPW`, worklog 230): the golf door's 50 was written at 0x80047418, a case of the set-up's switch.

**The port** (`world/fixups.rs`, `Set::fix_up` at the end of `Set::load`, so play and the cines both see it):
- **Objects.** An object's flags are its placement's, computed as the original does. Pickability now comes from them, where it came from the template.
- **Copies.** A copy is a placement more. A blocking one is not drawn (`Placement::drawn`), and `objects_except` skips it.
- **Renamed objects.** The two renamed 100s get a set-up template: 100's record with 623's flags. Only their +0x96 changes; their model and calls were already 100's.
- **Sound sources.** These are kept as `Set::sound_sources` (0x8006fe04: sound, place, near, far, volume). Nothing plays them yet.
- **New:** `Models::get_mut` and the tool `fixup_check LEVEL`. It lists every object of the original's room lists without a placement of its template there, and the templates and model boxes that differ. In levels 1, 3-7, 10-12, 15, 16, 18, 20, 23, 24, 27 and 28 only spawned or scripted objects remain: collectibles, health, eggs, Cynthia's spot, characters settled on their floors.

**Golf's ball meets objects as the original's does.** With 1264's real box, the port's stand-in object test (any box the ball's edge entered turned it back) stopped every shot of hole 23. 0x80086668 tests the ball's centre segment, boxes not grown (only flag 0x40000000 grows them), and keeps the world only when nearer on every axis. `Game::ball_meets` now does that through the flights' `flight_meets`, the ball bouncing off the normal as off the world. Hole 23 plays with its old shots again. `golf_course_search` takes GOLF_HOLES and GOLF_PAR_EXTRA.

**A guard bites when its line test meets an object (0x80085220).** The guard's attack runs 0x80090554, the full line test (objects first, the world kept only when nearer on every axis), from its +0x18 (the lift in it) to the player. It attacks only when the test meets an object: the player, or anything but 1177 or 843. Nothing met (0) or the world (1) keep it still. The port tested the world alone, from its place without the lift. A robot of level 5 floating over Angelica on a floor the segment crossed never bit, where the original's bites her from frame 11. `Npc::line_test` and `Line` are new, and `Obstacle` carries its template.

**Tests.**
- `level_5_robot_wakes_as_the_original` finds its robot by its placement (`set_up_id`: LP id 2000 is now 13000-something on both sides).
- `level_5_ambience_as_the_original` had synced no robot at all. With ids 0 it took the nearest object of id 0 and put Angelica on that object's floor. It now leaves her where she starts, plays both sides from the start, and raises the oracle's frame budget (its lines had been a frame early where the original took dt 15 frames). All seven lines match on their frames.

**Sounds.**
- Level 24's door sounds all match (8).
- Levels 26 and 29 are still off:
  - Level 26: the golf Reptar's roar timer (0x800c4ce4) is a global the oracle carries in from before.
  - Level 29: class 10's think (0x8004e0f4) calls its +0x50 callback every period. For template 616 that is 0x80053344, a random sound (134, 143, 140) on the shared countdown 0x800c4ba0, and particles. The port has class 10 only in level 14's own code.

**Still unknown:**
- Level 5's robot bite scene: the port's robot bites a frame after the original's (frame 11), and the respawn spots differ ((38395, -5591, 28281) against (34864, -5365, -5324)).
- The sound sources' playing (0x8006fe04's record and its updater).
- Level 30's 1178 source picks its sound by rand at load, which the port does not draw.
- Level 1's 836 and 881 door use with +0x30 bit 16, and level 18's objects with no use.
- Class 10 objects in general (0x8004e0f4 with the +0x50 table 0x800b7eb0).
