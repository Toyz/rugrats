---
number: 183
title: Throws fly as the original's flights
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T01:41:12-06:00
took: 22m
---

# 183. Throws fly as the original's flights

Throws and lobs now fly as the original's flights. The pick-up script (`hub_pick_up_as_the_original`) holds level 1's thrown toy to the original's on all 77 frames from the release until it settles: two bounces off toys 431 and 430, floor bounces, then rest.

The flight (`Game::fly_exact`, with the record in `Thing::flown`: position, radius, bounce, whether it meets objects), as 0x80087c5c and 0x80087594 have it:

- Gravity 13 (flags 0x700) by 0x8004fbb4: p += ((a dt dt >> 1) + v dt) >> 6, then v += a dt.
- The step from the old position to the new one carried on by the radius (at least 10) along the velocity's unit (0x8007652c), met by 0x80086668. Objects come first (0x80086340 -> 0x8006a824): the nearest crossing past the start by the exact dispatch (0x8006a714), skipping type flag 8. The world's segment (0x8002cd7c) comes second and wins only when its point is nearer the start on all three axes. The targets are the frame's solids (things at rest and placed objects), the characters, and the player by its hierarchy's box.
- A thing met is touched (template +0x0a's kinds); a character is struck (its line, `Strike`); the player is hurt by the thing's +0x6d. The original keeps no last-hit filter: a can thrown at Mr. Friend facing Tommy goes back and forth between them.
- A surface met head on (0x8007666c's acos over 1024) sends it back off the radius from the point. It stops when slower than 95 + 382 with something within its radius below (0x80087414: the same meeting straight down, things included); otherwise it goes back along the reflection at (|v| - 95) bounce >> 12. A glancing meeting leaves it on its way. When it stops, the flight ends (flag 0x1000000) and the thing rests where the flight left it.

Fixes the exact flight brought out:

- A thing taken up (grab, hand-over, rest) drops its flight record. A bottle caught in flight had kept its record and, knocked off later, resumed from the catch.
- The box toss's catch (0x80057518) needs the box in flight (+0x90) and within 1400 by 0x8001f57c's distance (dx + dz less half the smaller). The port's per-axis test also caught the box still in the thrower's hand, about 100 nearer since the carry became exact, and swapped bodies mid-throw.
- `level_9_a_can_bounces_back` now has Mr. Friend face Tommy. His box is turned by his placement's heading, and the can glanced off its diagonal face. The test checks the first two hits: him, then Tommy.
- The play bot leaves the stick alone while a throw is under way.

**Still unknown:**
- The put-down still slides by the port's rule. The original flies it by the same descriptor along (0, 0, 4096) at speed 10, after 0x80060810's wall test, and the level 6 box route depends on where it lands.
- Things whose template +0x30 is 1 fly by 0x80086ef0 (rolling things), still the port's approximation.
- Rooms: the original tests the start room's objects and only walks the rooms along the step if none is met there, clamping a start outside every room into the record's room. The port tests all solids at once.
- The flight's sounds (+0x88), the hit's +0x80 callback and the floor effect (+0x2a) are not played.
- The aimed throw (0x80061c98, level 9) still builds its direction in floats from the player to the target. The original turns (0, -384, 1024) by the angles toward the target when within 512 of the heading, makes it a unit, and sets y to -300.
