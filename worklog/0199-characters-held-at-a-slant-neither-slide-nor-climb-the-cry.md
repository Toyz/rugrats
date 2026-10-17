---
number: 199
title: Characters held at a slant neither slide nor climb, the cry comes with the damage and runs its 210 frames; Mr. Friend in levels 9 and 14 matches the original every frame
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/story.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T13:13:11-06:00
took: 1h 11m
---

# 199. Characters held at a slant neither slide nor climb, the cry comes with the damage and runs its 210 frames; Mr. Friend in levels 9 and 14 matches the original every frame

A longer sweep (`npc_check`, 900 frames past each level's opening) found Mr. Friend off the original in levels 9 and 14. Both now match on every frame: level 14's Mr. Friend for all 900, level 9's through his last bite, Tommy's whole cry and the frame after it.

**A character held by what it meets.** Level 14's Mr. Friend walks toward Tommy and stops at (4880, -5620, 8100) against an object (template 393, a 531 step) at a slant. The doc had guessed "blocked by furniture; not traced"; now it is traced (the character probe, 0x80064838: kind 5, delta -531, the wall's normal (0, 0, -4096)):
- 0x80065120, the slide along the face, answers no at once for an object of flag 0x4000. A character never takes it; the port slid him along the wall.
- 0x80062cf4 lets a climb start only under the slant `head_on` (384; 220 for Spike). The port climbed the step. A character held at a slant now stays held with no floor to climb to; it stands and its speed is put by, as the original's does.

**The cry (ForceCry, 0x8007fd0c, 0x8007f920).**
- 0x80077864, applying the Damage that takes the player to 0, calls ForceCry there and then, before any update. Level 9's last bite froze the world that same frame; the port's check came at the frame's end, after the characters' step.
- 0x8007f920 runs the body's update, asks the cry (16 at 2560), and runs the body again, which steps it once, before its loop. The loop takes dt from 2100 each pass and steps the animation. It ends at 0, or when the base completes (+0xe0 0x8000000), not when the cry holds on its last frame: 210 frames. The port stepped the cry once fewer and ended it when the animation held, two frames early.

**npc_check.** It gives a follower its place in its bite (+0x699) and its aim (+0x67c), and the player's health, with the rest of a character's state. It stops at the port's level exit, prints the player's base track beside the original's (`PLAYER`), and with `CALLS` the character probe's kind, limit, record and the object it touched. Levels 2-7, 9-12, 14-16, 18 and 21 now show no character off the original for 900 frames, but for lag frames.

**Still unknown:**
- Level 13's Angelica in `npc_check`: the port's race is not under way at its frame 0 when the original's is. The race itself is proven by `level_13_angelica_races_as_the_original`.
- Level 20's Angelica: her lunge depends on the player, whom the tool does not move.
- The cry from damage other than a posted Damage (a hazard's touch): when in the frame the original's ForceCry comes.
