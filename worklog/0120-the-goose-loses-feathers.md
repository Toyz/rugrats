---
number: 120
title: The goose loses feathers
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T00:26:54-06:00
took: 3m
---

# 120. The goose loses feathers

The touch callback 0x800511a8 switches on character - 2. For the goose,
char 21 (case 19), it does three things:

- calls 0x800509d8 five times at the touch point;
- plays sound 34, unplaced;
- starts animation 8 at 4096.

The port played 34 for every puck hit. It now does this:

- The goose throws five feathers, then sound 34.
- Chuckie (char 2) says 185 through `hit_line` instead, as the same
  switch has it for level 2.

A feather (0x800509d8):

- kind 5, flags 0x1030505: gravity 0x500, the world and objects, ends on
  the floor;
- sprite 276 (TEXPOS + 3312), half-size 50 x 200, radius 50;
- speed (rand & 0x7fff) % 250, capped at 100, from its own LCG at
  0x800c418c;
- within 2048 of up;
- no lifetime, bounce 1000, spin 3000.

The seed is 0xbe5c (48732), read on the original at frame 2. That is the
same value as the splash and burst seeds.

Test: level_2_grandpas_teeth_is_won now also checks that feathers fly.
Up to 9 are seen in the air at once. A check for 500 fails, so the count
is real.

**Still unknown:**

- The touch point: 0x800511a8's second argument from the collision
  (0x80086ef0). The port uses the puck's position.
- Animation 8's mode 1 for the goose. The port keeps its own 300 ms hit
  pause.
