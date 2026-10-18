---
number: 218
title: The floor query puts a step's lift back with the place; the cry comes from damage, not the health; hole 26's Reptar takes the golf table: the sweep matches in levels 1-7, 9-18 and holes 23-31
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/examples/npc_check.rs, docs/engine/gameplay.md, docs/levels/golf.md
started: 2026-10-18T03:42:09-06:00
took: 1h 39m
---

# 218. The floor query puts a step's lift back with the place; the cry comes from damage, not the health; hole 26's Reptar takes the golf table: the sweep matches in levels 1-7, 9-18 and holes 23-31

Every character in the height sweep now matches the original in levels 1-7, 9-18 and the golf holes 23-31. The last ones off were:
- level 13's Angelica, running in place at the start line;
- hole 26's Reptar, whose animation table was the wrong one;
- the golf holes' player, made to cry by a health of 0 the original never cries at.

**The floor query puts the lift back too (0x800627a0).** After the body's step the original asks the floor at the new place (0x8002b240 at the origin, +0x1c, the root's lift in it). With no tile there, or the origin below the floor (+0x13c) or above the ceiling (+0x140), the whole place goes back to where the frame began (+0x160), that step's lift with it. The animation's keys go on, so the lift stays where it was put back.

Level 13's Angelica shows it. At the start line she runs at the raised block (a tile floored at -5120), let by her probe (+0x688 bit 0x1000). Each frame the original moves her 80, -150 and lifts her by the key, then puts it all back: she stays at y 520. The port's lift went on, -33, -55, -77.
- `RootCurve` adds each step's change to its height, where it used to set the height from the keys. A height put back stays put back (`RootCurve::put_back`).
- `Player::drive` marks the put-backs (`put_back`): no tile, and into a wall or a step too high, which the original reaches by the same query.
- A character's body checks its origin, place and lift, against the floor query at its new place (`Npc::drive_body`).

**The cry comes from the damage, not the health (0x80077864).** The original cries (ForceCry, 0x8007fd0c) when Damage leaves the player no health; it never polls the health. The port cried whenever the health was 0. In the golf holes the original's player has 0 health and plays on, and the port cried and played the hole again at frame 209. `Player::hurt` now marks the cry (`hurt_out`), and the game and level 2 cry on that.

**Hole 26's Reptar takes the golf table (0x8008a42c).** For character 9 in levels 23-31, 0x8008a42c gives the golf holes' tables (entries 0x800c4584, remap 0x800ba158). The port used the other one, whose state 0 is an empty column (966), so Reptar stood frozen. The port's comment had it that the golf table has no Reptar to use it. `Animator::load_in` now takes the level; every other character's tables were checked against the switch.

**The sweep, every level** (`npc_check`, YCHECK, 300 frames):
- 0 frames off in levels 1, 2, 3, 4, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17 and 18, and in holes 23-31;
- level 9 matches from its sync at frame 20, after the original's respawn.

Not swept:
- Level 8 is the golf course's record only (Ice Cream Mountain's door goes to the holes); its LEVEL file has no record table the port can read.
- Level 19 (Be Bad) and level 22 (multiplayer golf) have no files on the disc (453, 328).

**Level 20 and 21's Angelica.** Booted without the cinematics, the original never runs the round start (0x80079cb0, watched for 300 frames), so its Angelica stays unset (mode 0, never scheduled, its tracks never set up). Booted with them (CINES=1), the babies match every frame. Angelica is then in mode 8 after a lunge (state 28), a mode the tool does not give the port.

The tool gained:
- `WATCH=addr`, each call of a function in the original;
- `PRESS=f`, Cross on the original's pad that frame;
- `TAIL`, the place through the body's last calls.

**Still unknown:**
- Levels 20 and 21's Angelica compared through her modes 4, 8 and the lunge: the tool syncs only the post and follow modes.
- Level 5's robot 17 after the original's lag frame (dt 15). The tool's frame ends there before the original's update: frame 3's root step prints before frame 3's place (still -8118), and frame 4 runs no update yet moves the place by the step. So the pairing is a frame apart from there. `level_5_robot_wakes_as_the_original` covers the robots frame by frame without a lag.
- The trigger pass's cursor at a level's start in play, and what the list's other entries do in the port (doors, speech lines, signs, batteries).
- The crouch over an edge.
