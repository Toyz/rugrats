//! `DBnnLP.BIN`, a database's placements (loader SLUS_006.50:0x800454dc),
//! and `DB00/TEMPLATE.BIN`, the object templates they name.
//!
//! ```text
//! LP   u32 count; record[count], 96 bytes
//!        u32 kind              1 an object; 6 and 99 other (not yet read)
//!        u32 template          (kind 1)
//!        s32 position[3]
//!        u32 unknown_14
//!        s32 rotation[4]       a quaternion (x, y, z, w), 4096 = 1
//!        u16 unknown_28
//!        u16 id                the object registry's (0x800c413c), which
//!                              cinematics name
//! TEMPLATE  template[1272], 56 bytes
//!        u32 flags             bit 0: built on a hierarchy
//!        +0x10 u16 shape       the hierarchy (flags bit 0) or a model number
//! ```

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub kind: u32,
    pub template: u32,
    pub pos: [i32; 3],
    pub rotation: [i32; 4],
    pub id: u16,
    /// The object's flags (+0x14), which it starts with (0x8004691c).
    pub flags: u32,
    /// A door's level (+0x42) and the cinematic its use plays (+0x44)
    /// (0x8004f78c); 0 for none.
    pub level: i16,
    pub cine: i16,
}

pub fn parse_lp(b: &[u8]) -> Vec<Placement> {
    let w = |at: usize| b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()));
    let n = w(0).unwrap_or(0) as usize;
    (0..n)
        .filter_map(|k| {
            let r = 4 + 96 * k;
            Some(Placement {
                kind: w(r)?,
                template: w(r + 4)?,
                pos: [w(r + 8)? as i32, w(r + 12)? as i32, w(r + 16)? as i32],
                rotation: [w(r + 0x18)? as i32, w(r + 0x1c)? as i32, w(r + 0x20)? as i32, w(r + 0x24)? as i32],
                id: (w(r + 0x28)? >> 16) as u16,
                flags: w(r + 0x14)?,
                level: (w(r + 0x40)? >> 16) as i16,
                cine: w(r + 0x44)? as i16,
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Template {
    pub flags: u32,
    pub shape: u16,
    /// The 56-byte record (class +0x04, use +0x07, touch +0x0a, sound
    /// +0x24, pickable +0x28, trigger radius +0x34).
    pub raw: [u8; 56],
}

impl Template {
    /// Whether objects of it can be picked up (+0x28, object flag
    /// 0x10000, 0x8004691c).
    pub fn pickable(&self) -> bool {
        u16::from_le_bytes([self.raw[0x28], self.raw[0x29]]) != 0
    }

    pub fn hierarchy(&self) -> Option<usize> {
        (self.flags & 1 != 0).then_some(self.shape as usize)
    }
}

pub fn parse_templates(b: &[u8]) -> Vec<Template> {
    b.chunks_exact(56)
        .map(|e| Template { flags: u32::from_le_bytes(e[..4].try_into().unwrap()), shape: u16::from_le_bytes([e[0x10], e[0x11]]), raw: e.try_into().unwrap() })
        .collect()
}
