---
number: 115
title: Collectibles sparkle; Reptar's sounds in the world
date: 2026-10-15
area: render
files: crates/rugrats/src/world/flights.rs crates/rugrats/src/world/game/goodies.rs crates/rugrats/src/levels/reptar_2010.rs crates/rugrats/tests/flights.rs
started: 2026-10-15T23:57:27-06:00
took: 2m
---

# 115. Collectibles sparkle; Reptar's sounds in the world

Two of 114's and 112's leftovers.

**Collectible sparks** (0x800514b0, every level). Taking a collectible
throws three kind-5 flights from where it lay:

- flags 0x305, half-size 200, face 0xd2800000;
- 500 ms, bounce 2048;
- speed rand mod 100 from a seed set afresh to 0xb0a5 each time;
- directions all round about up (0x80088bac(up, 0, 4096), the shared
  cone seed).

The sprite is by template: 835 TEXPOS + 3552 (296), 863 and 1017 359,
87 362, otherwise TEXPOS + 3540 (295). `Flights::sparkle` builds them
and the goodies' touch calls it. Test `a_collectible_sparkles`.

**Reptar's sounds** play where they happen now, which 112 left while
reptar_2010.rs was with the effects work:

- the building break sounds 90-96 and 91 / 92 at the building
  (0x8004d7c4);
- the rumble landing's 93 at him (0x8005cba8).

**Still unknown:**
- the cone is taken about world up; the original turns it by the object's own matrix (+0x04), which only matters for tilted collectibles
