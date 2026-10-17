---
number: 169
title: The port played against the original
date: 2026-10-16
area: test
files: crates/rugrats-hle/src/play.rs, crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/examples/boot_time.rs, crates/rugrats-hle/examples/air_push.rs, crates/rugrats/tests/hle.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T18:19:26-06:00
took: 56m
---

# 169. The port played against the original

The port is now proved against the original by playing both. `rugrats_hle::play`
boots the original into any level as the oracle probes did (the level
patched into 0x80018824 for the boot, the global 0x800c5678 set at the
loader 0x80017c0c, 0x8004472c branched past the attract wait, `play_cine`
0x80074958 returning 0 unless cinematics are wanted), about a minute a
boot; `tests/hle.rs` plays the same pad script on the port and on the
original and compares the player each game frame: position, heading
(+0x24), state (+0x18c), the camera's heading (atan2(-m02, m22) of the
matrix at +0x224), and on request the air velocity (+0x170). The tests
are ignored by default (a boot a minute): `cargo test --release -p
rugrats --test hle -- --ignored --test-threads 1`.

What the comparisons found, and the port now does as the original:

- **Input latency**: a press reaches the original's player a game frame
  later (the pad is read at the vertical blank). The harness holds the
  port's pad back by that frame; the port itself stays as quick as the
  host's input.
- **The first frame from a standstill** only changes the state
  (0x8005d11c: probes, slide or climb test, state 1 or 2); the step is the
  walk's or run's handler's, from the next frame. The port stepped at
  once.
- **The step comes before the turn**: the handler steps along the
  heading as it is (0x8005caac), the step is added (0x8005cb48), then the
  turn (0x8005bc80). The port turned first; from a standstill with a turn
  it went 2048 -> 1888 -> 1728 a step ahead of the original.
- **The step's rounding**: 0x8005caac sets the way (+0x178) to the step's
  length with 0x80076944 (VectorNormal, scaled, each axis toward zero).
  The port's `>> 12` floored negative steps: a unit a frame of drift.
- **No steering in the air or while landing**: the heading holds from the
  takeoff to standing again (1246 throughout in the run-and-jump test).
- **The air step in integers, in the body's frame**: the read of the
  original's stack (0x800616ac, 0x800616d4) gave the push 12 an axis
  before any turn (the branch that scales each axis by 340 >> 15, then
  0x4a90be59 >> 51 - not the other, which gives 9), turned by the camera
  (RotY) into the world and by the heading's transpose into the body,
  sideways halved; the drag is 32/4096 of the speed's length (an isqrt,
  0x8001f4ec) along its VectorNormal, both toward zero; the velocity is
  kept in the body's frame (+0x170) and each step turned back into the
  world. Worklog 164's fitted 12.55 and the float step are gone; the
  original's velocities are matched exactly frame by frame.
- **The launch** is built along the heading at the takeoff, not at the
  press (the original's +0x170 (-1, -1000, 679) for a run at 17 facing
  1246).
- **The bump's end**: state 9 reads no stick, so the original turns a
  frame after the bump ends; the port turned on that frame.

The hub run along the hall into the wall now matches the original
exactly on every compared field, every frame; the run and jump from the
hub's start matches exactly through the takeoff and within 1 to 4 in the
air. Tests that had been aligned by hand to the oracle's first moving
frame are shifted by the standing frame.

**Still unknown:**
- In the air the original is stopped by level 1's toy 403 (a run and jump
  from the start, frame 28) where the port lands on it: the air's wall
  test (0x80066014) against objects.
- Level 18's slide: the port stops 5 short of the wall (30861 to the
  original's 30866) and takes one more step into the corner before its
  bump; the ground move's probes against the COL (0x80064a34 through
  0x80067448, 0x80064178, 0x80064444) are not yet ported as the original
  runs them.
- The camera's heading differs from the original's by a unit now and
  then, and so the heading the stick sets.
- The original's position carries the run's root bob in y (748, 739, 756
  ... over the floor's 779); the port keeps it apart (`root_y`).
