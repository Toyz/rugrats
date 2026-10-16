---
number: 69
title: Positional sound: the falloff, the pan and Mono
date: 2026-10-15
area: audio
files: crates/rugrats/src/world/game/mod.rs crates/rugrats/src/audio/sound.rs crates/rugrats/src/app/mod.rs docs/engine/front-end.md
resolves: 60.1
started: 2026-10-15T18:11:55-06:00
took: 3m
---

# 69. Positional sound: the falloff, the pan and Mono

World sounds now follow 0x8006f644 (docs/engine/front-end.md, The positional sounds). Before, the port faded the player's sounds linearly to 16384 from the eye, with no pan, and the Audio menu's Mono did nothing.

- **Game::play_at(sound, pos):**
  - The distance from the camera is dx^2/16384 + dy^2/4096 + dz^2/16384.
  - The falloff (0x8006fd68, table 0x800b8024: 128, 16384, 127) is capped by the effects setting.
  - Unless Mono, it is split by 0x8006fc40: about a third lost from behind, and the far side up to three quarters.
- **Effect** is { sound, at: Option<(left, right)> }. A world sound carries its own volumes; the rest play at the effects setting, as before.
- **Spu::effect** takes (left, right) like SsUtKeyOn's voll and volr, on top of the tone's and program's volume and pan.
- **The player's sounds** (take-off, landing, steps) go through play_at.
- **Test:** world::game::tests::positional_falloff (the table's steps).

**Still unknown:**
- Which other sounds the original plays positionally (NPCs, doors, objects). The port still plays those at full volume, centred.
- The bearing (0x80021590) is computed in floating point here, not from the original's arctangent table.
