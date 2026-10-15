---
number: 26
title: Music and menu effects: a software SPU playing libsnd's songs
date: 2026-10-15
area: audio
files: crates/rugrats/src/sound.rs, crates/rugrats-data/src/sep.rs, crates/rugrats/src/main.rs, crates/rugrats/src/lib.rs, crates/rugrats-data/src/lib.rs, crates/rugrats/examples/song_wav.rs, docs/engine/sound.md, docs/README.md
started: 2026-10-15T01:02:31-06:00
took: 17m
---

# 26. Music and menu effects: a software SPU playing libsnd's songs

The title had no music. The game's music is libsnd: SEP songs played
through VAB banks on the SPU.

What the original does:

- A song set (0x8006e96c) names a music bank, an effects bank and a SEP.
  The menu's set is at 0x800b8030: MENU.VB/VH, banks 151/152, and
  RUGRATS.SEP with 5 songs.
- MidiPlaySong (0x8006eac4) plays a song. The title plays song 0 forever.
- The song volume (0x8006e74c) is the music setting, 90 at boot (read
  from the oracle's 0x800c5684).
- Effects (0x8006f26c) key on program, tone and note from the 180-entry
  table at 0x800b84fc. They play at the effects setting, 100, on voices
  17-23 in turn.

The port:

- `rugrats_data::sep` reads SEP files.
- `rugrats::sound::Spu` plays them:
  - 24 voices with the SPU's ADSR (psx-spx rates) and rrt-kit's Gaussian
    interpolation.
  - A sequencer for note on/off, program, volume, pan, expression, pitch
    bend, tempo, libsnd's NRPN loop marks, and looping at the track end.
  - Effects keyed on the same way the game does.
- The audio device now plays `Mix`: the XA soundtrack plus the SPU.
- The title loads the menu set and plays song 0. Its menu sounds go to
  `Spu::effect`, and leaving for the demo stops the song.

Checks: `song_wav` renders song 0 for 30 s at peak 21990 and RMS 2859,
with no clipping. Before the main volume (0x3fff) and the >> 15 voice
volume were applied, it clipped.

**Still unknown:**

- How close the volume law, pitch and envelopes are to libsnd's. Nothing
  compares them against the original yet: the oracle has no SPU.
- Reverb, vibrato, portamento.
