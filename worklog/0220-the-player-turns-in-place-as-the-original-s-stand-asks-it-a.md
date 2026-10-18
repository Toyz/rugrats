---
number: 220
title: The player turns in place as the original's stand asks it; a half turn keeps its sign; Spike and 34 turn slowly; the oracle's put leaves the floor
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T06:03:34-06:00
took: 1h 15m
---

# 220. The player turns in place as the original's stand asks it; a half turn keeps its sign; Spike and 34 turn slowly; the oracle's put leaves the floor

The player turns where it stands as the original's does. A half turn goes the original's way, and a put leaves the original's floor under the body.

**The turn in place (0x8005d11c with 0x80059204).** Standing, the stand handler asks 0x80059204 for the turn left to make. For the player that is the wanted heading less the heading, folded as the turn folds it, while the stick is pushed (+0x1b0) and the speed (+0x184) is 0.
- The stand gets there only when neither the run test (0x800590d4) nor the walk test (0x80059008) passes. Both need the speed the stick set this frame, which the stand zeroes only later. So a full stick just after a landing runs, and does not turn in place, though +0x184 reads 0 after the frame (`hub_start_run_and_jump_as_the_original`, frames 38 and 39). The port keeps the stick's speed for it (`Player::stick_speed`).
- Under 0 it asks logical 53 (state 63, to the left), over 0 logical 52 (state 64, to the right), unless that state plays.
- At 0 it asks the stand (0x80057274) unless the base plays state 0, 23, 24 or 60.

The port's player stood in the idle through a standing turn, so it missed the turn's root bob (-7, -3, +14, ...). It now plays it (`anim::TURN_LEFT`, `anim::TURN_RIGHT`; `Player::stick_size`).

**The turn rate (0x8005bbf4).** 1025 dt >> 6. Characters 6 and 34 turn at 298, half as fast again while turning in place. The game's body setup gave the slow rate to 6 and 21 (0x80062eac's pair, another test); it is now 6 and 34, as level 11 already had.

**A half turn keeps its sign (0x8005bc80).** The turn folds the difference from 2049 by less 4096 and under -2048 by plus 4096, so +2048 stays +2048. The port's `wrap` made it -2048. Facing 0 and asked for 2048, the original turns up through 1024 and the port went down through 3936. `fold` now does as the turn does. The stick (0x8005be24) keeps both headings in 0..4096, which the port's do too.

**A put leaves the floor (the oracle's tools).** The settling (0x800452f8) queries the floor (0x8002b240 into +0x13c and +0x140) and copies it to +0x148. `Both::put` moved the original's player without it, so the next state's first frame took its gap to the old floor. On a shelf top that launched a jump 345 too high. The put now writes the floor from the same query.

**Oracle.** `level_3_turn_in_place_as_the_original`: Tommy put on a shelf top facing 0, the camera along 2048, Up held. For 8 frames his place, heading, tracks, height with the root and camera are the original's (160 a frame up through 1024, state 64).

**The crouch over an edge, tried again.** A crouch comes only from a jump begun walking. A run jumps with no crouch (state 41). Level 3's shelf drops (found by a scan of floors along straight ways) are reachable walking, but from the run's or the walk's first frame the follow camera parts from the original's by a few units. That turns the stick's way, so no run reaches the edge on the original's line.

**Still unknown:**
- The follow camera from a walk's or run's first frame (0x80027304): 2051 against 2048, 2059 against 2062 on level 3's shelf. It keeps the crouch over an edge from being compared.
- Whether the stand's turn in place runs for a carrying player (the port keeps the carry's idle).
- Levels 20 and 21's Angelica compared through her modes 4, 8 and the lunge.
- Level 5's robot 17 after the original's lag frame (dt 15): the sweep tool pairs frames a frame apart from there.
