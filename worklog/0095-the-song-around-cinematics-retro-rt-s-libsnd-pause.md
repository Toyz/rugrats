---
number: 95
title: The song around cinematics; retro_rt's libsnd pause
date: 2026-10-15
area: audio
files: crates/rugrats/src/app/flow.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/audio/sound.rs crates/rugrats/src/media/cine.rs crates/rugrats/tests/sound.rs docs/engine/sound.md Cargo.lock
resolves: 71
started: 2026-10-15T22:30:16-06:00
took: 5m
---

# 95. The song around cinematics; retro_rt's libsnd pause

71 left the fades around cinematics. The port stopped the song for
every cinematic. The original decides by the cinematic's table byte +7
(0x800b8bd0 + 16 n, 0x80074b80), in 0x8007424c:

- neither 8 nor 32: stop (0x8006ee14);
- 32: pause (0x8006ec40, SsSeqPause), and at the end replay
  (0x8006ed30) at volume 0 and crescendo to the setting over 60 ticks
  (0x8006ef64);
- 8 alone: the song plays on.

Posted inside a level (0x80018ad0), a 32 also fades the song down over
60 ticks first (0x8006ef0c) while the screen fades for 127 dt. Pause
applies to 4, 13, 18, 34 (Golf Intro), 38, 41-43, 75-77 and 87-90. Play
on applies to the golf cameras 29-33 and 81-84. A stopped song has no
way back but the next room with a song; the saved id at +9 is never
read. The port already behaved that way.

retro_rt's libsnd had no pause, so it gained `Spu::pause`, `replay` and
`paused` (retro_rt 0a13f7f, worklog 18 there, test
`a_song_pauses_and_replays`). With the user's OK it is pushed and the
port is pinned to 7e0b6d4.

In the port:

- `media::cine::music_flags` reads the byte;
- the app's `cine_song_start` / `cine_song_end` stop, pause or leave
  the song, and resume a paused one with the fade;
- `audio::sound::Spu` gains `pause_song` / `resume_song`.

The test `cinematics_keep_pause_or_stop_the_song` checks 29, 34 and 64.

**Still unknown:**
- the screen's 127 dt fade before an in-level cinematic (0x800c3f2c) is not ported; with it the 60-tick fade-out would be heard for its 0.4 s before the pause
- whether SsSeqPause cuts the notes or releases them
