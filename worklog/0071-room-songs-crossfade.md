---
number: 71
title: Room songs crossfade
date: 2026-10-15
area: audio
files: crates/rugrats/src/audio/sound.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/app/play.rs crates/rugrats/src/app/flow.rs crates/rugrats/src/world/game/rooms.rs crates/rugrats/src/world/game/mod.rs docs/engine/sound.md
resolves: 29.2
started: 2026-10-15T18:16:08-06:00
took: 4m
---

# 71. Room songs crossfade

A room's song now crossfades as 0x8006efcc does (docs/engine/sound.md, A room's song). Before, the port switched songs at once.

- **Spu** keeps two song slots: the one playing and one fading out. Each has a live volume and a fade.
  - switch_song(song, id, volume, 30): the playing song fades from its volume to 0 while the new one fades from 0 to the music volume, both over 30 libsnd ticks. The old one stops at 0.
  - Asking for the song playing, or the one fading out, does nothing. A switch during a fade drops the fading song first.
- **Song volume** is now applied as the voices mix, not baked in at note-on. The fades and SsSeqSetVol (the Audio menu's live music volume) also change notes already sounding.
- **Game** has room_song for RoomChange's switch, apart from song, which the level scripts start at once (MidiPlaySong). Only 0x80077a54 calls 0x8006efcc.
- **Test** (audio::sound::tests::a_room_switch_crossfades, the menu set's songs 0 and 1):
  - Halfway, both are near 45 of 90.
  - Asking again for the fading song changes nothing.
  - At the end, only the new song is left, at 90.

**Still unknown:**
- libsnd's tick rate here (SsSetTickMode); 60 a second is a guess, so the fade's length may differ.
- 0x8006ef0c / 0x8006ef64's fades around the level loop and 0x8007424c are not ported.
