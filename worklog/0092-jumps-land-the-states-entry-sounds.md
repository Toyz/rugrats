---
number: 92
title: Jumps land; the states' entry sounds
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/animator.rs crates/rugrats/src/actors/player.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/tests/play.rs
started: 2026-10-15T22:17:38-06:00
took: 4m
---

# 92. Jumps land; the states' entry sounds

Following up 91's open items.

**Jumps that never landed.** A test of a standing jump on level 3 found
the player hanging 3 above the floor in Fall, with the fall speed
growing and the stick doing nothing. The same happens on the committed
code before 91. The cause was the step that would cross the floor: it
ends below the floor, the floor probe there is inside the ground (kind
non-zero, no tile), and the port put the body back where the frame
began, 3 short, every frame. The original lands when y + vy reaches the
floor (0x8005f1b4). Now a descending step that would end in the ground
lands on the floor under where the step began.

**The landing animation.** The port went from Land to Stand on the
landing frame, because the check asked whether LAND was playing before
it had been asked for. Land now holds until its animation (logical 4,
table 8) has played. The jump now runs 5, 6, 8, 0, as in the oracle.

**Entry sounds** (0x800562bc, positional via 0x8006f644). They play when
a body's base track enters a table state:

- 6 entering 6 or 41: a standing jump's fall half, or a moving jump at
  once. The port used to play 6 at every take-off.
- 11 entering 8: landing.
- footsteps entering the walk's halves 1 and 2, only for Spike in
  levels 9 and 14 (132) and characters 47 (166), 33 (164) and 22 (165).
  The babies have none.

`Animator::take_entered` hands over the states entered; `Game` plays
their sounds at the player and at every shown NPC. The test
`a_jump_sounds_at_its_states` checks 6 after the take-off and 11 on
landing.

**Still unknown:**
- the root motion of bit-32 states and tracks 1-3's own sequencing (from 91)
- the idle fidgets (+0xf0 = 23 after +0xf2 = 1200) and the flags 0x20000000 / 0x80000000 that 0x800562bc sets
