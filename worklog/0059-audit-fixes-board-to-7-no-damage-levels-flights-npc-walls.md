---
number: 59
title: Audit fixes: board to 7, no-damage levels, flights, NPC walls, activity golf
date: 2026-10-15
area: play
files: crates/rugrats/src/game.rs crates/rugrats/src/npc.rs crates/rugrats/src/golf.rs crates/rugrats/src/screens.rs crates/rugrats/src/main.rs crates/rugrats/src/level20.rs
started: 2026-10-15T15:43:50-06:00
took: 14m
---

# 59. Audit fixes: board to 7, no-damage levels, flights, NPC walls, activity golf

An audit compared every docs/levels "For the port" list with the code and ranked the gaps: A blocks progress, B is wrong while playing, C is cosmetic. Fixed from it:

- **A1 (the board):** once +0x30 bit 512 is set (Reptar 2010 begun), the board's use is SwitchEpisode(7) (0x8004d3f0). Before, a player who left level 7 had no way back and could not finish the game.
- **B1 (no damage):** Damage is dropped in levels 1, 11, 17 and 23-31 (0x800780d8). Game::no_damage and Player::falls_hurt are now set by level. Verified: standing on golf's template 408 in DB25 no longer hurts.
- **B2 (flights):** a thrown or dropped thing's step into a wall or a higher column is taken back, and the thing falls where it was (0x80060810). Things also land on resting things' tops (stacking, 0x8006c7a0). Verified: level 6's box thrown at the -4736 ledge now falls to the floor, where before it went onto the platform.
- **B5/B6 (buttons):**
  - R1 puts the camera behind the player (0x80026ddc).
  - Select's restart menu no longer opens in activity play.
  - L1 (0x8002729c's animated tilt) is not ported.
- **B7 (pause):** the pause shows the counters: pieces with model 292 at (-192, 76), bars with 291 at (192, 76) (screenshot).
- **B10 (NPCs and objects):** npc::World now carries the frame's object boxes. A chaser's step deeper into one is taken back; route walkers in Posts/Wait mode are exempt. Without that exemption level 13's race stuck at post 37; with it the race ends at cine 20 as before. Level 9's Mr. Friend still reaches Tommy.
- **B13 (levels 20-21):** Angelica follows the box itself, through Target::Point, and the lunge distance is measured to the box.
- **B14 (Hole Five):** the ball-reset zone (template 1264, 0x80081758).
- **B4 (activity golf):**
  - In activity play, holing ends the turn with no par judgement, cry or record; the strokes go into main's golf_scores.
  - Page::Scores is the score card (0x80095768 layout, strings 123-125, the globals 0x800c46d4-0x800c4704; screenshot). It is shown before each next hole and at the round's end, then the Activities menu follows.

**Still unknown:**
- Mini-golf's player count and face picks (0x80096134) are not ported, so the round is one player, Tommy.
- B3 (level 6's box route): stacking and the wall test are in, but the route has not been played through.
- B8 (the save record menu), B9 (the Audio menu), B11 (level 5's float cameras) and B12 (level 6's moving toys) are still to do.
