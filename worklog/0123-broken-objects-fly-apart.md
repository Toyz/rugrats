---
number: 123
title: Broken objects fly apart
date: 2026-10-16
area: render
files: crates/rugrats/src/world/shards.rs, crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/mod.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/levels/toy_palace.rs, crates/rugrats/src/levels/mirrorland.rs, crates/rugrats/src/levels/reptar_2010.rs, crates/rugrats/tests/play.rs
resolves: 29.1, 54.2, 110.2, 111.1
started: 2026-10-16T00:38:58-06:00
took: 13m
---

# 123. Broken objects fly apart

0x8004cdfc breaks an object into its faces. It walks the model's face
sections from the first, but only flat textured ones (0x24 triangles,
0x2c quads, 32 faces a section at most), and stops at the first section
of another kind. Each face is cut in four, then again, until no edge
spans more than 1024 on an axis or more than the object's radius (+0x86):

- triangles by 0x8004c2e8, into the three corner triangles and the
  middle;
- quads by 0x8004b850, into four quads about the centre;
- texels and colours split at the same midpoints.

Each finished piece takes one of ten slots (0x800e2788, 96 bytes,
0x8004ab18); a piece with no slot left is not made. It flies as a flight
(0x8004b580 for triangles, 0x8004b2b0 for quads):

- flags 0xc010700: kind 0, gravity 0x700, the world, a random spin axis;
- from its centre in the world, away from the point struck, at the
  caller's speed;
- 1000 ms;
- bounce 768 + (rand & 255), spin 3072 + (rand & 255);
- a spin axis of three rand bytes, normalised;
- the game's rand drawn six times a piece.

The port:

- `world::shards` cuts the faces.
- `Flights::shatter` launches the pieces with `Skin::Shard(slot)` and
  frees the slot when the flight ends.
- The game draws each piece as a one-face model, turned by the object's
  matrix and then by the flight's own angles.
- `Game::shatter(placement, speed)` breaks a placed object from its own
  place.

Callers ported:

- Level 15's ? boxes (0x8004d434): 100, with sound 114 and the burst.
- Level 6's breakable toys, template 1102 with template flag 1024, stood
  on (0x8005ac98): 100. The port played sound 173 at the player, but
  0x8005ac98 plays it at the toy (a1 = +0xf8). Fixed.
- Level 6's toy bits, 0x80054ae0: five kind 5, flags 0x10505, sprites
  421-425 by rand % 5, size and radius 600, speed rand % 400 (both from
  0x800c41d4), within 1024 of up, 1000 ms, bounce 2048, spin 3391, face
  0xc0800000, the toy's damage.
- Level 7's things Reptar takes, templates 401 and 795 (0x8005ac54): 200.

Test: level_15_mirrorland_is_won also checks that each box breaks into
1 to 10 pieces. Box template 1102 is 12 flat textured faces, radius 886.

**Resolved from earlier entries:**

- 29.1: the room lookup walks portals (worklog 122).
- 54.2, 110.2, 111.1: the signs' sparks (worklog 121).

**Still unknown:**

- The pieces have not been compared with the original's picture. On the
  oracle, Tommy put beside level 15's box 207 is either on top of it or
  falls about 6400 to the floor. Level 6's toy test lost the camera after
  the teleport.
- The piece's centre and radius (0x8004acbc, 0x8004aff8) are taken as the
  corners' mean and the farthest corner.
- The order of the four cuts after the first: the triangle's middle piece
  is taken last.
- The object's +0x86 is stood in for by the model's radius.
- The shade (+0x3a) the slot keeps is not used; the pieces keep the
  face's colours.
- The toy bits' seed 0x800c41d4 is assumed 48732, like its neighbours;
  it has not been peeked.
