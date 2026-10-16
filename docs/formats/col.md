---
title: LEVELnn.COL, a database's collision
status: partial
discs: US
covers: US /DATA/DB00/LEVEL00.COL, US SLUS_006.50:0x8002a65c, US SLUS_006.50:0x8002aec4, US SLUS_006.50:0x8002af20, US SLUS_006.50:0x8002b114, US SLUS_006.50:0x8002b240, US SLUS_006.50:0x8002baa0, US SLUS_006.50:0x8002bbdc, US SLUS_006.50:0x8002c3a0, US SLUS_006.50:0x8002c720, US SLUS_006.50:0x8002cb70, US SLUS_006.50:0x8002cd7c, US SLUS_006.50:0x80068510, US SLUS_006.50:0x8009f278
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

## The segment test (0x8002cd7c, "vectorwalk")

`vectorwalk(&from, &to, &depth, &point, &normal, &tile)`, the last two on
the stack. It walks the segment from `from` to `to` a cell at a time and
says whether it gets through. The aim marker's ray (0x80090554), the
camera, NPC line of sight and golf call it.

It returns one of:

- **0:** the segment gets through. Only `tile` is written (the last tile
  passed), unless from == to: then depth, point and normal are zeroed and
  tile is the start's.
- **1:** something stops it: `point` is where, `normal` what (4.12), `tile`
  the tile (0 for a missing one). `depth` is 0 for a wall and is not
  written for a floor or ceiling hit.
- **2:** `from` is not in the open (the query, 0x8002b240, does not answer
  0, which includes outside the bounds). Then point = from, depth =
  from y - the query's floor, tile = the query's, and normal = from - to
  normalized (`VectorNormal`, 0x8009f278).

A step at a time, with the scratchpad (0x1f800000) as its state:

1. **Next point.** From the current point to the edge of its 1024 cell
   in the axis the line leaves by first, or to `to` if that is nearer.
   The point lands on the first coordinate past the cell (cell + 1024, or
   cell - 1). The other axis is kept inside the cell (cell + 1023, or
   cell), and y follows the line. All of this is integer: products
   wrap, and divisions round toward zero. The step also records the side
   it moved through: 0 for -z, 1 for +z, 2 for -x, 3 for +x. A vertical
   segment (no x or z) steps straight to `to`.
2. **The diagonal** (0x8002bbdc). It finds the current point's stack
   (0x8002b114, which also checks y against the bounds) and, for a
   halved tile, the tile for the point's half. A step heading at the
   diagonal crosses it at a point found in xz by 0x8002baa0. That point
   is clamped onto the diagonal's cells and gets y from the line. The
   step then splits there, into the half it leaves and the opposite half
   (0<->2, 1<->3).
3. **The stack** (0x8002c720), for each part, with the normal a wall
   answers with. That normal is the side's (0x800b7018: +z, -z, +x, -x,
   none) for the first part, and for the second the diagonal's
   (0x800b7068, 2896 = 4096/sqrt 2, facing back into the half left).
   - **The tile.** Down the stack from the cell's first tile (of the
     half) to the first tile whose ceiling the start of the part is at
     or below. This is first by the flat ceiling - 1024, then by the
     sloped ceiling. If there is none, it is a wall. If the start is
     under the tile's floor, it is a wall.
   - **Clear of both.** If both ends are more than 1024 above the flat
     floor and the end is more than 1024 below the flat ceiling, the
     part gets through.
   - **The planes** (0x8002c3a0). Otherwise the part is tested against
     the tile's floor and ceiling planes (0x80068510, through the tile's
     corner as in the height). Of the two, the crossing nearer the start
     within the part (0..4096 along it) stops it, with that plane's
     normal.
   - **No tile.** A cell with no tile for the point stops the step
     where it came in (0x8002cb70). The normal is the side's, or
     next - current for the first cell.
4. **Repeat** while any moving axis has not reached `to`.

`rugrats::world::collide::segment` is the port. To compare it, run
`cargo run --release -p rugrats --example vectorwalk_check -- 3 1 --bare 2000`.
This calls the original in the oracle on 600 segments around the player
in level 3 and in level 1. It also calls it in a bare R3000 with only the
COL set up, on 2000 segments in each of databases 0-7 and 9-18
(`Col::parse` rejects database 8's file). That is 37198 segments. They
cover all three answers, 266 diagonal stops, 2759 floor or ceiling stops,
and 361 normals of a from - to wider than 16 bits. The return and all
four outputs match on every one.

Two segments were not compared, because the original raises an overflow
exception on them. In `VectorNormal` the `add` of the squares
(0x8009f2fc) traps when the sum passes 2^31. That needs two components of
from - to near 32768 in their low 16 bits (the GTE's IR registers keep
only those bits). The port answers a zero normal there.

## Unknown

- Tile bytes 4, 5 and 7, and header +0x24.
- The other collision functions (0x8002b660 to 0x8002ba30): walls and
  objects.
