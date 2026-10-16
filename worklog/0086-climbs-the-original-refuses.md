---
number: 86
title: Climbs the original refuses
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/player.rs crates/rugrats/src/world/game/mod.rs
started: 2026-10-15T21:20:04-06:00
took: 1m
---

# 86. Climbs the original refuses

0x80058da4, the climb test, returns 0 before looking at the ledge in two cases. The port allowed both climbs.

- **Levels 1 and 12 (0x800c5678), the player in room 4** (+0x3b): every climb is refused (0x80058e04).
- **Level 10, a ledge of type 14** (0x80056e44's class, 906-1033 above the feet; docs/levels/level-10.md): refused (0x80058e14). Chuckie climbs at most 905 there.

The port: Player::climb_off and climb_max, which Game sets each frame from the level and room before the player moves. The climb's reach is climb_max (521 carrying, as before, capped by it), and walking into a ledge with climb_off stops as at a wall. The play tests still pass, Chuckie's Glasses included.

**Still unknown:**
- No test stands at a 906-1033 ledge in level 10 or in room 4 of the hub; the rule is from the code, not watched in the oracle.
