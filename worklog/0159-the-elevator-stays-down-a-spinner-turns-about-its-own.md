---
number: 159
title: The elevator stays down; a spinner turns about its own upright
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/movers.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/toys.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T10:45:59-06:00
took: 25m
---

# 159. The elevator stays down; a spinner turns about its own upright

Two of worklog 158's guesses checked on the oracle; both were the port's.

**Level 5's elevator stays down while ridden.** The oracle, Angelica
jumped onto it at (65540, 8708) at frame 3600 and left alone: the
elevator from y -4578 down to 7722 by 4100, and there still at 5600, she
at 6739 on it. The port took her back up: its think ended at the bottom
(every axis done) and, ridden, a ridden-only mover was armed again every
frame, each arming turning the done axes. In the original arming is
0x8004f228, called on landing (from 0x80062894; teleporting onto it does
not set +0xf8, a landing does), and at the bottom 0x8004ee00 asks
0x8004ed00, which answers 0 while ridden: the think ends (-1) and nothing
arms it again. `Mover::rode` keeps the last frame's riding; a ridden-only
mover arms on the frame riding begins. Test
`level_5s_elevator_stays_down_while_ridden_as_the_original` holds the
oracle's numbers (7722, 6739), the walking test steps off at the bottom.

**A spinner's box turns as the original's.** The oracle, Tommy run east
along z 4290 at level 3's tipped toy (LP 143, template 526, class 2) at
four phases of its turn: never stopped (x 170 a frame through 73600). The
port stopped him at 69860 every time. Its think (0x8004e2ac, dumped by a
hook on the oracle) adds the turn to the object's own angles: +0x24 rises
70 a frame (template +0x18 = 35 a VSync) while +0x28 holds -1024 (its
placement); the collision (0x80069c40) and drawing make the matrix from
the angles together (0x80020ee0 = `Trig::euler`). M(a, 0, c) = M_c M_a,
so the spin turns about the toy's own upright and its height stays
692..968. The port had turned the placement by the spin (the other
order): the box swept up and down through Tommy. `Spinner::rot` now
takes the placement's angles from its matrix (`placed_angles`), adds the
turn, and makes one matrix; level 3's toy keeps its height, and the run
goes on as the oracle's, frame for frame in x.

The object collision itself, read on the way: 0x80065210 tests five
segments of the body's centre line (up to the body box's top, along the
step there, down to its bottom capped by +0x68, along the step there, and
the end's top to bottom) against each model box of the objects in the
nearby list (+0x100: the objects of the player's room and its portals'
rooms whose cylinder meets his, 0x8006add8, those without +0x00 0x80000),
template flag 8 skipped, at the object's angles now. The port's
footprint-and-height test stands in for it.

**Still unknown:**
- What +0x68 holds (the body's lowest tested height): the port steps
  Tommy onto the toy (y 447 for five frames) where the oracle's stays at
  779.
- Whether walking (not a jump) onto level 5's elevator arms it in the
  original (the oracle runs land on it).
