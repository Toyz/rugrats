//! `LEVELnn.BIN`: a database's rooms, as the room loader walks it
//! (SLUS_006.50:0x8002a208).
//!
//! ```text
//! u32 n; u8 a[n][56]          unknown records
//! u32 n; quad[n]              48 bytes: four (s16 x, y, z, u16 flags), 16 more
//! u32 n; u8 c[n][24]          unknown records
//! u32 rooms; room[rooms]      112 bytes each
//! per room: u8 count, count bytes (each stored less 1)
//! 4 - (list bytes % 4) bytes of padding (a whole word when aligned)
//! u32 size[rooms]             bytes of each room's geometry section
//! per room: u32 n, u32 len[n], n blobs of len[k] bytes
//! u32 m; u8 d[m][12]          (0x800c4a74 = m, 0x800c4a98 = the records)
//! ```
//!
//! A room's record: +0x00 its centre (s32 x, y, z; blob coordinates are
//! relative to it), +0x10 and +0x20 its bounds, +0x30 its quads (count; the loader points +0x34 at
//! them, in file order across rooms), +0x38 its 24-byte records (count, at
//! +0x3c), +0x40 the list its geometry blobs are linked into, +0x5e its
//! byte list (count, at +0x60).

#[derive(Debug)]
pub struct LevelError(pub String);

impl std::fmt::Display for LevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LEVEL: {}", self.0)
    }
}

impl std::error::Error for LevelError {}

/// A corner of a quad.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corner {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub flags: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quad {
    pub corners: [Corner; 4],
    pub tail: [u8; 16],
}

#[derive(Clone, Debug)]
pub struct Room {
    /// The 112-byte record as stored.
    pub record: Vec<u8>,
    /// The byte list after the records (each value as the game keeps it,
    /// one less than stored).
    pub list: Vec<i8>,
    /// The room's geometry blobs.
    pub blobs: Vec<Vec<u8>>,
}

impl Room {
    fn u32_at(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.record[at..at + 4].try_into().unwrap())
    }

    fn i32_at(&self, at: usize) -> i32 {
        self.u32_at(at) as i32
    }

    /// Where the room's geometry sits in the world (+0x00): blob
    /// coordinates are relative to it.
    pub fn centre(&self) -> [i32; 3] {
        [self.i32_at(0), self.i32_at(4), self.i32_at(8)]
    }

    /// The room's bounds in the world (+0x10, +0x20).
    pub fn bounds(&self) -> ([i32; 3], [i32; 3]) {
        (
            [self.i32_at(0x10), self.i32_at(0x14), self.i32_at(0x18)],
            [self.i32_at(0x20), self.i32_at(0x24), self.i32_at(0x28)],
        )
    }

    /// How many of the file's quads are this room's (+0x30).
    pub fn quads(&self) -> u32 {
        self.u32_at(0x30)
    }

    /// How many 24-byte records are this room's (+0x38).
    pub fn boxes(&self) -> u32 {
        self.u32_at(0x38)
    }
}

#[derive(Clone, Debug)]
pub struct Level {
    pub a: Vec<Vec<u8>>,
    pub quads: Vec<Quad>,
    pub c: Vec<Vec<u8>>,
    pub rooms: Vec<Room>,
    /// The trailing count and the bytes after it.
    pub tail_count: u32,
    pub tail: Vec<u8>,
}

struct Cursor<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], LevelError> {
        let s = self
            .b
            .get(self.at..self.at + n)
            .ok_or_else(|| LevelError(format!("ends at {:#x} wanting {n}", self.at)))?;
        self.at += n;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32, LevelError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn records(&mut self, size: usize) -> Result<Vec<&'a [u8]>, LevelError> {
        let n = self.u32()? as usize;
        if n > 1 << 16 {
            return Err(LevelError(format!("{n} records of {size} at {:#x}", self.at)));
        }
        (0..n).map(|_| self.take(size)).collect()
    }
}

pub fn parse(bytes: &[u8]) -> Result<Level, LevelError> {
    let mut c = Cursor { b: bytes, at: 0 };
    let a = c.records(56)?.into_iter().map(<[u8]>::to_vec).collect();
    let quads = c
        .records(48)?
        .into_iter()
        .map(|r| {
            let h = |k: usize| u16::from_le_bytes([r[2 * k], r[2 * k + 1]]);
            let corner = |i: usize| Corner {
                x: h(4 * i) as i16,
                y: h(4 * i + 1) as i16,
                z: h(4 * i + 2) as i16,
                flags: h(4 * i + 3),
            };
            Quad { corners: [corner(0), corner(1), corner(2), corner(3)], tail: r[32..48].try_into().unwrap() }
        })
        .collect();
    let cc = c.records(24)?.into_iter().map(<[u8]>::to_vec).collect();
    let records: Vec<Vec<u8>> = c.records(112)?.into_iter().map(<[u8]>::to_vec).collect();
    let mut lists = Vec::new();
    let start = c.at;
    for _ in &records {
        let n = c.take(1)?[0] as usize;
        lists.push(c.take(n)?.iter().map(|&v| v.wrapping_sub(1) as i8).collect::<Vec<i8>>());
    }
    // The loader steps past the lists by 4 - used % 4: to the next word,
    // and a whole word when they end on one.
    let used = c.at - start;
    c.at += 4 - used % 4;
    let sizes: Vec<u32> = (0..records.len()).map(|_| c.u32()).collect::<Result<_, _>>()?;
    let mut rooms = Vec::new();
    for ((record, list), size) in records.into_iter().zip(lists).zip(sizes) {
        let begin = c.at;
        let n = c.u32()? as usize;
        let lens: Vec<usize> = (0..n).map(|_| c.u32().map(|v| v as usize)).collect::<Result<_, _>>()?;
        let blobs = lens.iter().map(|&l| c.take(l).map(<[u8]>::to_vec)).collect::<Result<_, _>>()?;
        if c.at - begin != size as usize {
            return Err(LevelError(format!("room section {} bytes, size says {size}", c.at - begin)));
        }
        rooms.push(Room { record, list, blobs });
    }
    let tail_count = c.u32()?;
    let tail = bytes[c.at..].to_vec();
    Ok(Level { a, quads, c: cc, rooms, tail_count, tail })
}

/// One entry of a texture table (a LEVEL file's tail, TEXPOS.DAT): the
/// words a textured polygon's packet takes, as the hand-written renderer
/// copies them: u0 v0 CLUT, u1 v1 texture page, u2 v2 u3 v3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Texture {
    pub uv: [[u8; 2]; 4],
    pub clut: u16,
    pub tpage: u16,
}

impl Texture {
    pub fn parse(b: &[u8]) -> Texture {
        Texture {
            uv: [[b[0], b[1]], [b[4], b[5]], [b[8], b[9]], [b[10], b[11]]],
            clut: u16::from_le_bytes([b[2], b[3]]),
            tpage: u16::from_le_bytes([b[6], b[7]]),
        }
    }
}

/// A vertex of room geometry: its place, its baked colour, and a brightening
/// the renderer adds once and clears (the top byte of the colour word).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertex {
    pub pos: [i16; 3],
    pub rgb: [u8; 3],
    pub flash: u8,
}

/// A face: three or four vertex indices and a texture (the table's index in
/// the low half, flags in the high half: 0x200 drawn from both sides).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face {
    pub v: [u8; 4],
    pub quad: bool,
    pub texture: u16,
    pub flags: u16,
}

/// A geometry blob as the hand-written renderer (0x80010e14) reads it.
///
/// ```text
/// u32  link            (the room list's)
/// s16  min[3], u16 unknown_0a, s16 max[3], u16 unknown_12   the bounds
///      (the two u16s vary by blob, 0 to 0x8172, and the renderer does
///      not read them)
/// u32  vertices
/// u32  faces_at        from the blob's start
/// vertex[vertices]     s16 x, y, z, pad; u8 r, g, b, flash
/// sections at faces_at: u16 kind, u16 count, records
///   0x61  quads:     u8 v[4], u16 texture, u16 flags
///   0x60  triangles: u8 v[3], u8 0xcd, u16 texture, u16 flags
///   0xff  the end
/// ```
#[derive(Clone, Debug)]
pub struct Mesh {
    pub min: [i16; 3],
    pub max: [i16; 3],
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl Mesh {
    pub fn parse(b: &[u8]) -> Result<Mesh, LevelError> {
        let err = |what: &str| LevelError(format!("blob: {what}"));
        let h = |at: usize| b.get(at..at + 2).map(|s| i16::from_le_bytes([s[0], s[1]])).ok_or_else(|| err("short"));
        let w = |at: usize| {
            b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap())).ok_or_else(|| err("short"))
        };
        let min = [h(4)?, h(6)?, h(8)?];
        let max = [h(0xc)?, h(0xe)?, h(0x10)?];
        let count = w(0x14)? as usize;
        let mut p = w(0x18)? as usize;
        let vertices = (0..count)
            .map(|k| {
                let at = 0x1c + 12 * k;
                let c = b.get(at + 8..at + 12).ok_or_else(|| err("vertices run past the blob"))?;
                Ok(Vertex { pos: [h(at)?, h(at + 2)?, h(at + 4)?], rgb: [c[0], c[1], c[2]], flash: c[3] })
            })
            .collect::<Result<Vec<_>, LevelError>>()?;
        let mut faces = Vec::new();
        loop {
            let kind = w(p)? & 0xffff;
            let n = (w(p)? >> 16) as usize;
            p += 4;
            let quad = match kind {
                0xff => break,
                0x61 => true,
                0x60 => false,
                k => return Err(err(&format!("section kind {k:#x}"))),
            };
            for _ in 0..n {
                let r = b.get(p..p + 8).ok_or_else(|| err("faces run past the blob"))?;
                let v = if quad { [r[0], r[1], r[2], r[3]] } else { [r[0], r[1], r[2], r[2]] };
                if v.iter().any(|&i| i as usize >= count) {
                    return Err(err("a face names a vertex past the end"));
                }
                faces.push(Face {
                    v,
                    quad,
                    texture: u16::from_le_bytes([r[4], r[5]]),
                    flags: u16::from_le_bytes([r[6], r[7]]),
                });
                p += 8;
            }
        }
        if p != b.len() {
            return Err(err(&format!("ends at {p:#x} of {:#x}", b.len())));
        }
        Ok(Mesh { min, max, vertices, faces })
    }
}

impl Level {
    /// The room texture table: the tail's 12-byte records.
    pub fn textures(&self) -> Vec<Texture> {
        self.tail.chunks_exact(12).map(Texture::parse).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every LEVEL file on the disc but LEVEL08 is walked to its last byte,
    /// every blob parses as a mesh, and every face's texture is in the
    /// file's table.
    #[test]
    fn every_level_parses_and_its_faces_name_its_textures() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/DATA");
        let Ok(dbs) = std::fs::read_dir(&dir) else {
            eprintln!("skipped: no work/fs");
            return;
        };
        let mut checked = 0;
        for db in dbs.flatten() {
            let name = db.file_name().to_string_lossy().to_string();
            let Some(n) = name.strip_prefix("DB") else { continue };
            if n == "08" {
                continue;
            }
            let Ok(bytes) = std::fs::read(db.path().join(format!("LEVEL{n}.BIN"))) else { continue };
            let level = parse(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            let textures = level.textures();
            assert_eq!(textures.len(), level.tail_count as usize, "{name}");
            for room in &level.rooms {
                for blob in &room.blobs {
                    let mesh = Mesh::parse(blob).unwrap_or_else(|e| panic!("{name}: {e}"));
                    assert!(mesh.faces.iter().all(|f| (f.texture as usize) < textures.len()), "{name}");
                }
            }
            checked += 1;
        }
        assert_eq!(checked, 31);
    }
}
