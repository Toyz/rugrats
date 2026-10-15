//! A memory card for the oracle, kept as a folder on the host: each file
//! on the card is a host file of the same name, byte for byte (title frame
//! and data, whole 8 KB blocks), the way the port keeps its cards. Slot 1
//! holds `work/oracle/memcard/slot1` (or RUGRATS_CARD); slot 2 a card only
//! when `slot2` beside it exists.
//!
//! The BIOS calls the game's card code makes are answered here: the card
//! events (OpenEvent, TestEvent, ... for classes 0xf0000011, the hardware
//! side, and 0xf4000001, the BIOS's), `_card_info` and `_card_load`, raw
//! sector reads (`_card_read`, from a card image made up from the folder:
//! the "MC" frame, the directory frames, the blocks), and the `bu00:` file
//! calls (open, read, write, lseek, close, firstfile, nextfile, erase,
//! format).

use std::path::{Path, PathBuf};

/// The hardware card events and the BIOS's.
pub const HW_CARD: u32 = 0xf000_0011;
pub const SW_CARD: u32 = 0xf400_0001;
/// Event specs: done, error, timeout, new card.
pub const EV_IOE: u32 = 0x0004;
pub const EV_ERROR: u32 = 0x8000;
pub const EV_TIMEOUT: u32 = 0x0100;
pub const EV_NEW: u32 = 0x2000;

pub const BLOCK: usize = 8192;
pub const FRAME: usize = 128;
pub const BLOCKS: usize = 15;

struct Event {
    id: u32,
    class: u32,
    spec: u32,
    pending: bool,
}

struct Open {
    path: PathBuf,
    pos: usize,
}

pub struct Card {
    slots: [Option<PathBuf>; 2],
    events: Vec<Event>,
    next_id: u32,
    files: Vec<Option<Open>>,
    /// The last firstfile pattern and how far nextfile has got.
    search: Option<(usize, String, usize)>,
}

impl Card {
    pub fn new(slot1: PathBuf) -> Card {
        let slot2 = slot1.with_file_name("slot2");
        let slot2 = slot2.is_dir().then_some(slot2);
        Card { slots: [Some(slot1), slot2], events: Vec::new(), next_id: 0xf100_0001, files: Vec::new(), search: None }
    }

    /// Whether events of `class` are modelled here (the rest are left to
    /// the stock BIOS, which reports every event delivered).
    pub fn owns(class: u32) -> bool {
        class == HW_CARD || class == SW_CARD
    }

    pub fn open_event(&mut self, class: u32, spec: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.events.push(Event { id, class, spec, pending: false });
        id
    }

    pub fn is_ours(&self, id: u32) -> bool {
        self.events.iter().any(|e| e.id == id)
    }

    /// TestEvent: whether `id` was delivered, clearing it.
    pub fn test_event(&mut self, id: u32) -> bool {
        self.events.iter_mut().find(|e| e.id == id).is_some_and(|e| std::mem::take(&mut e.pending))
    }

    pub fn deliver(&mut self, class: u32, spec: u32) {
        tracing::debug!("card deliver {class:#x} {spec:#x} to {:x?}", self.events.iter().filter(|e| e.class == class && e.spec == spec).map(|e| e.id).collect::<Vec<_>>());
        for e in self.events.iter_mut().filter(|e| e.class == class && e.spec == spec) {
            e.pending = true;
        }
    }

    pub fn undeliver(&mut self, class: u32, spec: u32) {
        for e in self.events.iter_mut().filter(|e| e.class == class && e.spec == spec) {
            e.pending = false;
        }
    }

    /// A channel as the BIOS numbers it: port in bit 4, 0x00 or 0x10.
    fn slot(&self, chan: u32) -> Option<&Path> {
        self.slots.get(((chan >> 4) & 1) as usize).and_then(|s| s.as_deref())
    }

    /// `_card_info(chan)`: done when a card is there, a time-out when not.
    pub fn info(&mut self, chan: u32) {
        let spec = if self.slot(chan).is_some() { EV_IOE } else { EV_TIMEOUT };
        self.deliver(HW_CARD, spec);
    }

    /// `_card_load(chan)`: the BIOS reads the directory.
    pub fn load(&mut self, chan: u32) {
        let spec = if self.slot(chan).is_some() { EV_IOE } else { EV_TIMEOUT };
        self.deliver(SW_CARD, spec);
    }

    /// The files on the card in slot `chan`, in the order they take blocks.
    fn listing(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut out: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_file())
            .filter_map(|e| Some((e.file_name().to_str()?.to_string(), std::fs::read(e.path()).ok()?)))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// The card's 128 KB as the hardware would hold it.
    pub fn image(dir: &Path) -> Vec<u8> {
        let mut img = vec![0u8; BLOCK * 16];
        let checksum = |f: &mut [u8]| f[FRAME - 1] = f[..FRAME - 1].iter().fold(0, |a, b| a ^ b);
        img[0] = b'M';
        img[1] = b'C';
        checksum(&mut img[..FRAME]);
        let mut block = 1;
        let mut dir_frames = vec![(0xa0u32, 0u32, 0xffffu16, String::new()); BLOCKS];
        for (name, bytes) in Self::listing(dir) {
            let n = bytes.len().div_ceil(BLOCK).max(1);
            if block + n > 16 {
                break;
            }
            for k in 0..n {
                let state = match (k, n) {
                    (0, _) => 0x51,
                    (k, n) if k + 1 == n => 0x53,
                    _ => 0x52,
                };
                let next = if k + 1 < n { (block + k) as u16 } else { 0xffff };
                let size = if k == 0 { (n * BLOCK) as u32 } else { 0 };
                dir_frames[block + k - 1] = (state, size, next, if k == 0 { name.clone() } else { String::new() });
            }
            let at = block * BLOCK;
            let len = bytes.len().min(n * BLOCK);
            img[at..at + len].copy_from_slice(&bytes[..len]);
            block += n;
        }
        for (k, (state, size, next, name)) in dir_frames.into_iter().enumerate() {
            let f = &mut img[(k + 1) * FRAME..(k + 2) * FRAME];
            f[0..4].copy_from_slice(&state.to_le_bytes());
            f[4..8].copy_from_slice(&size.to_le_bytes());
            f[8..10].copy_from_slice(&next.to_le_bytes());
            let n = name.len().min(20);
            f[10..10 + n].copy_from_slice(&name.as_bytes()[..n]);
            checksum(f);
        }
        // The broken-sector list: none.
        for k in 16..36 {
            let f = &mut img[k * FRAME..(k + 1) * FRAME];
            f[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
            f[8..10].copy_from_slice(&0xffffu16.to_le_bytes());
            checksum(f);
        }
        img
    }

    /// `_card_read(chan, sector, buf)`: the frame, and done.
    pub fn read_sector(&mut self, chan: u32, sector: u32) -> Option<[u8; FRAME]> {
        let dir = self.slot(chan)?.to_path_buf();
        let img = Self::image(&dir);
        let at = (sector as usize % (img.len() / FRAME)) * FRAME;
        self.deliver(HW_CARD, EV_IOE);
        Some(img[at..at + FRAME].try_into().unwrap())
    }

    /// `_card_write(chan, sector, buf)`: done; raw writes are not kept (the
    /// folder holds files, not frames).
    pub fn write_sector(&mut self, chan: u32, sector: u32) {
        if self.slot(chan).is_some() {
            if sector != 63 {
                tracing::warn!("card: raw write to sector {sector} not kept");
            }
            self.deliver(HW_CARD, EV_IOE);
        } else {
            self.deliver(HW_CARD, EV_TIMEOUT);
        }
    }

    /// "buXY:NAME" to the slot's folder and NAME.
    fn split(&self, path: &str) -> Option<(PathBuf, String)> {
        let rest = path.strip_prefix("bu")?;
        let (dev, name) = rest.split_once(':')?;
        let port = dev.chars().next()?.to_digit(10)?;
        let dir = self.slots.get(port as usize)?.clone()?;
        Some((dir, name.to_string()))
    }

    /// open(path, flags): 1 read, 2 write, 0x200 create with the blocks in
    /// the high half. The file descriptor, or -1.
    pub fn open(&mut self, path: &str, flags: u32) -> i32 {
        let Some((dir, name)) = self.split(path) else { return -1 };
        let file = dir.join(&name);
        if flags & 0x200 != 0 {
            let blocks = ((flags >> 16) as usize).max(1);
            let used: usize = Self::listing(&dir).iter().map(|(_, b)| b.len().div_ceil(BLOCK).max(1)).sum();
            if file.exists() || used + blocks > BLOCKS || name.is_empty() || name.len() > 20 {
                return -1;
            }
            if std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&file, vec![0u8; blocks * BLOCK])).is_err() {
                return -1;
            }
        } else if !file.is_file() {
            return -1;
        }
        let open = Some(Open { path: file, pos: 0 });
        let fd = match self.files.iter().position(Option::is_none) {
            Some(k) => {
                self.files[k] = open;
                k
            }
            None => {
                self.files.push(open);
                self.files.len() - 1
            }
        };
        fd as i32 + 2
    }

    fn file(&mut self, fd: u32) -> Option<&mut Open> {
        self.files.get_mut((fd as usize).checked_sub(2)?)?.as_mut()
    }

    pub fn close(&mut self, fd: u32) -> i32 {
        match (fd as usize).checked_sub(2).and_then(|k| self.files.get_mut(k)) {
            Some(f) if f.is_some() => {
                *f = None;
                fd as i32
            }
            _ => -1,
        }
    }

    pub fn seek(&mut self, fd: u32, off: i32, whence: u32) -> i32 {
        let Some(f) = self.file(fd) else { return -1 };
        let len = std::fs::metadata(&f.path).map(|m| m.len() as i64).unwrap_or(0);
        let base = match whence {
            1 => f.pos as i64,
            2 => len,
            _ => 0,
        };
        f.pos = (base + off as i64).clamp(0, len) as usize;
        f.pos as i32
    }

    pub fn read(&mut self, fd: u32, len: usize) -> Option<Vec<u8>> {
        let f = self.file(fd)?;
        let bytes = std::fs::read(&f.path).ok()?;
        let end = (f.pos + len).min(bytes.len());
        let out = bytes[f.pos.min(end)..end].to_vec();
        f.pos = end;
        Some(out)
    }

    pub fn write(&mut self, fd: u32, data: &[u8]) -> i32 {
        let Some(f) = self.file(fd) else { return -1 };
        let Ok(mut bytes) = std::fs::read(&f.path) else { return -1 };
        // A file never grows past the blocks it was made with.
        let end = (f.pos + data.len()).min(bytes.len());
        let n = end - f.pos.min(end);
        bytes[f.pos..end].copy_from_slice(&data[..n]);
        if std::fs::write(&f.path, &bytes).is_err() {
            return -1;
        }
        f.pos = end;
        n as i32
    }

    pub fn erase(&mut self, path: &str) -> bool {
        self.split(path).is_some_and(|(dir, name)| std::fs::remove_file(dir.join(name)).is_ok())
    }

    /// format("buXY:"): every file gone.
    pub fn format(&mut self, path: &str) -> bool {
        let Some((dir, _)) = self.split(path) else { return false };
        for (name, _) in Self::listing(&dir) {
            let _ = std::fs::remove_file(dir.join(name));
        }
        std::fs::create_dir_all(&dir).is_ok()
    }

    fn matches(pattern: &str, name: &str) -> bool {
        let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
        let mut k = 0;
        for &c in &p {
            match c {
                '*' => return true,
                '?' if k < n.len() => k += 1,
                c if n.get(k) == Some(&c) => k += 1,
                _ => return false,
            }
        }
        k == n.len()
    }

    /// firstfile / nextfile: the next match, with its size and first block.
    pub fn find(&mut self, path: Option<&str>) -> Option<(String, u32, u32)> {
        if let Some(path) = path {
            let (dir, pattern) = self.split(path)?;
            let port = self.slots.iter().position(|s| s.as_deref() == Some(dir.as_path()))?;
            self.search = Some((port, pattern, 0));
        }
        let (port, pattern, from) = self.search.clone()?;
        let dir = self.slots[port].clone()?;
        let mut block = 1;
        for (k, (name, bytes)) in Self::listing(&dir).into_iter().enumerate() {
            let n = bytes.len().div_ceil(BLOCK).max(1);
            if k >= from && Self::matches(&pattern, &name) {
                self.search = Some((port, pattern, k + 1));
                return Some((name, (n * BLOCK) as u32, block as u32));
            }
            block += n;
        }
        self.search = None;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns() {
        assert!(Card::matches("*", "BASLUS-00650RUGRATS"));
        assert!(Card::matches("BASLUS-00650*", "BASLUS-00650RUGRATS"));
        assert!(!Card::matches("BASLUS-00651*", "BASLUS-00650RUGRATS"));
        assert!(Card::matches("BASLUS-00650RUGRATS", "BASLUS-00650RUGRATS"));
        assert!(!Card::matches("BASLUS-00650RUGRAT", "BASLUS-00650RUGRATS"));
    }

    #[test]
    fn an_empty_card_is_formatted() {
        let img = Card::image(Path::new("no-such-card-folder"));
        assert_eq!(&img[..2], b"MC");
        assert_eq!(img[FRAME + 0], 0xa0);
    }
}
