---
number: 210
title: Levels 9, 10 and 15's uses are ACTION's touched object, a character in the nearby list by its size
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/levels/mirrorland.rs, crates/rugrats/src/levels/chuckies_glasses.rs, crates/rugrats/src/levels/mr_friend.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs
started: 2026-10-17T20:33:39-06:00
took: 26m
---

# 210. Levels 9, 10 and 15's uses are ACTION's touched object, a character in the nearby list by its size

The remaining ACTION uses checked by distance are ACTION's touched object now, as the original's (0x800631d8, then the object's +0x54).

**What ACTION takes.** 0x800631d8 walks the player's nearby list: things, characters and doors, and the other placed objects with a model. It takes the nearest within reach and within a quarter turn of the heading. `Game::touched_any` now covers the last kind too (`Touched::Placement`).

**The nearby list (0x8006abbc).** An object is in it when the middle of its box +0x78..+0x84 (turned, plus its place) lies within its +0x86 and the player's cylinder (3 x his box's larger half-extent), and the heights overlap. A character's +0x86 is 8192 (its bound cube 1773), so near enough any character is in the list, and the reach (the body's depth and 1800) and the quarter turn decide. The port took a character by its hierarchy box's corner (about 600), which lost Mr. Friend at 1400.

**Oracle.** `level_9_kick_as_the_original`: Tommy put short of Mr. Friend, his places the original's, Triangle once:
- at 600, 1000 and 1400 facing him, the kick (state 12) on the original's frame;
- at 2200 and at 600 facing away, no kick on either.

**The uses moved over.**
- Level 15's boxes (ids 207-212, 0x8004d434): within 1300 across before, any heading.
- Level 10's tag (0x800831a4): a hiding baby within 900 before.
- Level 9's kick (state 12, 0x800833a4): Mr. Friend within 1100 before.

The proximity thinks that are distances in the original stay as they are:
- the hub's signs and training's sparkles (0x80049eec within 2500 in the player's room);
- level 5's sliding doors (5000);
- level 7's taken-by-touch (a touch, not ACTION).

**The play routes, under the original's rules.**
- Level 15 put Tommy beside each box facing +z; ACTION takes nothing behind a quarter turn, so the route now faces the box.
- Level 9's route pressed Triangle from under 1000.
  - With a ball nearer than Mr. Friend, the ball is the touched object and the kick does nothing.
  - Run into, the ball stands Tommy (it can be picked up, worklog 209).
  - The route now takes up a ball it touches and throws it at him; level 9's throw aims at +0x1f4.
- The tests' `steer` gave no keys when no stick direction turned by at least the turn needed, so a player stuck facing away never turned. It now falls back to the stick direction nearest the wanted heading.

The play suite plays every level through, the whole game included.

**Still unknown:**
- Levels 10's and 15's uses against the oracle at their edges, and a ball nearer than Mr. Friend taking the press in the oracle (the code says it does).
- Level 7's taken-by-touch (0x8005ab60) against the collision touch rather than its 1400 across.
