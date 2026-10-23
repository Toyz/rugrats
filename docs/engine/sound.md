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

A sound at a place (0x8006f644) is heard from the camera: dx^2 / 16384
+ dy^2 / 4096 + dz^2 / 16384 through the falloff 0x800b8008 (full within
128, none past 16384, 127 at most). It is panned by the heading from the
eye to it (0x80021590: asin of x / length where |x| < |z|, else acos of
z / length) less the heading the camera faces. That facing is the
heading toward the eye plus the camera matrix's third column:
0x80010b54 copies the matrix transposed. With the third row the pan
comes out mirrored, as the port's did until now.

## Sound sources (0x8006fe04, 0x8006f88c)

A looping source is a 40-byte record at 0x800e3218, 0x800c434b of them:
+0x00 its sound, +0x04 its voice, +0x08 on, +0x0c its place, +0x1c and
+0x20 the distances it is full within and gone past, +0x24 its volume.
They are started by:
- the loader for an LP kind 3 record (0x800454dc at 0x80045b24: the
  sound the template word's high half, near and far the rotation's first
  two words, the volume the third's low half), in LP order;
- the object set-up for some templates (91 and 614 in level 3, 227 in
  16, 657 in 23, 1191, 920 and 535 in 28, 789 in 31, and 1178 with its
  sound rand % 3);
- level 4's start (one), and level 10's start, which empties the list
  first and adds three.

A source's sound is the level's table 0x800b847c[level] (s16 program,
tone, note) at its sound. Each frame (0x8006f88c, after the draw) each
source is heard as a placed sound is, but through its own falloff
(0x8006fd68 with its record; mono uses 0x800b8008):
- heard and on, it is keyed on a free voice (0x800c4be8: 7 at a level's
  start, so 15, 16 and 17; 0x800a99a8 with the voice) or turned
  (SsUtSetVVol, 0x800b0788);
- not heard, it is keyed off (0x800a9cf0) and its voice freed.

Leaving a level (0x80094bfc) silences them (0x8006f828). The port's
`world::game::sources` does this, through retro_rt's `Spu::key_on_voice`,
`set_voice_volume` and `key_off`. npc_check SRCDBG=1 compares the turns
each frame: the same where the cameras are.

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
- Reverb is silent: the SPU init (0x800add98, from the sound init) clears
  SPUCNT and the reverb volumes. The only paths to SpuSetReverb
  (0x800af2f8) and the reverb depth and type (0x800af4d8) are libsnd's
  SEQ handlers that SsSepOpen (0x800a5ec8) installs: CC 91, the NRPN
  data entries. The game's 33 SEPs send no CC 91; besides the loop marks
  (NRPN 20, 30) their only NRPN is 40, always with data 0.
- Pans add up around 64.
- `Mix` puts the SPU's voices over the XA soundtrack for the audio device.
- `song_wav` renders a song offline. Song 0 at 90: peak 21990, RMS 2859.

## Unknown

- libsnd's exact volume and pan law and its pitch formula (tone shift
  taken as 1/128 semitone).
- Reverb (the SPU's reverb is not modelled).
- Vibrato and portamento from the tone attributes.

## Around cinematics

A cinematic's table byte +7 (0x800b8bd0 + 16 n, read by 0x80074b80)
decides the song:

- PostCine in a level (0x80018ad0): with 32, the song fades down over 60
  ticks (0x8006ef0c, SsSeqSetDecrescendo). The screen fades over 127 dt
  unless 16 is set. The level is drawn as it stopped (0x8001886c, no
  update) under a full-screen POLY_F4 with draw mode 0x640 (subtract)
  whose grey 0x800c3f2c steps by 256 / (127 / dt) to 255 (0x8001cf2c,
  from the flip 0x8001d00c).
- At the start (0x8007424c), the case depends on the byte:
  - with neither 8 nor 32, the song stops (0x8006ee14: SsSeqStop on both
    slots, the effect voices keyed off);
  - with 32, it pauses (0x8006ec40);
  - with 8 alone, it plays on.
- At the end, with 32: SsSeqReplay (0x8006ed30), then SsSeqSetVol 0 and
  SsSeqSetCrescendo to the music setting over 60 ticks (0x8006ef64).

A stopped song stays stopped. The song saved at +9 has no reader. The
next room with a song (flag 128) starts one again.

| byte | cinematics |
|---|---|
| 0 (stop) | most |
| 1, 2, 0x10, 0x11 (stop) | 0, 3; 5, 10, 11, 15, 16, 19, 22, 36, 69-71, 78, 80, 86, 91; 52, 55, 58, 68; 39 |
| 0x20, 0x30, 0x31 (pause) | 13, 18, 38, 41-43, 75-77, 87-90; 34; 4 |
| 0x58 (play on) | 29-33, 81-84 (the golf cameras) |
