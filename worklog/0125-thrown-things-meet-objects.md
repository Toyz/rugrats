---
number: 125
title: Thrown things meet objects
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/tests/play.rs
resolves: 61.3
started: 2026-10-16T00:55:21-06:00
took: 6m
---

# 125. Thrown things meet objects

0x80061f88 builds the descriptor of a thing thrown or dropped. Its flags:

- 0x9070700 for most things: gravity 0x700; the world, objects (0x20000)
  and their touches (0x40000); ends on the floor. Level 9 also sets
  0x40000.
- 0x190f0700 for an object with flag 0x800000: all collisions, damage,
  and the objects' own test (0x10000000).
- 0x9010700 for levels 20 and 21's held box or bottle (0x800c42a4): the
  world only.

The port's thrown things met only the COL world and things at rest. They
passed through furniture and could not land on a table.

Now the game keeps this frame's object boxes in `Game::solid`: the
static objects, breakables, movers, spinners, walkers and doors, without
things and characters. A thrown thing:

- **lands** on an object's top as it does on a resting thing's
  (stacking, 0x8006c7a0);
- **bounces** off an object's side. That is when the step moves it into
  the box seen from above, at a height the box spans, and it was outside
  before. The bounce uses `rebound` off the side it came through
  (`Box3::side_normal`) with the template's bounce, as off a wall.

`Thing::world_only` marks levels 20 and 21's box or bottle, which skips
both. Without it, level 20's lob landed on the furniture short of the
baby and the level could not be won. That matches the original's world
only flags.

Test: a_thrown_thing_lands_on_an_object drops a hub thing 1500 over a
wide object whose top is more than 300 above the floor. It rests on the
top. With `world_only` set, the same drop ends on the floor (727 + 297
against a top of -595), so the test tells the two apart.

**Resolved from earlier entries:**

- 61.3: the pause's Quit confirm already has the four items of
  0x80036368. They are "Yes?" and "Save Game?" on the left, "No?" and
  "Load Game?" on the right, under "Quit Level?" or "Quit Game?".

**Still unknown:**

- The original tests the objects along the flight's segment (0x80086340
  with 0x8006a714, against the model's faces), not boxes.
- The touches a thrown thing gives the objects it meets are not called:
  levels 3 and 16's model parts (touches 1-7), levels 5 and 6 (11, 13,
  15).
- The 0x190f0700 case: objects with flag 0x800000, and template 777's
  throw sounds 53, 163, 171.
