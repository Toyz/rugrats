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

## A room's song (0x8006efcc)

RoomChange (0x80077a54) calls 0x8006efcc(_, song, 30) for room flag 128
with +0x64 of 1-4. The set's record keeps the song playing (+0xa) and
one fading out (+0xb), with a fade timer +0x10:

- The song is +0xa or +0xb: nothing.
- No song playing (+0xa -1, or it has ended, 0x800a5e88): MidiPlaySong
  at once.
- A fade under way (+0x10 not 0): the fading song stops (0x800a8d00),
  +0xb = +0xa, +0xa = the song, played and set to volume 0 (0x800a91a0),
  then SsSeqSetCrescendo (0x800a59f4) to the music volume over 30.
- Else: +0xb = +0xa, +0x10 = 30 * 50, SsSeqSetDecrescendo (0x800a5bbc)
  on +0xb from its volume over 30, and the new song at 0 with the
  crescendo as above. So the two overlap: a crossfade.

0x8006ef0c (from the level loop 0x80018ad0) and 0x8006ef64 (0x8007424c)
fade the playing song out and back in over 60.

## Effects (0x8006f26c)

Effect n (under 180) reads 3 bytes at 0x800b84fc + 3n: program, tone,
note. It keys that tone on in the effects bank (0x800a99a8, SsUtKeyOn)
at the effects volume (s16 0x800c5688, 100 at boot) on both sides. The
voice is taken in turn from 17 to 23 (0x800c434c).

## SEP

See `rrt::kit::libsnd::sep`:

- "pQES", then u16 version.
- Then the songs back to back, each with: u16 id, u16 resolution,
  u24 tempo, u16 rhythm, u32 length (all big-endian), then MIDI events
  with running status, ending FF 2F 00.
- libsnd marks loops with NRPN 99 = 20 (start, data entry the count) and
  99 = 30 (end).

## Port

- The banks, songs, voices, envelopes, sequencer, slot volumes and fades
  are retro_rt's `rrt::kit::libsnd` (rrt-kit.md, moved there from this
  port): 24 voices, the SPU's ADSR, Gaussian interpolation, and libsnd's
  sequencer reading the SEP events.
- `rugrats::audio::sound` keeps the game's side: the song sets and their
  files, the effect table, effects on voices 17-23, and the room switch
  (0x8006efcc) over two of libsnd's song slots with the songs' ids.
- Voice volumes multiply velocity, tone, program, channel (CC 7),
  expression (CC 11) and bank master volumes, each out of 127; a slot's
  volume is applied as the voices mix, so SsSeqSetVol and the fades
  change notes already sounding.
- libsnd's ticks are 60 a second: the sound init (0x8006e7e8) sets
  SsSetTickMode(0x1000), SS_NOTICK, and the VSync callback (0x800192b4)
  calls SsSeqCalledTbyT (0x800a48b8) every vblank. The switch's 30 is half
  a second.
- The sequencer has 14 voices (SsSetReservedVoice(14), 0x800ad588);
  effects key on 17-23 directly.
- No tone in the game's 1779 has vibrato or portamento, and one has the
  reverb bit, so libsnd's unmodelled vibrato, portamento and reverb do
  not change what Rugrats plays.
- Pans add up around 64.
- `Mix` puts the SPU's voices over the XA soundtrack for the audio device.
- `song_wav` renders a song offline. Song 0 at 90: peak 21990, RMS 2859.

## Unknown

- libsnd's exact volume and pan law and its pitch formula (tone shift
  taken as 1/128 semitone).
- Reverb (the SPU's reverb is not modelled).
- Vibrato and portamento from the tone attributes.
