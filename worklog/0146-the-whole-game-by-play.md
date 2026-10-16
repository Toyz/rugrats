---
number: 146
title: The whole game, by play
date: 2026-10-16
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-16T02:34:15-06:00
took: 4m
---

# 146. The whole game, by play

Before this, the whole game was covered two ways:

- one play test per level, each opened fresh;
- the app's run from a new game to the credits
  (app::play::tests::the_whole_game_from_new_to_the_credits), which ends
  each level by finishing its ending cinematic for it.

Nothing yet played the whole campaign through.

`the_whole_game_by_play` does. It starts from a new game's progress:

1. **The first piece.** Reptar Puzzle gives it on the first entry to the
   hub (+0x30 bit 4, 0x80094bfc(1)), as the app does then.
2. **Each door, in an order a player can take:** 2, 5, 9, 10, 11, 12, the
   golf course (25), then 3, 4, 6. Doors 3, 4 and 6 open only with
   Mirrorland won. In the hub, opened with the progress so far,
   `through_door` finds the door's placement (LP +0x42), puts Tommy
   before it facing it, and presses Triangle. The exit must lead to that
   level.
3. **The level,** opened with that progress as the game opens it (from the
   hub), won by the same play as its own test. Each level test's body is
   now a `win_N` helper (`win_box_toss` for 20 and 21, `win_golf` for the
   course), and `win_level` picks it.
4. **Progress::finish** records the win. Any bonus level it opens (the
   count's 20, 15, 21 and 17) is played straight after, as the game goes
   there.
5. **With all twelve pieces,** Reptar 2010 is played to City Hall.

The run plays 2, 5, 20, 9, 15, 10, 11, 21, 12, the golf course, 17, 3, 4,
6, then 7. It ends with twelve pieces, every main and bonus level won.

The first try took the doors in number order. Door 3 could not be found
after level 2: it is shut until Mirrorland, as in the original.

All tests pass.

**Still unknown:**

- The policies still put the player at the spot each level needs. The
  getting from place to place is not played, the doors of the hub aside.
- Levels 13 (cookie race), 16 (training) and 18 (egg hunt) are not on the
  run. The count does not open them, and they have their own tests.
