//! `DBnnHIE.BIN`: a database's hierarchies, the skeletons its characters
//! and jointed props are built on (loader SLUS_006.50:0x80044e7c).
//!
//! ```text
//! u32 at[256]          offset of hierarchy i, 0 for none
//! hierarchy
//!   s16  parts
//!   u8   unknown[0x32]     byte lists (part groups, 0xff ended)
//!   part[parts]            40 bytes each, from +0x34
//!     u16  unknown_00
//!     u16  depth           1 for the root; a part's parent is the
//!                          nearest earlier part one level up
//!     u8   [16]            0 on disc (run-time)
//!     s32  offset[3]       from the parent's joint
//!     u32  unknown_20
//!     u32  model           an index into the database's MOD file
//! ```

use crate::level::LevelError as Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    pub depth: u16,
    pub parent: Option<usize>,
    pub offset: [i32; 3],
    pub model: u32,
}

#[derive(Clone, Debug)]
pub struct Hierarchy {
    pub header: Vec<u8>,
    pub parts: Vec<Part>,
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, Error> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap())).ok_or_else(|| Error(format!("HIE: short at {at:#x}")))
}

impl Hierarchy {
    /// The larger of the half sizes at header +0x08 and +0x10 (0x80084084
    /// takes it as the bite reach's half).
    pub fn half_size(&self) -> i32 {
        let h = |at: usize| self.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32).abs();
        h(0x08).max(h(0x10))
    }

    pub fn parse(b: &[u8]) -> Result<Hierarchy, Error> {
        let n = b.get(..2).map(|s| i16::from_le_bytes([s[0], s[1]])).ok_or_else(|| Error("HIE: short".into()))?;
        let mut parts: Vec<Part> = Vec::new();
        for k in 0..n.max(0) as usize {
            let at = 0x34 + 40 * k;
            let depth = (u32_at(b, at)? >> 16) as u16;
            let parent = parts.iter().rposition(|p| p.depth + 1 == depth);
            let offset = [u32_at(b, at + 0x14)? as i32, u32_at(b, at + 0x18)? as i32, u32_at(b, at + 0x1c)? as i32];
            parts.push(Part { depth, parent, offset, model: u32_at(b, at + 0x24)? });
        }
        Ok(Hierarchy { header: b[..0x34.min(b.len())].to_vec(), parts })
    }

    /// Each part's joint relative to the root, at rest (no rotation).
    pub fn rest(&self) -> Vec<[i32; 3]> {
        let mut out: Vec<[i32; 3]> = Vec::with_capacity(self.parts.len());
        for p in &self.parts {
            let base = p.parent.map_or([0; 3], |i| out[i]);
            out.push([base[0] + p.offset[0], base[1] + p.offset[1], base[2] + p.offset[2]]);
        }
        out
    }
}

/// A whole HIE file: the 256 slots.
pub fn parse_file(b: &[u8]) -> Result<Vec<Option<Hierarchy>>, Error> {
    (0..256)
        .map(|i| {
            let at = u32_at(b, 4 * i)? as usize;
            if at == 0 {
                return Ok(None);
            }
            Hierarchy::parse(b.get(at..).ok_or_else(|| Error(format!("HIE {i}: past the end")))?).map(Some)
        })
        .collect()
}
