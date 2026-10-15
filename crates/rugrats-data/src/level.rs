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
//! A room's record: +0x30 its quads (count; the loader points +0x34 at
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
        let s = self.b.get(self.at..self.at + n).ok_or_else(|| LevelError(format!("ends at {:#x} wanting {n}", self.at)))?;
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
            let corner = |i: usize| Corner { x: h(4 * i) as i16, y: h(4 * i + 1) as i16, z: h(4 * i + 2) as i16, flags: h(4 * i + 3) };
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
