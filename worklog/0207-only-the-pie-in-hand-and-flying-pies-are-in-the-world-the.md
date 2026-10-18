---
number: 207
title: Only the pie in hand and flying pies are in the world; the ambience runs last; cines move characters in no room
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/audio/ambience.rs, crates/rugrats/src/levels/cookie_race.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, docs/engine/cinematics.md, docs/levels/level-16.md, docs/levels/level-03.md, docs/levels/level-05.md, docs/levels/level-13.md
started: 2026-10-17T18:43:05-06:00
took: 17m
---

# 207. Only the pie in hand and flying pies are in the world; the ambience runs last; cines move characters in no room

Three guesses resolved: which pies the toss shows, when the ambience runs and what level 5's robot line waits for, and where level 13's Angelica starts.

**Pies in the world.** The toss's six pies are made by 0x80049870 / 0x8004997c, which link none into a room. 0x80049464 / 0x800493f8 do not link them either.
- At the start (0x8007e680 / 0x8007bc60), pie 0 goes into the hand and out of its room's list (0x8001a810). 0x8001a810 removes: 0x80049380 calls it until none is left, and 0x8004a34c calls it before 0x8001a1c0 inserts.
- The pie in hand is drawn with the player: 0x8003205c calls +0x1f0's draw.
- In the oracle, no pie is in any room's list at the start. A thrown pie is listed while it flies and leaves its room's list when its flight ends.
- The port showed all six at the pie spot. Now only the one in hand is shown; the next is shown as it comes to the hand.
- The ends: level 16's 0x8007e904 takes the pie in hand out (+0x1f0 = 0). Level 3's 0x8007e3c0 does too, and runs 0x80049380 on all six. So at either end only flying pies remain; this was `end_pie_toss`'s guess.
- The level 3 and 16 docs said the pie went "into" its room's list; corrected.
- `level_16_pie_flies_as_the_original` checks the original's lists and the port's hidden pies at the start.

**The ambience runs last.** 0x8001886c ends the game frame with 0x800818e0, after the objects, flights and shots. The port ran it early in the frame; it now runs after the rest, and still in the message path.
- `level_12_ambience_as_the_original`: Tommy standing where cine 66 leaves him, the rand (0x800c3f94) and the countdown (0x800c4cc0) given the original's. For 1500 frames the port says the original's lines on the original's frames (258, 256, 258, 256 at 91, 421, 1100, 1453).

**Level 5's robot line.** 0x800c443c is set by a guard in mode 11 as it begins its attack (0x80084084 at 0x800853a0). That needs:
- the target within the guard's range of home and less than 2800 off in height;
- the ray toward the target (0x80090554) meeting the target first (not a wall, not templates 1177 or 843);
- the target within reach (+0x690) and the heights overlapping (0x80083ff0).

The ambience reads 0x800c443c and clears it every call. The port's guess was any robot guarding within its range. Now `Game::attacker` is the last guard whose bite began this frame, taken by case 5, within 5000 by the game's distance (+0x674, 0x8001f57c).

**Cines move characters in no room.** A pose (0x80073f08) moves the character the registry has by the id, in a room or not:
- it puts it in the room at its new place (0x8004a218: characters' list +0x4c);
- it looks up the floor under it (+0x13c, +0x148);
- it does not settle its height.

`Game::cine_left` moved only characters already in a room, and settled them. Now it moves every one named, to the pose's place as it is.
- Level 13's Angelica, placed off the map, starts at cine 6's (23346, -5625, -4942), as the oracle has her. `CookieRace::start` placed her at her first post as a guess; that guess is gone.
- Level 7's babies are in room 12 after cine 79, as the level 7 doc's oracle RAM has them (Tommy at (43394, -1012, -15004)). `characters_in_no_room_are_not_drawn` held the opposite and now holds this.
- New: `a_start_cinematic_leaves_angelica_as_the_original`.

**Still unknown:**
- Level 5's robot line against the oracle (level 5's boot needs the character hook; not run).
- Whether a flying pie at level 3's end is relinked by its flight (0x80049380 takes it out of its list mid-flight).
- Whether the babies' AI in room 12 (level 7) ever runs in the original (+0x00 bit 0x4000, never read by level 7's code).
