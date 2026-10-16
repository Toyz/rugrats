---
number: 139
title: Running jumps and landings as the original's
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:09:15-06:00
took: 7m
---

# 139. Running jumps and landings as the original's

A running jump from the hub, peeked on the oracle (LEFT held, Cross at
VSync 2740, y and the table state 0x800e2bf0 each game frame):

- **Run** (3, 4): 769, 780, 751.
- **State 41**, entered on the press with no crouch: 731, one frame in
  place. Then off: 585, 461, 359, 279, 221, 185, 170, 177. The steps are
  -146, -124, -102, -80, -58, -36, -15, +7, the standing jump's arc.
  0x80060f1c gives Tommy's moving jump -1000 up, as the standing one.
- **Landing** (state 8): 809, 886, 912, 923, 923, 923, 901. Then state 0,
  886, and the run again. x and z are held through the landing, the stick
  held all the while.

The land state's handler (0x8005cba8) does not move the body. It only
does level 11's 505 touch and Reptar's rumble. A landing plays out
whatever the stick does.

The port:

- **Takeoff.** The moving jump now waits one frame in state 41, as the
  oracle's 731, before it is off. Its keys stop at the takeoff, so the
  first step is -146 and not -176.
- **Air.** Root y seen in the air does not carry on to the ground. The
  air state's leftover keys put the body 228 up as it landed.
- **Landing.** It stays in `State::Land` until the land animation ends,
  whatever the speed, and does not move.

Port against oracle: the eight steps match exactly. The landing goes
842, 886, 912, 923, 922, 908, then the run. The oracle's held 923 one
frame more here.

Test: a_running_jump_moves_as_the_original holds the eight steps
exactly, and six frames of landing.

**Still unknown:**

- In both oracle runs Tommy met something in the air (x and z held from
  2758), so the free landing's length was not seen. The landing lasts
  six frames in the port, seven or eight in the oracle.
