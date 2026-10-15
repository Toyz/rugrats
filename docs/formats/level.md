---
title: LEVELnn.BIN, a database's rooms
status: partial
discs: US
covers: US /DATA/DB00/LEVEL00.BIN, US SLUS_006.50:0x8002a208, US SLUS_006.50:0x800b6e98
worklog: 8
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
u32 m; u8 [m][12]            (0x800c4a74 = m, 0x800c4a98 the records)
```

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
- The room record's other fields, the quads' tails and flags, the 24-byte
  and 12-byte records, the byte lists (probably neighbouring rooms), and
  the geometry blobs.
