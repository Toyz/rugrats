---
number: 109
title: Movers and the elevator as the original thinks them
date: 2026-10-15
area: play
files: crates/rugrats/src/world/game/movers.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/src/levels/outer_space.rs crates/rugrats/tests/play.rs
resolves: 37.2
started: 2026-10-15T23:10:21-06:00
took: 7m
---

# 109. Movers and the elevator as the original thinks them

The port's moving platforms were a simplification. They now follow
0x80048804 (setup) and 0x8004ee00 (the think) as written
(docs/levels/level-05.md, "The elevator").

**Setup.**

- +0xbe starts 0: every axis "back" and so done. The enter callback
  0x8004f228 arms the mover (schedules its think) and turns each done
  axis, which is what sends it out.
- Template flag 0x10000000 gives object 0x20000: back and forth.
- 0x40000000 gives +0xbe 64 and no enter callback: armed only by landing
  on it (0x800627a0 sees +0xf8).
- Template 855 gives +0xbe 256: it calls.
- +0xc0 is the base y.

**The think, each frame while armed.**

- Unridden, it holds while 0x8006baa8 finds the player in its boxes, and
  asks 0x8004ed00 whether to turn. A calling mover turns toward the end
  nearer the player's height. Level 5's 855 just answers 1.
- Per axis, step toward the range or back to 0 until done. There is no
  clamp, so 205 steps of 60 end 12 past the elevator's 12288.
- Level 5's elevator, unridden with that 1, does not move while below
  half its range.
- After the move:
  - back-and-forth movers turn their done axes, unless ride-only and
    neither ridden nor with the player over it (0x8004ec34);
  - others stop their think when every axis is done and 0x8004ed00
    answers 0.

The port arms on the trigger's enter edge (not every frame within it),
on landing for ride-only movers, and has `Mover::call`, `arm` and `done`.
`Game::mover_tests` gives 0x8004ec34 (the player over the model's first
box in x and z, in the mover's frame) and the blocking test.

Level 5's init sets template 855's step to 60 (0x80046ed8 writes the
template), so the port now matches by template, not by id.

The test `level_5_the_elevator_goes_down` puts Tommy on the elevator. It
goes down 12300 in 205 frames from the landing, as the oracle's 410
VSyncs from -4578 to about 7710, carrying him.

**Still unknown:**
- the blocking test 0x8006baa8 in full (the port tests the player's body against the mover's boxes)
- what +0xbe bit 128 (template flag 0x20000000) does
