---
title: LEVELnn.COL, a database's collision
status: partial
discs: US
covers: US /DATA/DB00/LEVEL00.COL, US SLUS_006.50:0x8002a65c, US SLUS_006.50:0x8002aec4, US SLUS_006.50:0x8002af20, US SLUS_006.50:0x8002b114, US SLUS_006.50:0x8002b240, US SLUS_006.50:0x80068510
worklog: 27
---

# LEVELnn.COL

A height field on 1024-unit tiles. `load_rooms` (0x8002a208) loads it
(file 466 + db, kept at 0x800c4a88), and 0x8002a65c sets up the globals:

| global | what |
|---|---|
| 0x800c4a78 | the file (its bounds) |
| 0x800c4a7c | floor normals |
| 0x800c4a90 | ceiling normals |
| 0x800c4a94 | the 512 columns |

## Layout

Little-endian.

```
+0x000  s32 min[3], pad
+0x010  s32 max[3], pad          heights count down from max y
+0x020  u32 floor normals
+0x024  u32 unknown
+0x028  u32 offset of the floor normals; the ceiling normals follow
        normal: s16 x, y, z (4.12)
+0x02c  column[512], 8 bytes: u32 data pointer (set at load),
        s16 cells, s16 tiles
+0x102c the columns' data in order, each: cell[cells], tile[tiles]
  cell  u8 z cell, u8 tile index      sorted by z cell
  tile  u8 floor, u8 floor normal, u8 ceiling, u8 ceiling normal,
        u8 unknown[2], u8 flags, u8 unknown
```

- The column of a point is (x - min x) >> 10 and its cell is
  (z - min z) >> 10. Both round toward zero.
- 0x8002aec4 finds the cell by halves.
- Heights are max y - byte * 128. PlayStation y points down, so the floor
  is the larger y.
- A nonzero normal index slopes the height. The height is where the
  vertical line through the point, from h - 1024 to h + 1024, meets the
  plane through the tile's corner (x & -1024, h, z & -1024; for a
  half-3 tile, x | 1023 and z | 1023) with that normal (0x8002af20,
  0x80068510).
- Tiles stack: flags bit 0 marks the last of a cell's stack.
- Flags bit 7 halves a tile along a diagonal, and bits 5-6 name the half.
  - Bit 5 set: half 1 when (x & 1023) + (z & 1023) < 1024, else half 3.
  - Bit 5 clear: half 0 when (z & 1023) < (x & 1023), else half 2.

## The query (0x8002b240)

`query(pos, &ceiling, &floor, &tile)` returns one of:

- **2:** pos is outside the bounds.
- **1:** no tile, or the point is not between the heights.
- **0:** the point is in the open: ceiling < y < floor.

It picks the stack's tile for the point's half, then walks down to the
first tile whose ceiling the point is at or below.

`rugrats::collide::query` is the port. `tests/collide.rs` holds it to
the original called in rrt-emu over 4 million points of databases 0, 1,
2, 5 and 16, and every answer matches.

## Unknown

- Tile bytes 4, 5 and 7, and header +0x24.
- The other collision functions (0x8002b660 to 0x8002cd7c): walls,
  objects, the camera.
