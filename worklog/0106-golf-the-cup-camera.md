---
number: 106
title: Golf: the cup camera
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/golf.rs crates/rugrats/tests/play.rs
started: 2026-10-15T23:01:50-06:00
took: 2m
---

# 106. Golf: the cup camera

Part of 35.2: the golf camera's view near the cup (docs/levels/golf.md,
"Golf mode 12"). After the stroke, 0x8002953c counts 0x800c4a58 down
from 4096 (set on entering state 13) by 0x800c3f20 = 136 a frame, about
30 frames. Once it is out, a rolling ball whose squared 2D distance to
the cup is under 0x800b6df0[level - 22] (131072000 for every hole,
11448) is watched by 0x800259a0(camera, ball, 0x800c4a50): the fixed
camera of the ball's room (its post, room +0x5c), looking at the ball.
The doc's oracle run saw a fixed eye at (31580, -3554, 29640).

The port followed the ball from the golf eye the whole roll. Now
`Golf::cup_delay` runs the countdown, and near the cup the eye is the
ball's room post. The test `golf_watches_the_ball_near_the_cup` plays
hole 27's hole-in-one and checks the eye sits on a room post during the
roll. It fails on the code before this change.

**Still unknown:**
- the activity golf score card (35.2's last part)
- whether the cup camera lets go of the ball before it stops (the port keeps it until the roll ends)
