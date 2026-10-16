---
number: 75
title: The sound engine moves to retro_rt's libsnd
date: 2026-10-15
area: audio, tooling
files: Cargo.lock crates/rugrats/src/audio/sound.rs crates/rugrats-data/src/lib.rs crates/rugrats-re/src/main.rs crates/rugrats/tests/sound.rs docs/engine/sound.md
started: 2026-10-15T20:13:32-06:00
took: 7m
---

# 75. The sound engine moves to retro_rt's libsnd

The generic half of the port's sound now lives in retro_rt as rrt::kit::libsnd (retro_rt worklog 16, commit 4a945de, pushed as 3eb434d), and the port is pinned to it. That includes the bank fix from 72, which is now retro_rt's.

- **Moved to retro_rt:** rugrats_data::vab and sep (deleted here), and from audio::sound the envelope, voices, sequencer, volume law, slot volumes and fades, effect key-on and mix. The port's duplicate ADPCM decoder is gone too; rugrats-re uses rrt-kit's, and keeps its own small WAV writer.
- **Kept in the port (audio::sound):**
  - the song sets (level_set, MENU_SET, Loaded, load_bank by the game's file numbers);
  - the effect table, and effects on voices 17-23;
  - Spu, Rugrats' policy over libsnd::Spu with two slots. play stops all; switch_song is 0x8006efcc's rule: the song ids per slot, the old slot faded out and stopped, the new one faded in, nothing for a song already playing or fading.
  - Mix (the XA soundtrack under the SPU).
- **Checks:**
  - The menu's song 0 renders as before the move (song_wav: peak 15581, RMS 2026).
  - tests/sound.rs every_bank_parses runs every VH on the disc through libsnd.
  - The crossfade test, the boot tests and the whole-game test pass: 50 tests, no warnings in the workspace.

**Still unknown:**
- nothing
