---
title: In-engine cinematics
status: partial
discs: US
covers: US SLUS_006.50:0x8007424c, US SLUS_006.50:0x80073810, US SLUS_006.50:0x8007412c, US SLUS_006.50:0x80019da4, US SLUS_006.50:0x80019c88, US SLUS_006.50:0x800b8bd0, US /DATA/CINEMAT/OPEN.BIN, US SLUS_006.50:0x80073b08, US SLUS_006.50:0x80073c34, US SLUS_006.50:0x80073f08, US SLUS_006.50:0x8001e0f4, US SLUS_006.50:0x8001d218
worklog: 7, 14, 15
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
  u16  unknown_02
  s32  camera position[3]
  u32  a              bits 0-12 q0, 13-25 q1 (signed), 26-31 actors
  u32  b              bits 0-12 q2, 13-25 q3 (signed), 26-31 poses
  actor[actors]       24 bytes (0x80073c34)
    u32  bits 0-12 q0, 13-25 q1, 26-29 animation, 30-31 state
    u32  bits 0-12 q2, 13-25 q3
    s32  position[3]
    s16  id, u16 unknown_16
  pose[poses]         16 bytes and 8 a key (0x80073f08)
    s32  position[3]
    u32  bits 0-15 id, 16-23 keys, 24-27 animation, 28-29 state
    key[keys]
      u32  bits 0-12 q0, 13-25 q1, 26-31 channel
      u32  bits 0-12 q2, 13-25 q3
  0xdddddddd          as the first word: the stream's end
```

Quaternions are (x, y, z, w) = (q0, q1, q2, q3), at any scale. Ids are the
object registry's, from the database's LP file ([lp](../formats/lp.md)).

## Timing and sound

The stream's data sectors arrive every 5 sectors at double speed (OPEN.BIN:
frame 1 at sector 2, frame 1412 at sector 7056), so 30 frames a second;
the XA between them is filtered to file 1, channel 1 (cine 0 has no
channel byte in the table at 0x800b4290; one with a channel starts 2
sectors a channel in and its data sectors are read from +16). The CD
volume is the cine table's byte +0x0d (100 for the opening).

Each frame play_cine samples the pads after drawing (0x8001d3e8 calls
0x8001acc0), and stops when the first port's Triangle history reads "pressed"
(0x800cda4c & 3 == 1), or when the counter reaches the frame count. With
the entry's flag 0x01 clear it fades out over the last 13 frames (25 a
frame into 0x800c3f2c); the opening has it set.

The opening's one timed call (frames 1184 to 1413, 0x8008ea70) is the
title card: the logo object moved by the frame, and "Search For Reptar"
and "Loading" drawn through the text system (0x8002e144).

## Applying a frame

- The camera (0x80073b08): position to 0x800e5648, the quaternion through
  0x80020580 (the transpose of the parts' 0x8001fc48) to the camera matrix
  at 0x800e5428, and 0x8001e0f4 makes the view matrix (0x800e5448) from it
  by scaling its x row by (s16 0x800c4a14) / 320 and its y row by (s16
  0x800c4a16) / 240: 512 and 240 here, so x by 1.6. A point p is seen at
  view (p - camera) (0x8001eb4c), projected with H 178 and no offset
  onto the 512 x 240 screen.
- An actor (0x80073c34) is a single-model object: its rotation goes
  through 0x8001f9fc to object +0x04, its position to object +0x18. Ids
  510 and 511 are special: the actor is moved 0x2000 down with +0x24 set
  to 1024, and object 602 (601 for 511) takes the record's rotation and
  position instead, its +0x24 growing by 50 a frame to 1024 once
  0x800c43d0 passes 50 (a swap, or a fade in; not yet seen).
- A pose (0x80073f08) is a hierarchy object: position to object +0x18,
  angles at +0x24 cleared, and each key's quaternion to the part its
  channel maps to (hierarchy +0x14 + channel, 0xff for none).

The backdrop is a fill of (217, 175, 141): play_cine's setup (0x8007475c)
calls `set_backdrop(view, r, g, b)` (0x8001d218, view 3 at 0x800cdf18)
with it, which turns on the view's DRAWENV isbg.

## Measured

`rugrats-viewer 22 --cine 199` (rooms, LP props, stream actors and poses,
this camera) against the oracle's picture when the counter at 0x800c4c34
reads 200 (oracle frame 2039; the picture shows the frame before): the
camera position equals the oracle's, every pose's keys equal the
oracle's part quaternions, and rooms, props and characters line up. The
original's wall shading (depth cue and lighting) is not drawn yet.

## Unknown

- `unknown_02` (1380, the setup's size, in every OPEN.BIN frame), the
  actors' and poses' animation and state fields (0x80074ec0, 0x80075304),
  the setup block, the rest of the 16-byte table entries.
- Whether Triangle really skips: the oracle has no input during the
  cinematic (see [input](input.md)).
