---
title: LEVELnn.BIN, a database's rooms
status: partial
discs: US
covers: US /DATA/DB00/LEVEL00.BIN, US SLUS_006.50:0x8002a208, US SLUS_006.50:0x800b6e98
worklog: 8, 10
---

# LEVELnn.BIN

A database's rooms, loaded by `load_rooms(db)` (0x8002a208). A table at
0x800b6e98, 12 bytes a database, names its three files: LEVEL (434 + db),
COL (466 + db) and A.RAW (499 + db).

## Layout

Little-endian. Counts are u32.

```
u32 n; u8 [n][56]            never used: n is 0 in every file
u32 n; quad[n]               48 bytes each
  corner[4]                  s16 x, s16 y, s16 z, u16 flags
  u8 [16]                    unknown
u32 n; u8 [n][24]            only LEVEL04 has any (63)
u32 rooms; room[rooms]       112 bytes each (kept at 0x800c4a8c, count 0x800c4a9c)
per room: u8 count; u8 [count]   each kept one less than stored
4 - (list bytes % 4) bytes   a whole word when the lists end on one
u32 size[rooms]              bytes of each room's geometry section
per room: u32 n; u32 len[n]; n blobs of len bytes
u32 m; texture[m]            the room texture table (0x800c4a74 = m, 0x800c4a98)
  u8 u0, v0; u16 clut; u8 u1, v1; u16 tpage; u8 u2, v2, u3, v3
```

## A room record (112 bytes)

```
+0x00  s32 centre[3], pad     blob coordinates are relative to it
+0x10  s32 min[3], pad        the room's bounds in the world
+0x20  s32 max[3], pad
+0x30  u32 quads              its portals; pointer at +0x34 after loading
+0x38  u32 boxes              its 24-byte records; pointer at +0x3c
+0x40  list                   its geometry blobs, linked by the loader
+0x48, +0x4c  lists           objects drawn with the room (each item's
                              function at +0x40 is called)
+0x58  u32 256                in every DB22 room
+0x5e  u8 count, +0x60 list   the byte list
```

## A geometry blob

Read by the hand-written renderer (0x80010e14):

```
u32  link
s16  min[3], u16 unknown_0a, s16 max[3], u16 unknown_12
u32  vertices
u32  faces_at                 from the blob's start
vertex[vertices]              s16 x, y, z, pad; u8 r, g, b, flash
at faces_at, sections:        u16 kind, u16 count, records
  0x61  quad      u8 v[4], u16 texture, u16 flags
  0x60  triangle  u8 v[3], u8 0xcd, u16 texture, u16 flags
  0xff  the end (count 1)
```

`texture` indexes the file's texture table. `flags` is 0 on disc; at run
time the renderer ORs in the outcodes its vertices share (bits 0-4 off
left, right, top, bottom, near; 0x20 all nearer than 8192, 0x40 all
nearer than 4096), and 0x200 makes a face double-sided. The vertex
colour's top byte is a brightening added to r, g and b once and cleared.

The renderer projects the vertices three at a time (RTPT), depth-cues the
colour of any vertex beyond depth 8192 with DPCS at `(z - 8192) >> 2`
(toward the far colour, black: 0x8009fd38(0, 0, 0)), though in play this
never happens: hooks on the DPCS branches (0x80011194, 0x800111ac) never
fired in the hub, level 2 or level 12 while the room renderer (0x80010e14)
ran thousands of times, so the port draws rooms without it,
rejects a face whose vertices share an outcode, whose farthest depth >> 2
is 16250 or more, or (unless 0x200) whose NCLIP is not positive, and
subdivides faces whose vertices are all nearer than 8192 (0x80012294) or
4096 (0x800123b8). The packets are POLY_GT3 and POLY_GT4, the texture
entry's three words copied into the uv slots, linked into the depth
buckets at 0x800ce238.

The loader gives each room, in file order, the next `room+0x30` quads
(pointer at +0x34) and the next `room+0x38` 24-byte records (pointer at
+0x3c; it sets each record's +6 to +0x12 less +0x0a, and room flag 1 when
there are any), clears +0x0c and +0x40..+0x57, and links each geometry
blob into the list at `room+0x40` (0x8001a1c0). The byte list's count is
kept at +0x5e, its bytes (less one) at a block pointed to by +0x60.

## Measured

Every LEVEL file but LEVEL08 is consumed whole by this walk, and in every
one the tail is exactly 12 bytes for each of its `m` records. Room counts
run from 1 (the golf holes 23, 24, 27, 29, 30) to 76 (DB05). Quad corners
are multiples of 256 in the files looked at (LEVEL22).

## Unknown

- LEVEL08.BIN does not fit: its first quad reads as 32-bit values
  (0xfffe5c00, 0xfffffe80, ...) and the third count is not a count. Whether
  another loader reads it, or the file is stale, is open.
- The room record's fields from +0x44 to +0x5d and +0x64 on, the
  portals' tails and corner flags beyond corner 1's (the room a portal
  leads to), the 24-byte records, the byte lists.
- The blob header's halfwords at +0x0a and +0x12.
