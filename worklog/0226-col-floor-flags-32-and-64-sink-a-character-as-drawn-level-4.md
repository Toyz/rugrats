---
number: 226
title: COL floor flags 32 and 64 sink a character as drawn: level 4's deep water
date: 2026-10-18
area: render
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/collide.rs, docs/levels/level-04.md
started: 2026-10-18T13:04:53-06:00
took: 5m
---

# 226. COL floor flags 32 and 64 sink a character as drawn: level 4's deep water

COL floor flags 32 and 64 are the sink depth: a character on such a tile is drawn 298 lower per step of depth and gets no shadow. Level 4's deep water has them, and the port now draws its waders sunk.

**The reader (0x8003205c).** The character draw takes the body's tile (+0x8c) unless it stands on an object (+0xf8). Its flags (COL +4) give depth (flags & 96) >> 5, and fp = 298 × depth.
- Each part's matrix translation y gets fp before its draw (0x8001eb4c) and loses it after. The parts as kept, which the collision tests read, are not sunk.
- With fp set, the shadow (0x8003414c) is skipped.

The only byte +4 tests in the code are of 1, 2, 4, 16 and 96. Nothing found reads 8.

**Where.** Only level 4 has such tiles: 54 of flags 33 (water and depth 1), the deep water around (-229000, 256, 64000). The oracle's room 40 floor in the level 4 doc carries the same 33.

**The port.**
- `Game::sink` gives the depth for a place and whether it stands on an object.
- `Player::triangles` and `npc::triangles` take it and draw each part that much lower. `Npc::drawn` keeps the unsunk pose, and the hand still places the carried thing from the parts put back.
- `collide::floor_flags` names every bit.

**Still unknown:**
- The characters' shadow (0x8003414c with 0x8006c118): the port draws none.
- Who, if anyone, reads floor flag 8 (levels 1, 12, 13 and 18 have it).
