---
number: 81
title: Mini golf played hole by hole; every level now won by play
date: 2026-10-15
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-15T20:39:48-06:00
took: 9m
---

# 81. Mini golf played hole by hole; every level now won by play

golf_course_is_played plays the nine holes in course order, stroke by stroke as a player does. Each stroke walks up to the ball, aims Left/Right (the heading set), starts the power meter with Triangle and stops it with Triangle at a power, then the ball rolls. Every hole is holed at or under par. Each holes out to the next hole and the ninth to cine 35: 1, 2, 3, 2, 3, 2, 3, 4 and 1 strokes against pars 4 5 4 5 5 5 4 5 3.

- **How the shots were found:** golf_course_search (ignored, about a minute) searches each stroke over a fan of aims (the cup's bearing +-2048 by 128) and powers (30-217 by 17). It replays the hole from the tee each time, since the game is deterministic.
  - It keeps the shot that holes out, else the one that leaves the ball nearest the cup along the fairway.
  - That distance is a shortest-path field over the COL grid (250 cells, eight ways, through floor no higher than the cup's by more than 60).
  - Straight-line distance failed on hole 8 (31): the cup's pocket opens to the east, the far end of a loop, and the ball stalled behind its wall. Hole 7 (26) needs two banks off the walls.
- **Roll:** about 0.65 x power^2 on the flat (power 200 from hole one's tee is a hole in one).

**Won by play, every level:** 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21 and golf 23-31 (worklog 74 and 76-80, and here). 1 is the hub; 8, 14, 19 and 22 are cut.

**Still unknown:**
- nothing
