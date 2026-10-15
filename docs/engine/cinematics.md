---
title: In-engine cinematics
status: partial
discs: US
covers: US SLUS_006.50:0x8007424c, US SLUS_006.50:0x80073810, US SLUS_006.50:0x8007412c, US SLUS_006.50:0x80019da4, US SLUS_006.50:0x80019c88, US SLUS_006.50:0x800b8bd0, US /DATA/CINEMAT/OPEN.BIN
worklog: 7
---

# In-engine cinematics

The story scenes are rendered by the engine from a database's models and
rooms, driven frame by frame by a stream of small data sectors read from
`/DATA/CINEMAT` with the XA dialogue and music interleaved.

## play_cine (0x8007424c, cinemat.c)

`play_cine(n)` takes `n < 93` ("Invalid cine number %d") from a table at
0x800b8bd0, 16 bytes an entry (byte 7 flags: 0x40, 0x20, 0x08, 0x01 seen
tested; byte 6 a count of timed calls with a pointer at +8; byte 12 a
post-scene flag). It starts the stream (`0x80073810`), then each frame:
waits for the next data sector (`0x8007412c`), updates the camera
(0x80088c80, 0x80088e0c), renders the scene (`render`, 0x80030b80), runs
the entry's timed calls whose frame window contains the counter, and
flips. It ends when the stream's frame counter (0x800c4c34) reaches the
header's frame count, or on an end-marker sector.

The opening (`Opening Cine Init`, 0x8008f078) sets up database 22 (its
textures, rooms, collision, models, hierarchy and loops) and the menu's
eye and mouth TIMs before it.

## The stream (0x80019da4)

Started with CdControlB(Setmode 0xc8), CdControlB(ReadS, the table's
position), then Setmode 0xe9 (double speed, whole sector, XA filter,
XA-ADPCM) and Setfilter(1, 1). The ready callback `0x80019c88` copies each
data sector (4 words of header and subheader, then 511) into one of eight
2060-byte buffers at 0x800c7510 and points 0x800c3ee0 at its data
(+12, or +16 when the table entry names a channel byte to check).

## Sector contents

Little-endian.

```
first sector (header)
  u32  magic          0xf00dbabe (or 0xbabed00d)
  u16  version        must equal 0x800c43cc (OPEN.BIN: 3)
  u16  0xdead
  u16  frames         OPEN.BIN: 1412
  u16  setup_size     OPEN.BIN: 1380
  u8   setup[setup_size]   copied to a fresh block (0x800c43d8)
each later data sector: one frame
  u16  frame          from 1
  ...                 camera and actor values; +0x10 and +0x14 carry
                      counts in their top 6 bits (0x80073c34, 0x80073f08 per item)
  0xdddddddd          as the first word: the stream's end
```

## Unknown

- The per-frame record's fields, the setup block, and the 16-byte table entries.
- Why the oracle's opening renders nothing: the stream arrives, but the
  camera is "outside room" every frame and every room yields 0 polygons.
