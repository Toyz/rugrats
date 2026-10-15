---
title: Music and sound effects
status: partial
discs: US
covers: US SLUS_006.50:0x8006e96c, US SLUS_006.50:0x8006eac4, US SLUS_006.50:0x8006e74c, US SLUS_006.50:0x8006f26c, US SLUS_006.50:0x800b8030, US SLUS_006.50:0x800b84fc, US /DATA/SOUND/SEPS/RUGRATS.SEP, US /DATA/SOUND/VAB/MENU.VH
worklog: 26
---

# Music and sound effects

The game's music is libsnd sequences (SEP) played through VAB banks on
the SPU. Sound effects are key-ons on the same voices. Speech and the
cinematics' sound are XA ([cinematics](cinematics.md)).

## Song sets (0x8006e96c)

A song set is a small struct the opener loads and keeps at 0x800c4338:

| offset | what |
|---|---|
| +0x00 | s16 music bank body (VB); its header (VH) is the next file |
| +0x02 | s16 effects bank body; its header the next file |
| +0x04 | s16 SEP file |
| +0x06 | u8 songs in the SEP |
| +0x08 | u8 the SEP's access number (SsSepOpen) |
| +0x0a | s8 the song playing, -1 none |

- The menu's set is at 0x800b8030: music 109/110 (MENU.VB/VH), effects
  151/152, SEP 230 (RUGRATS.SEP), 5 songs.
- 0x800b8180: music 133/134, effects 151/152, SEP 223 (INTRO.SEP), 1 song.
- A level's set is picked by database (0x80018598) and opened by the level
  setup (0x80017c0c).

## Playing a song (MidiPlaySong, 0x8006eac4)

`MidiPlaySong(song, loops)` stops the song playing (0x8006ebb4), sets the
SEP song's volume (0x800a91a0) and plays it (0x800a84e0, mode 1; loops 0
is forever).

- The volume (0x8006e74c) is the settings' music volume (s32 0x800c5684,
  90 at boot). Some levels raise it for one song: game mode 2 song 2 by
  30, mode 5 song 2 by 20, mode 18 by 10. It is clamped to 127.
- The title's loop (0x8003d884) plays song 0.

## Effects (0x8006f26c)

Effect n (under 180) reads 3 bytes at 0x800b84fc + 3n: program, tone,
note. It keys that tone on in the effects bank (0x800a99a8, SsUtKeyOn)
at the effects volume (s16 0x800c5688, 100 at boot) on both sides. The
voice is taken in turn from 17 to 23 (0x800c434c).

## SEP

See `rugrats_data::sep`:

- "pQES", then u16 version.
- Then the songs back to back, each with: u16 id, u16 resolution,
  u24 tempo, u16 rhythm, u32 length (all big-endian), then MIDI events
  with running status, ending FF 2F 00.
- libsnd marks loops with NRPN 99 = 20 (start, data entry the count) and
  99 = 30 (end).

## Port

- `rugrats::sound::Spu` is a software SPU: 24 voices, the SPU's ADSR,
  Gaussian interpolation (rrt-kit), and libsnd's sequencer reading the SEP
  events.
- Voice volumes multiply velocity, tone, program, channel (CC 7),
  expression (CC 11), song and bank master volumes, each out of 127.
- Pans add up around 64.
- `Mix` puts the SPU's voices over the XA soundtrack for the audio device.
- `song_wav` renders a song offline. Song 0 at 90: peak 21990, RMS 2859.

## Unknown

- libsnd's exact volume and pan law and its pitch formula (tone shift
  taken as 1/128 semitone).
- Reverb (the SPU's reverb is not modelled).
- Vibrato and portamento from the tone attributes.
