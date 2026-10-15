//! The memory cards, as folders on the host: `work/memcard/slot1` and
//! `slot2`, each holding the game's card file under its card name,
//! byte for byte what the game writes to a real card (its title frame,
//! then the save). Slot 1 always has a card; slot 2 when its folder exists.
//! There is no card image and no "no memory card" path.
//!
//! The game keeps one file, `BASLUS-00650Games   ` (built by
//! `bu%d%d:%2s%10s%-8s`, SLUS_006.50:0x80072498), one block:
//!
//! ```text
//! +0x000  title frame (0x80071ee0): "SC", 0x13 (three icon frames),
//!         blocks, the title in Shift-JIS (64 bytes), the icon CLUT at
//!         +0x60, three 16x16 4-bit icon frames at +0x80, +0x100, +0x180
//! +0x200  the save, 0x1590 bytes (1380 words), see [`Save`]
//! +0x1790 the rest of the last 128-byte frame the save's write sends
//!         (0x80072320 pads its tail from the work buffer at 0x800e5228)
//! ```

use std::path::{Path, PathBuf};

/// The two slots' folders.
pub const SLOTS: [&str; 2] = ["work/memcard/slot1", "work/memcard/slot2"];

pub const BLOCK: usize = 8192;
pub const FRAME: usize = 128;
pub const TITLE_SIZE: usize = 512;
/// The save's bytes and words.
pub const SAVE_SIZE: usize = 0x1590;
pub const WORDS: usize = SAVE_SIZE / 4;
/// A record's place and size in the save; record 0 is the save's header.
pub const RECORD: usize = 344;
/// The game state a record holds (0x800c5678, 328 bytes).
pub const STATE_SIZE: usize = 328;
pub const RECORDS: usize = 16;

/// What the card code takes from the executable.
#[derive(Clone, Debug)]
pub struct Art {
    /// The region code ("BA") at 0x800c4394.
    pub region: [u8; 2],
    /// The icon's 16-colour CLUT (0x800b8aac).
    pub clut: [u8; 32],
    /// The three icon frames (0x800b8958, 0x800b8a18, 0x800b8ad8).
    pub icons: [[u8; 128]; 3],
    /// Shift-JIS for digits, capitals and small letters: (code of the
    /// first, its ASCII) (0x800b8b58).
    pub ranges: [(u16, u8); 3],
    /// Shift-JIS for the 33 other printable characters, space first
    /// (0x800b8b64).
    pub punctuation: [u16; 33],
}

impl Art {
    /// From the executable's image (its bytes from 0x80010000).
    pub fn from_exe(exe: &[u8]) -> Option<Art> {
        let at = |addr: u32, len: usize| exe.get((addr - 0x8001_0000) as usize..(addr - 0x8001_0000) as usize + len);
        let h = |addr: u32| at(addr, 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
        let mut art = Art {
            region: at(0x800c_4394, 2)?.try_into().ok()?,
            clut: at(0x800b_8aac, 32)?.try_into().ok()?,
            icons: [[0; 128]; 3],
            ranges: [(0, 0); 3],
            punctuation: [0; 33],
        };
        for (icon, addr) in art.icons.iter_mut().zip([0x800b_8958, 0x800b_8a18, 0x800b_8ad8]) {
            icon.copy_from_slice(at(addr, 128)?);
        }
        for (k, r) in art.ranges.iter_mut().enumerate() {
            *r = (h(0x800b_8b58 + 4 * k as u32)?, h(0x800b_8b5a + 4 * k as u32)? as u8);
        }
        for (k, p) in art.punctuation.iter_mut().enumerate() {
            *p = h(0x800b_8b64 + 2 * k as u32)?;
        }
        Some(art)
    }

    /// One character in Shift-JIS, as 0x80070e84 converts it; anything
    /// unprintable is a space.
    pub fn sjis(&self, c: u8) -> u16 {
        let c = if (0x20..0x7f).contains(&c) { c } else { b' ' };
        let range = |k: usize| self.ranges[k].0 + (c - self.ranges[k].1) as u16;
        let punct = |first: u8, base: usize| self.punctuation[base + (c - first) as usize];
        match c {
            0x20..=0x2f => punct(0x20, 0),
            b'0'..=b'9' => range(0),
            0x3a..=0x40 => punct(0x3a, 16),
            b'A'..=b'Z' => range(1),
            0x5b..=0x60 => punct(0x5b, 23),
            b'a'..=b'z' => range(2),
            _ => punct(0x7b, 29),
        }
    }

    /// The title frame for a file named `name` of `blocks` blocks
    /// (0x80071ee0): the title is "RugRats " and the name, in full-width
    /// Shift-JIS over full-width spaces (0x80070fa0).
    pub fn title_frame(&self, name: &str, blocks: u8) -> [u8; TITLE_SIZE] {
        let mut f = [0u8; TITLE_SIZE];
        f[..4].copy_from_slice(&[b'S', b'C', 0x13, blocks]);
        let title = &mut f[4..68];
        for pair in title.chunks_exact_mut(2) {
            pair.copy_from_slice(&[0x81, 0x40]);
        }
        let text = format!("RugRats {name}");
        for (pair, c) in title.chunks_exact_mut(2).zip(text.bytes()) {
            pair.copy_from_slice(&self.sjis(c).to_be_bytes());
        }
        f[0x60..0x80].copy_from_slice(&self.clut);
        for (k, icon) in self.icons.iter().enumerate() {
            f[0x80 * (k + 1)..0x80 * (k + 2)].copy_from_slice(icon);
        }
        f
    }

    /// The card file's name: region, product, and `name` padded to 8.
    pub fn file_name(&self, name: &str) -> String {
        format!("{}{}{name:<8}", String::from_utf8_lossy(&self.region), PRODUCT)
    }
}

/// The product code in every file name.
pub const PRODUCT: &str = "SLUS-00650";
/// The one file's name part, and its records' names ("GAME %d").
pub const FILE: &str = "Games";

/// The save: 1380 words as the game keeps them at 0x800e3c98.
///
/// ```text
/// word 0   the sum of every word (this one taken as 0)
/// word 1   which records are in use, a bit each
/// word 2   set when a save has been loaded (0x800e3ca0)
/// word 3   the record last saved to or loaded
/// record k (1..15) at 344 k:
///   +0x10  the sum of the 86 words from here (this one taken as 0)
///   +0x14  its name, 12 bytes ("GAME %d"), +0x1f 0
///   +0x20  the game state (0x800c5678, 328 bytes)
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Save {
    pub words: Vec<u32>,
}

impl Default for Save {
    fn default() -> Save {
        Save { words: vec![0; WORDS] }
    }
}

impl Save {
    pub fn from_bytes(b: &[u8]) -> Option<Save> {
        let b = b.get(..SAVE_SIZE)?;
        Some(Save { words: b.chunks_exact(4).map(|w| u32::from_le_bytes(w.try_into().unwrap())).collect() })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.words.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    /// Record `k`'s game state (1-15), when its bit in word 1 is set.
    pub fn record(&self, k: usize) -> Option<[u8; STATE_SIZE]> {
        if k == 0 || k >= RECORDS || self.words[1] >> k & 1 == 0 {
            return None;
        }
        let b = self.to_bytes();
        b.get(RECORD * k + 0x20..RECORD * k + 0x20 + STATE_SIZE)?.try_into().ok()
    }

    /// Writes record `k` (1-15): its name ("GAME %d") and the game state,
    /// marks it used and last saved (words 1, 3), and seals.
    pub fn put_record(&mut self, k: usize, state: &[u8; STATE_SIZE]) {
        let mut b = self.to_bytes();
        let at = RECORD * k;
        let name = format!("GAME {k}");
        b[at + 0x14..at + 0x20].fill(0);
        b[at + 0x14..at + 0x14 + name.len()].copy_from_slice(name.as_bytes());
        b[at + 0x20..at + 0x20 + STATE_SIZE].copy_from_slice(state);
        *self = Save::from_bytes(&b).unwrap();
        self.words[1] |= 1 << k;
        self.words[3] = k as u32;
        self.seal();
    }

    fn record_sum(&self, k: usize) -> u32 {
        let at = (RECORD * k + 0x10) / 4;
        self.words[at + 1..at + 86].iter().fold(0u32, |a, w| a.wrapping_add(*w))
    }

    fn total(&self) -> u32 {
        self.words[1..].iter().fold(0u32, |a, w| a.wrapping_add(*w))
    }

    /// The sums, as the save's write makes them (0x8007062c): each
    /// record's, then the whole.
    pub fn seal(&mut self) {
        for k in 1..RECORDS {
            let s = self.record_sum(k);
            self.words[(RECORD * k + 0x10) / 4] = s;
        }
        self.words[0] = self.total();
    }

    /// Whether the sums hold, as the load checks them (0x80070234).
    pub fn check(&self) -> bool {
        self.words[0] == self.total() && (1..RECORDS).all(|k| self.words[(RECORD * k + 0x10) / 4] == self.record_sum(k))
    }
}

/// The card file, 8 KB: the title frame, the save, and the last frame's
/// tail as the game's write leaves it. Before writing, the save path read
/// sector 0 into the work buffer (the "MC" frame, its checksum 0x0e last)
/// and printed the file's path over its start; the save's last 16 bytes
/// then go over the path's first 16, and the frame goes out whole.
pub fn file_bytes(title: &[u8; TITLE_SIZE], save: &Save, port: usize, name: &str) -> Vec<u8> {
    let mut out = vec![0u8; BLOCK];
    out[..TITLE_SIZE].copy_from_slice(title);
    let data = save.to_bytes();
    out[TITLE_SIZE..TITLE_SIZE + SAVE_SIZE].copy_from_slice(&data);
    let mut frame = [0u8; FRAME];
    frame[..2].copy_from_slice(b"MC");
    frame[FRAME - 1] = b'M' ^ b'C';
    let path = format!("bu{}0:{name}\0", port);
    frame[..path.len()].copy_from_slice(path.as_bytes());
    let tail = SAVE_SIZE % FRAME;
    frame[..tail].copy_from_slice(&data[SAVE_SIZE - tail..]);
    let at = TITLE_SIZE + SAVE_SIZE - tail;
    out[at..at + FRAME].copy_from_slice(&frame);
    out
}

/// The cards in the slots.
pub struct Cards {
    dirs: [PathBuf; 2],
}

impl Default for Cards {
    fn default() -> Cards {
        Cards { dirs: SLOTS.map(PathBuf::from) }
    }
}

impl Cards {
    pub fn at(slot1: &Path, slot2: &Path) -> Cards {
        Cards { dirs: [slot1.to_path_buf(), slot2.to_path_buf()] }
    }

    /// Whether a card is in `port` (0 or 1): slot 1 always.
    pub fn present(&self, port: usize) -> bool {
        port == 0 || self.dirs[port].is_dir()
    }

    fn path(&self, port: usize, art: &Art) -> PathBuf {
        self.dirs[port].join(art.file_name(FILE))
    }

    /// The save on the card in `port`, if the file is there and whole.
    pub fn load(&self, port: usize, art: &Art) -> Option<Save> {
        let bytes = std::fs::read(self.path(port, art)).ok()?;
        Save::from_bytes(bytes.get(TITLE_SIZE..)?)
    }

    /// Writes the save, sealed, making the file (and the folder) if need be.
    pub fn store(&self, port: usize, art: &Art, save: &Save) -> std::io::Result<()> {
        if !self.present(port) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no card in slot 2"));
        }
        let mut save = save.clone();
        save.seal();
        let name = art.file_name(FILE);
        let blocks = SAVE_SIZE.div_ceil(BLOCK) as u8;
        std::fs::create_dir_all(&self.dirs[port])?;
        std::fs::write(self.path(port, art), file_bytes(&art.title_frame(FILE, blocks), &save, port, &name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sealed_save_checks() {
        let mut s = Save::default();
        for (k, w) in s.words.iter_mut().enumerate() {
            *w = (k as u32).wrapping_mul(2_654_435_761);
        }
        assert!(!s.check());
        s.seal();
        assert!(s.check());
        s.words[500] ^= 1;
        assert!(!s.check());
    }
}
