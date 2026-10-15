//! The game's own directory: a table in the executable of (LBA, size)
//! pairs, indexed by file number. The game never opens a file by name; every
//! load goes through this table (`read_file`, SLUS_006.50:0x80019794).
//!
//! ```text
//! entry[714] at 0x800b2c40
//!   u32  lba     first sector; 0 = no file (the entry is unused)
//!   u32  size    bytes; for an interleaved stream, sectors x 2336
//! ```

/// Where the table sits in SLUS_006.50, and how many entries it has.
pub const TABLE_ADDR: u32 = 0x800b_2c40;
pub const COUNT: usize = 714;

/// Bytes per sector the size of a streamed (Form 2) file counts in: the
/// sector without its 12-byte sync and 4-byte header.
pub const STREAM_SECTOR: u32 = 2336;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub lba: u32,
    pub size: u32,
}

impl Entry {
    pub fn is_used(&self) -> bool {
        self.lba != 0
    }
}

/// The table, read out of the executable image loaded at `base`.
pub fn table(image: &[u8], base: u32) -> Option<Vec<Entry>> {
    let at = TABLE_ADDR.checked_sub(base)? as usize;
    let bytes = image.get(at..at + COUNT * 8)?;
    Some(
        bytes
            .chunks_exact(8)
            .map(|e| Entry {
                lba: u32::from_le_bytes(e[..4].try_into().unwrap()),
                size: u32::from_le_bytes(e[4..].try_into().unwrap()),
            })
            .collect(),
    )
}
