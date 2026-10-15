---
title: STR movies
status: partial
discs: US
covers: US /DATA/OPEN/THQ_LOGO.STR, US /DATA/OPEN/NICKLOGO.STR, US /DATA/OPEN/LOGOXA.STR, US /DATA/OPEN/DEMO01.STR, US /DATA/OPEN/DEMO02.STR, US /DATA/OPEN/DEMO03.STR, US /DATA/OPEN/CREDITS.STR, US SLUS_006.50:0x800bae4c
worklog: 6
---

# STR movies

`/DATA/OPEN/*.STR`: 320 x 240 frames in real-time data sectors, each frame
split over several sectors, with an XA soundtrack (file 1, channel 1,
submode 0x64) interleaved about one sector in eight.

## A video sector

Form 1, submode 0x48 (data, real time). User data, little-endian:

```
u16  0x0160
u16  0x8001
u16  chunk        this sector's place in the frame, from 0
u16  chunks       sectors in the frame (9 in THQ_LOGO)
u32  frame        from 1
u32  unknown_0c   e.g. 0x428 for THQ_LOGO frame 1
u16  width        320
u16  height       240
u8   [12]         the frame's BS header, repeated in every chunk
u8   [2016]       the frame's BS bytes
```

A frame is its chunks' 2016-byte parts in order; it is a [BS](bs.md)
picture. THQ_LOGO's frames are BS version 3, NICKLOGO's version 2. The
original decodes them at 24 bits (MDEC command 0x32xxxxxx: 24-bit,
unsigned) and shows them in the GPU's 24-bit display mode.

## The movie table

`movie_play(n, skippable)` (0x8008b448) takes movie `n` from a table at
0x800bae4c, 8 bytes each: `u32 frames, u32 lba`.

| n | frames | file | played by |
| ---: | ---: | --- | --- |
| 0 | 101 | `THQ_LOGO.STR` | main, 0x8008b230 |
| 1 | 100 | `NICKLOGO.STR` | main, 0x8008b27c |
| 2 | 150 | `LOGOXA.STR` | main, 0x8008b0b4 |
| 3 | 150 | `DEMO01.STR` | front end, 0x8008b100 |
| 4 | 400 | `DEMO02.STR` | front end, 0x8008b14c |
| 5 | 550 | `DEMO03.STR` | front end, 0x8008b198 |
| 6 | 1875 | `CREDITS.STR` | a database, 0x8008b1e4 |

## Playback

The player streams with libcd's St* calls at double speed and shows each
frame as it completes, then waits two vertical blanks; it ends after
`frames` frames. When skippable it asks the input module for a button
pressed this frame (`0x8001baf8(pads, 0)`) once a frame and stops if there
is one.

## Unknown

- `unknown_0c` in the sector header.
- The exact pacing: the original shows a frame when its sectors are in; the
  port schedules frame k at its last sector's time at 150 sectors a second.
- The `...B.STR` files (640 x 192, BS version 2, submode 0x42): no movie
  table entry names them; see [the B movies](../cut/b-movies.md).
