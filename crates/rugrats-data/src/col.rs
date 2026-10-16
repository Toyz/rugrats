//! `LEVELnn.COL`: a database's collision, a height field on a grid of
//! 1024-unit tiles, as `load_rooms` (SLUS_006.50:0x8002a208) loads it and
//! 0x8002a65c sets it up.
//!
//! ```text
//! +0x000  s32 min[3], pad      the field's bounds in the world
//! +0x010  s32 max[3], pad      heights count down from max y
//! +0x020  u32 normals          entries in the first normal array
//! +0x024  u32 unknown
//! +0x028  u32 normals_at       offset of the first normal array; the
//!                              second follows it (s16 x, y, z each, 4.12)
//! +0x02c  column[512]          8 bytes: u32 (the pointer, set at load),
//!                              s16 cells, s16 tiles
//! +0x102c the columns' data, in order: cell[cells] then tile[tiles]
//!   cell   u8 z cell, u8 tile   sorted by z cell
//!   tile   u8 floor, u8 floor_normal, u8 ceiling, u8 ceiling_normal,
//!          u8 unknown[2], u8 flags, u8 unknown
//!          heights are max y - byte * 128 (y points down, so the floor
//!          is the larger); a normal index (1 on) slopes
//!          the height through the tile's corner; flags bit 0 ends a
//!          stack of tiles, bit 7 halves the tile along a diagonal and
//!          bits 5-6 name the half
//! ```
//!
//! Column = (x - min x) >> 10, cell = (z - min z) >> 10.

#[derive(Clone, Debug, Default)]
pub struct Column {
    pub cells: Vec<[u8; 2]>,
    pub tiles: Vec<[u8; 8]>,
}

#[derive(Clone, Debug, Default)]
pub struct Col {
    pub min: [i32; 3],
    pub max: [i32; 3],
    /// Normals for the floors (0x800c4a7c) and the ceilings (0x800c4a90).
    pub floor_normals: Vec<[i16; 3]>,
    pub ceiling_normals: Vec<[i16; 3]>,
    pub columns: Vec<Column>,
}

pub const COLUMNS: usize = 512;
const DATA: usize = 0x102c;

impl Col {
    pub fn parse(b: &[u8]) -> Result<Col, String> {
        let w = |at: usize| -> Result<u32, String> {
            b.get(at..at + 4)
                .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
                .ok_or_else(|| format!("COL: short at {at:#x}"))
        };
        let h = |at: usize| -> Result<i16, String> {
            b.get(at..at + 2).map(|s| i16::from_le_bytes([s[0], s[1]])).ok_or_else(|| format!("COL: short at {at:#x}"))
        };
        let min = [w(0)? as i32, w(4)? as i32, w(8)? as i32];
        let max = [w(0x10)? as i32, w(0x14)? as i32, w(0x18)? as i32];
        let normals = w(0x20)? as usize;
        let at = w(0x28)? as usize;
        let normal =
            |k: usize| -> Result<[i16; 3], String> { Ok([h(at + 6 * k)?, h(at + 6 * k + 2)?, h(at + 6 * k + 4)?]) };
        let floor_normals = (0..normals).map(normal).collect::<Result<Vec<_>, _>>()?;
        // The second array runs to the end of what the tiles can name.
        let ceilings_at = at + 6 * normals;
        let mut columns = Vec::with_capacity(COLUMNS);
        let mut p = DATA;
        for c in 0..COLUMNS {
            let (cells, tiles) = (h(0x2c + 8 * c + 4)? as usize, h(0x2c + 8 * c + 6)? as usize);
            let cell_bytes = b.get(p..p + 2 * cells).ok_or("COL: cells run past the file")?;
            let tile_bytes = b.get(p + 2 * cells..p + 2 * cells + 8 * tiles).ok_or("COL: tiles run past the file")?;
            columns.push(Column {
                cells: cell_bytes.chunks_exact(2).map(|x| [x[0], x[1]]).collect(),
                tiles: tile_bytes.chunks_exact(8).map(|x| x.try_into().unwrap()).collect(),
            });
            p += 2 * cells + 8 * tiles;
        }
        let ceilings = columns.iter().flat_map(|c| &c.tiles).map(|t| t[3] as usize + 1).max().unwrap_or(0);
        let ceiling_normals = (0..ceilings)
            .map(|k| -> Result<[i16; 3], String> {
                Ok([h(ceilings_at + 6 * k)?, h(ceilings_at + 6 * k + 2)?, h(ceilings_at + 6 * k + 4)?])
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        Ok(Col { min, max, floor_normals, ceiling_normals, columns })
    }
}
