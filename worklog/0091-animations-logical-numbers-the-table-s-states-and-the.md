---
number: 91
title: Animations: logical numbers, the table's states, and the sequencer
date: 2026-10-15
area: engine
files: crates/rugrats/src/actors/animator.rs crates/rugrats/src/actors/player.rs crates/rugrats/src/levels/golf.rs crates/rugrats/tests/play.rs docs/engine/gameplay.md
started: 2026-10-15T22:03:23-06:00
took: 14m
---

# 91. Animations: logical numbers, the table's states, and the sequencer

Worklog 90 left a question: are the port's animation numbers the
game's logical ones played as table states? They are, and the oracle
shows it.

**The map.** 0x800572a8 first calls 0x80056fe4, which turns its argument
into a state of the character's table. The port indexed the table with
the argument itself. Peeking the player's base track (+0xa8) on level 3
in the oracle:

| | oracle state | port before |
|---|---|---|
| stopped | 0 | 17 |
| running | 3, 4 | 2 |
| walking | 1, 2 | 1 |
| jump | 5, 6, landing 8 | 3, falling 15 |

The ANM frames match: state 3 is ANM 49 = frames 374..377 in DB03, the
oracle's 375-377.

`animator::table_state` is now 0x80056fe4: the table, plus three
computed cases:

- the climb (5): its rise picks 9-14 in bands of 128 (0x80056e44);
- 6: 28 or 32 (0x80056e10);
- 17: "as you were".

`Animator::set` takes logical numbers again, so golf's 29-31 stay as the
game asks for them. The idle is logical 0, the state the oracle shows
stopped.

**The sequencer.** At a state's last frame the word at 0x800b9c30
decides what happens (0x8005562c): ping-pong (4), loop (2), complete (1),
or hold. On completion 0x80056484 picks the next state:

- walk 1 and 2 swap, run 3 and 4 swap, idle 17 and 18 swap, 34 and 35
  swap;
- jump 5 goes to 6, 39 and 58 go to 40, 45 goes to 46;
- anything else returns to standing.

The base track now does this. The port's Fall after a jump keeps the
jump's animation, as the original keeps state 5. `set` asks only when the
logical number changes (the game asks on entering a state). `start`
restarts it for actions.

**Two fixes found on the way:**

- 0x8008a42c's cases for bodies 9 (Reptar: 0x800ba754 / 0x800ba0d4) and
  43 were missing, so Reptar used Tommy's table. His chomp (49) and fire
  (51) now play ANM 1030 and 1034.
- Climbs play the original's height animations (two boxes: state 14, 12
  frames). The level 6 stack tests now allow 120 frames for the climbs.

New tests:

- the level 3 sequence (run 3/4, walk 1/2, jump 5/6, stopped 0);
- every level's body has frames for standing, walking, running and
  jumping.

**Still unknown:**
- the root motion of states with bit 32 (climbs, the jump) is not ported; the port moves the body itself
- tracks 1-3 still loop; the original runs them through 0x8005678c and 0x80056708
- the footsteps at loop ends (0x800562bc: states 1 and 2, sounds 164-166 and 132) and the idle fidgets (+0xf0 = 23 after +0xf2 = 1200) are not ported
- what bit 64 of the end word does (state 16, the cry, has 97)
