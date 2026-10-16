//! A database's animations: `DBnnANM.BIN` (the index), `DBnnFRM.BIN` (the
//! frames), `DBnnROT.BIN` (the keys), as the loader (SLUS_006.50:0x80054fd0)
//! and the samplers (0x80055170 one frame, 0x800552bc between two) read
//! them.
//!
//! ```text
//! ANM  (u16 first_frame, u16 frames)[1280]     first 0xffff: none
//! FRM  frame: u16 unknown_0, u16 duration, u16 first_key, u16 keys
//!             (the loader multiplies duration by 5)
//! ROT  key, two words:
//!        w0: bits 0-13 q0, 14-27 q1 (signed), bit 29 a lift key
//!        w1: bits 0-13 q2, 14-26 q3 (signed), 27-31 channel
//!      a rotation key is the quaternion (q0, q1, q2, q3) = (x, y, z, w)
//!      for its channel; a lift key is the root's move over its frame:
//!      x w0's bits 0-13, y w0's bits 14-27, z w1's bits 0-13
//! ```
//!
//! A hierarchy's header maps channels to parts (hierarchy + 0x14 +
//! channel, 0xff for none).

use crate::level::LevelError as Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub unknown_0: u16,
    /// As the game keeps it: the file's value times 5.
    pub duration: u16,
    pub first_key: u16,
    pub keys: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Rotation {
        channel: u8,
        q: [i16; 4],
    },
    /// The root's displacement over the frame (0x800557ec): x in w0's
    /// bits 0-13, y in 14-27, z in w1's bits 0-13.
    Lift {
        channel: u8,
        dx: i16,
        dy: i16,
        dz: i16,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Anims {
    pub index: Vec<Option<(u16, u16)>>,
    pub frames: Vec<Frame>,
    pub keys: Vec<Key>,
}

fn sext(v: u32, bits: u32) -> i16 {
    (((v << (32 - bits)) as i32) >> (32 - bits)) as i16
}

impl Key {
    pub fn parse(w0: u32, w1: u32) -> Key {
        let channel = (w1 >> 27) as u8;
        if (w0 >> 28) & 2 != 0 {
            Key::Lift { channel, dx: sext(w0, 14), dy: sext(w0 >> 14, 14), dz: sext(w1, 14) }
        } else {
            Key::Rotation { channel, q: [sext(w0, 14), sext(w0 >> 14, 14), sext(w1, 14), sext(w1 >> 14, 13)] }
        }
    }
}

impl Anims {
    pub fn parse(anm: &[u8], frm: &[u8], rot: &[u8]) -> Result<Anims, Error> {
        let h = |b: &[u8], at: usize| u16::from_le_bytes([b[at], b[at + 1]]);
        let w = |b: &[u8], at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
        let index = anm.chunks_exact(4).map(|e| (h(e, 0) != 0xffff).then(|| (h(e, 0), h(e, 2)))).collect();
        let frames = frm
            .chunks_exact(8)
            .map(|f| Frame { unknown_0: h(f, 0), duration: h(f, 2).wrapping_mul(5), first_key: h(f, 4), keys: h(f, 6) })
            .collect::<Vec<_>>();
        let keys = rot.chunks_exact(8).map(|k| Key::parse(w(k, 0), w(k, 4))).collect::<Vec<_>>();
        if let Some(f) = frames.iter().find(|f| f.first_key as usize + f.keys as usize > keys.len()) {
            return Err(Error(format!(
                "FRM: a frame's keys {}..+{} run past ROT's {}",
                f.first_key,
                f.keys,
                keys.len()
            )));
        }
        Ok(Anims { index, frames, keys })
    }

    /// Animation `n`'s frames.
    pub fn frames_of(&self, n: usize) -> &[Frame] {
        match self.index.get(n).copied().flatten() {
            Some((first, count)) => self.frames.get(first as usize..first as usize + count as usize).unwrap_or(&[]),
            None => &[],
        }
    }

    /// The keys of a frame.
    pub fn keys_of(&self, f: &Frame) -> &[Key] {
        &self.keys[f.first_key as usize..f.first_key as usize + f.keys as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_unpack_their_fields() {
        // q0 = -1, q1 = 2, q2 = 3, q3 = 2048, channel 9 (q3 is 13 bits: the
        // matrix code normalises by |q|, so keys need not be unit).
        let w0 = 0x3fff | (2 << 14);
        let w1 = 3 | (2048 << 14) | (9 << 27);
        assert_eq!(Key::parse(w0, w1), Key::Rotation { channel: 9, q: [-1, 2, 3, 2048] });
        let lift = (2 << 28) | ((-5i32 as u32 & 0x3fff) << 14);
        assert_eq!(Key::parse(lift, 1 << 27), Key::Lift { channel: 1, dx: 0, dy: -5, dz: 0 });
    }

    /// Every database's three files read, every frame's keys in range.
    #[test]
    fn every_database_animations_parse() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/DATA");
        let Ok(dbs) = std::fs::read_dir(&dir) else {
            eprintln!("skipped: no work/fs");
            return;
        };
        let mut n = 0;
        for db in dbs.flatten() {
            let name = db.file_name().to_string_lossy().to_string();
            let Some(k) = name.strip_prefix("DB") else { continue };
            let read = |s: &str| std::fs::read(db.path().join(format!("DB{k}{s}.BIN")));
            let (Ok(a), Ok(f), Ok(r)) = (read("ANM"), read("FRM"), read("ROT")) else { continue };
            Anims::parse(&a, &f, &r).unwrap_or_else(|e| panic!("{name}: {e}"));
            n += 1;
        }
        assert!(n >= 28, "{n} databases");
    }
}
