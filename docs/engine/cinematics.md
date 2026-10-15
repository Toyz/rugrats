---
title: In-engine cinematics
status: partial
discs: US
covers: US SLUS_006.50:0x8007424c, US SLUS_006.50:0x80073810, US SLUS_006.50:0x8007412c, US SLUS_006.50:0x80019da4, US SLUS_006.50:0x80019c88, US SLUS_006.50:0x800b8bd0, US /DATA/CINEMAT/OPEN.BIN, US SLUS_006.50:0x80073b08, US SLUS_006.50:0x80073c34, US SLUS_006.50:0x80073f08, US SLUS_006.50:0x8001e0f4, US SLUS_006.50:0x8001d218, US SLUS_006.50:0x80074b9c, US SLUS_006.50:0x80074ec0, US SLUS_006.50:0x80075304, US SLUS_006.50:0x80075698
worklog: 7, 14, 15, 22, 23
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
    u32  bits 0-12 q0, 13-25 q1, 26-29 mouth, 30-31 eyes
    u32  bits 0-12 q2, 13-25 q3
    s32  position[3]
    s16  id, u16 unknown_16
  pose[poses]         16 bytes and 8 a key (0x80073f08)
    s32  position[3]
    u32  bits 0-15 id, 16-23 keys, 24-27 mouth, 28-29 eyes
    key[keys]
      u32  bits 0-12 q0, 13-25 q1, 26-31 channel
      u32  bits 0-12 q2, 13-25 q3
  0xdddddddd          as the first word: the stream's end
```

Quaternions are (x, y, z, w) = (q0, q1, q2, q3), at any scale. Ids are the
object registry's, from the database's LP file ([lp](../formats/lp.md)).

## Timing and sound

The audio sectors are four in five of the stream, yet one stereo 37.8 kHz
channel at double speed needs one in eight. The encoder repeats each audio
sector to fill the bandwidth (535 of the first 634 neighbours are byte for
byte the same). The CD-ROM controller's decoder takes a sector only when
its output has nearly drained and drops the rest (no flow control; DuckStation
models it so: more than 10 frames queued, the sector is skipped and leaves
the ADPCM history alone). The 5643 audio sectors of the opening come out as
882 played, 47.1 s, matching the 1412 frames at 30 a second. Decoding every
sector gives 300 s with each piece repeated about six times.


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

## Faces

Mouths and eyes are blocks of texture that the cinematic overwrites in
VRAM (LoadImage after DrawSync) with frames from face banks:

- The banks are files 531-550, each loaded with a 64-byte header skipped
  (0x80074b9c), one per bit of the face mask at 0x800c4c70. A level's
  setup (0x80017c0c) takes the mask from 0x800b91a0 + 4 db; the opening's
  (0x8008f078) from db 22's, 0xd515.
- The mouth setter 0x80074ec0(frame, id) and the eye setter
  0x80075304(id, frame) map an id to its bank and rect. A frame is
  w x h words into the bank. A mouth frame past the count wraps back once
  (10; 12 for id 88; 13 for id 112).

| id | mouth rect (bit) | eye rect (bit) |
|---|---|---|
| 1492 | 704,256 8x32 (1) | 720,256 16x64 (0) |
| 1776 | 704,320 8x32 (3) | 736,256 16x64 (2) |
| 600 | 736,320 8x32 (5) | 720,320 16x64 (4) |
| 13 | 856,384 8x32 (7) | 832,384 8x32 (6) |
| 15 | 864,288 8x32 (9) | 776,352 8x32 (8) |
| 16 | 784,480 8x32 (11) | 816,320 8x32 (10) |
| 608, 609 | 704,384 8x32 (13) | 720,384 16x64 (12) |
| 88 | 160,384 16x32 (14) | the same |
| 112 | 256,256 32x96 (15) | the same |
| 75 | 808,384 8x32 (17) | 776,352 8x32 (16) |
| 76 | 800,352 8x32 (19) | 816,384 8x32 (18) |

Bit to file: 0 535, 1 531, 2 536, 3 532, 4 537, 5 545, 6 538, 7 546,
8 539, 9 540, 10 541, 11 542, 12 543, 13 544, 14 533, 15 534, 16 547,
17 548, 18 549, 19 550.

Ids 88 and 112 are not faces. They are the opening's milk effects, whose
texture the stream animates through the mouth field: 88 is the spray
from Tommy's bottle (records 599-608), and 112 is the splash over the
lens (from record 830).

How the records drive them:

- An actor (0x80073c34) first resets mouth and eyes to frame 0. Then a
  nonzero mouth field sets that frame, and a nonzero eyes field sets
  frame eyes - 1.
- A pose (0x80073f08) first blinks (0x80075698), then takes the same
  fields without the reset.
- The blink runs per object: state +0xa5, count +0xa7, and a rand of its
  own at 0x800c4c48 (x * 214013 + 2531011) that play_cine seeds with the
  cinematic's number. When open, the blink starts with a 1 in 32 chance
  ((x >> 16) & 31 == 0). It then shows eye frame 1 for three records and
  frame 2 for two.

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

Faces (`cine_faces` against the oracle's VRAM, `cine_blink` against its
blink rand):

- Every face rect at records 300, 604 and 840 equals the oracle's, except
  for blink timing.
- The oracle drops records when a frame runs long, 104 of the first 840.
  Each dropped record is a pose pass it never makes, so its blink rand
  falls behind the port's by those calls. Up to record 88, where nothing
  has dropped yet, the two take the same rand step for step.

## Unknown

- `unknown_02` (1380, the setup's size, in every OPEN.BIN frame), the
  setup block, the rest of the 16-byte table entries.
- Whether the real drive and GPU drop records as the oracle does, which
  sets when the characters blink.
- Whether Triangle really skips: the oracle has no input during the
  cinematic (see [input](input.md)).
