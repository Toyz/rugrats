---
number: 54
title: Level 1: the hub's doors, signs and Spike
date: 2026-10-15
area: play
files: crates/rugrats/src/level1.rs crates/rugrats/src/game.rs crates/rugrats/src/npc.rs crates/rugrats/src/player.rs
started: 2026-10-15T14:46:31-06:00
took: 20m
---

# 54. Level 1: the hub's doors, signs and Spike

The hub is ported from docs/levels/level-01.md.

- **Broken doors fixed.** Doors whose objects can be picked up (ids 52, 115, 519, 119, 33: the doors to 4, 5, 6, 11, 13) were filtered out of door use as "apart" placements, so five hub doors did nothing. A pickable door object is now used where it lies. Checked every door from its spot or beside it: exits to 2, 5, 9, 10, 11, 12, 13, 18 and 25. The doors to 4 and 6 are locked without bit 16.
- **Hides at every start.** The hides of cine 7's end hook (1776, 608, 609, 127, 514, 55-59) and of EpisodeStart (5555, 350-355). With the 117/715/720 objects hidden, message 33 cannot show, as in the original. The message 33/34 check now also skips hidden objects.
- **Door signs** (0x80049eec): drawn within 2500 (2D) in the player's room.
  - The difficulty word sits at y 62 and the level's name at y 80. Both are font 0, centred across, with shadows.
  - Hard is red for 3, 4 and 6; Medium is blue for 2, 5, 11 and 12; Easy is green for the rest. Level 25 is named "Ice Cream Mountain" (string 158).
  - The name turns cyan once the level is won (Game::won_levels from +0x38).
  - The mirror has no sign without bit 16.
  - The screenshot at the golf door matches the oracle's description.
- **Spike** (object 14):
  - He starts at home (47762, 727, 32948) and sleeps until Tommy is within 30000.
  - **Mode 2:** home once he is past the 12000 leash; Tommy while Tommy is within it, stopping at range / 2. He stopped 2973 away; the oracle showed 2992.
  - **Barks:** 0x800b96ec in rooms 24 and 39-41, every (rand & 255) x 8 dt.
  - **Fetch:** a throw of 678-680 says 191/192. While he follows, he runs to the toy, with the range shrunk to his reach so the follow doesn't stop at 3000. He picks it up (193/159/234/235), brings it back and puts it down 1024 ahead (190). He gives up in rooms 35/36.
  - Checked with a TP= teleport added to play_script: throw at 96, pick-up at 115, drop at 118.
- **No damage in the hub:** Game::no_damage turns off hazards, and Player::falls_hurt turns off fall damage.
- **Hub doors lose their talker line** to the sign's enter (0x800481c8), so the door to 3 no longer says 113.
- npc::Target::Point is new: a place to go (home, a toy).

**Still unknown:**
- Hinged and sliding doors (classes 3/4) are not ported. The playpen gate does not block Tommy in the port, so he walks out without opening it.
- Spike's sleep and wake animations (26/25), the mouth models 1212-1214, and the wall-bump give-up are not ported.
- Object 80's y = 746, object 518's template swap, the room 4 climb rule, and the signs' sparks are not ported.
- Rooms 11, 12 and 43 have speech 197-199 on entry. The port reads +0x64 only for flag 128, so check that they fire.
