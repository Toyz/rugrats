---
number: 121
title: Signs sparkle; a lobster's end
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/levels/hub.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/src/levels/aisle_seven.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T00:30:15-06:00
took: 4m
---

# 121. Signs sparkle; a lobster's end

0x80050fd0 throws three sparks:

- kind 5, flags 0x30305 (no gravity; the world and objects);
- sprite 296 (TEXPOS + 3552), half-size 200, face 0xd2800000;
- 500 ms, bounce 2048, radius 0, spin 0;
- damage is the object's +0x6d (template +0x32);
- each points anywhere in the sphere: a cone about +y, 0..4096, in the
  object's frame;
- speed is rand % 100 from a local LCG seeded 0xbe5c on every call, so
  the three speeds are the same each time.

After the sparks it plays sound 48 unplaced (179 in level 16), unless
0x800c416c is 0.

Callers:

- The hub's door signs, 0x80049eec. In the player's room it draws the
  label, then sparkles with 0x800c416c cleared (no sound), every frame.
  Checked on the original: with Tommy put at (9330, -700, 20760), it ran
  from 0x8004a1f4 on every game frame, and the screenshot shows yellow
  sparkles about level 10's sign.
- Level 16, templates 111 and 441: the same think, given trigger radius
  2500 by 0x8004691c. It sparkles only, no label.
- Level 3, 0x8007ebd4: a lobster at health 0 sparkles with sound 48,
  then goes.

The hub's sign selection moved into `signs()`, shared by the frame (the
sparks, one set per object) and the draw (one label per level).

Test: a_door_sign_sparkles puts Tommy 1200 in front of level 10's sign
(template 498, room 23). Three or more sprite 296 sparks are in the air.

**Still unknown:**

- Sign 111 (level 2's) at (-3394, 932, 8498) is in no room by the port's
  room boxes, so it never shows. The original compares object +0x3b with
  the player's +0x3b; the object's room may come from elsewhere.
- Damage carried by flights is not applied to anything yet.
- The trigger's enter and leave (0x80049e18, 0x80049e5c) are stood in for
  by a 2D 2500 test in the player's room.
