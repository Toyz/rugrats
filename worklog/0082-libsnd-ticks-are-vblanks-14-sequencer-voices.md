---
number: 82
title: libsnd ticks are vblanks; 14 sequencer voices
date: 2026-10-15
area: audio
files: Cargo.lock crates/rugrats/src/audio/sound.rs docs/engine/sound.md
resolves: 71.1
started: 2026-10-15T20:48:29-06:00
took: 4m
---

# 82. libsnd ticks are vblanks; 14 sequencer voices

Two of the sound's open points, from the sound init (0x8006e7e8) and the VSync callback, now in the port through retro_rt 8dd8b57 (its worklog 17).

- **The crossfade's length** (71's question): the init sets SsSetTickMode(0x1000) (0x800a8f58), SS_NOTICK. The game's VSync callback (0x800192b4) calls SsSeqCalledTbyT (0x800a48b8) every vblank, and that steps each SEQ's crescendo and decrescendo linearly (0x800a4b28). So a room's switch over 30 ticks is half a second, as the port had assumed; it is no longer a guess.
- **The sequencer's voices:** the init's SsSetReservedVoice(14) (0x800ad588) limits libsnd's allocator to voices 0-13; effects key on 17-23 directly. The port had let songs take 0-16. It now calls libsnd::Spu::set_reserved_voices(14), so a busy song steals among 14 voices, as on the PlayStation.
- **Vibrato, portamento and reverb**, which libsnd does not model, matter little here: none of the game's 1779 tones has vibrato or portamento, and one has the reverb bit.
- **Checks:** 64 tests pass; the menu's song 0 renders as before (peak 15581, RMS 2026).

**Still unknown:**
- Whether the game turns SPU reverb on at all (the one tone with the bit would be the only one it touched).
