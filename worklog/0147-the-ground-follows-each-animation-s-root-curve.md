---
number: 147
title: The ground follows each animation's root curve
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:38:04-06:00
took: 4m
---

# 147. The ground follows each animation's root curve

After a climb the oracle's Tommy goes from -55 over the floor to the
floor in about seven frames. Worklog 145 left that open. The oracle's
table states (0x800e2bf0) through it:

| VSync | y | state |
|---|---|---|
| 2734 | -125 | 0 |
| 2736 | -111 | 3 |
| 2738 | -115 | 3 |
| 2740 | -110 | 3 |
| 2742 | -93 | 3 |
| 2744 | -88 | 3 |
| 2746 | -80 | 3 |
| 2748 | -69 | 4 |

The floor is -70. Against it, from 2738 on, that is -45, -40, -23, -18,
-10, +1: the run's own curve from the floor, the same as in a run that
began there (worklog 138: -40, -23, -18, -10, +1). Only state 3's first
frame differs (-41 for -33). It starts from where the body was and blends
that away, as 0x80055904 sets +0xde from the frame's start. The landing's
return to the floor (868, 845, 822, 799, 779) is the same thing, through
the idle's first frame.

So the ground offset `root_y` is now each animation's root curve so far
plus `root_base`: where the body was as the animation began, scaled away
over its first frame. A new animation or a loop's next round takes the
base from where the last left the body, including the rest of one that
ran to its end. The fixed 23-a-frame return is gone.

Against the oracle:

- after the climb, port -125, -120, -110, -93, -88, -80, -69, -99, oracle
  -125, -111, -115, -110, -93, -88, -80, -69, -98;
- after the landing, port 893, 886, 864, 843, 821, 800, 779, oracle 894,
  868, 845, 822, 799, 779.

The jump, running, climb and Chuckie tests still hold.

The campaign by play (worklog 146) now also plays training (16) first.

**Still unknown:**

- The oracle spends a frame in state 0 between the climb and the run,
  and the port does not; that shifts the phase by one.
