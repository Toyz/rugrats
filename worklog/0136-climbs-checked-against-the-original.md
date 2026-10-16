---
number: 136
title: Climbs checked against the original
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
resolves: 91.1, 92.1
started: 2026-10-16T01:55:50-06:00
took: 5m
---

# 136. Climbs checked against the original

**The setup.** On the oracle, Tommy was put at (8945, 779, 12500) in the
hub, facing the hub's template 130, a seat whose top is 849 above the
floor. The stick was held toward it, and the position peeked every game
frame (/mnt/data/claude/scratch/screens/run35.txt).

**What the original does.** The climb takes 15 game frames. The body
rises from 779 to -125: the start plus the whole -904 of the 900 band's
root y. It is not scaled and not fitted to the rise. It settles on the
top (-111 to -69 while running). Its forward move is 405 across x and z,
along its heading, against the animation's 403. So Tommy's motion is not
scaled.

**The port.** The drawn y is now the start plus the root y so far, not
the rise in proportion (worklog 134). Against the oracle, frame by frame:

| frame | port y | oracle y | port along | oracle along |
|---|---|---|---|---|
| 1 | 724 | 725 | 0 | 0 |
| 4 | 474 | 475 | 52 | 52 |
| 8 | 255 | 256 | 92 | 94 |
| 10 | 24 | 25 | 292 | 294 |
| 12 | -97 | -97 | 350 | 353 |
| 14 | -93 | -92 | 416 | 419 |

Test: a_climb_moves_as_the_original holds the port to all 14 frames
(y within 3, the distance along within 4).

**Resolved from earlier entries:**

- 91.1, 92.1: the root motion of the climbs.

**Still unknown:**

- The oracle's start was pushed off the seat's face (to z 12342), the
  port's was not. The test compares the motion, not where it began.
- The root y of jumps, landings and walks (small bobs and dips) is not
  drawn; the floor test would take most of it back.
