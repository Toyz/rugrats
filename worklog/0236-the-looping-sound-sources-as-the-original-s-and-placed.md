---
number: 236
title: The looping sound sources as the original's, and placed sounds panned to the right side
date: 2026-10-22
area: audio
files: crates/rugrats/src/world/game/sources.rs, crates/rugrats/src/world/fixups.rs, crates/rugrats/src/gte.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/app/mod.rs, crates/rugrats/src/audio/sound.rs, crates/rugrats/examples/npc_check.rs, docs/engine/sound.md, Cargo.lock
started: 2026-10-19T01:40:54-06:00
took: 3h 30m
---

# 236. The looping sound sources as the original's, and placed sounds panned to the right side

The looping sound sources now play as the original's, and every placed sound now pans to the correct side: the port had them mirrored.

**retro_rt.** Approved and pushed (retro_rt worklog 19, c018dec): `Spu::key_on_voice`, `set_voice_volume`, `key_off`, `voice_on`. rugrats' lock moves to it.

**The sources** (`world::game::sources`, sound.md "Sound sources"):
- **Records:** 0x800e3218, 40 bytes each.
- **Starts:**
  - the loader's LP kind 3 records (new: the sound the template word's high half, near and far the rotation's first two words, the volume its third's low half), in LP order with the object set-up's;
  - template 1178's, whose sound is drawn rand % 3 at load;
  - level 4's start adds one; level 10's empties the list and adds three.
- **Sounds:** the level's table 0x800b847c.
- **Each frame after the draw** (0x8006f88c), each source is heard through its own falloff (0x8006fd68 with its record):
  - keyed on a free voice of 15, 16 and 17 (0x800c4be8 = 7);
  - turned (SsUtSetVVol);
  - keyed off out of hearing.
- **Leaving the level** silences them (0x8006f828 from 0x80094bfc).

In level 3 the original runs 15 sources where the port, without the kind 3 records, had 5.

**The pan.**
- **The facing.** 0x8006f88c and 0x8006f4c0 take the heading the camera faces as the bearing toward the eye plus the matrix's third column (0x80010b54 copies it transposed). The port had used the third row, so every placed sound was panned to the wrong side.
- **The bearing** is the original's integer one: `Trig::bearing`, 0x80021590, floor square root then asin or acos.
- **Checked** with npc_check SRCDBG=1, which hooks 0x800b0788 and compares the voices' turns each frame. In level 3 by the 614 source they are the same wherever the cameras are the same (70 frames); the rest follow the camera drifting after the tool's put.

**Also:**
- `Game::state_block` starts from the progress passed in, so a game outside the app hears at the default volumes (sources at volume 0 were never keyed);
- npc_check syncs the sources' voices and on flags, and compares their key-ons with the other sounds.

**Still unknown:**
- SsUtSetVVol's exact law against a tone's volume (retro_rt scales the tone's base).
- The camera after npc_check's put in level 3 (eye height off by about 270 from frame 5).
