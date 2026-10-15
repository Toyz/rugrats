//! Speech (0x8008d8e8, 0x8008da4c): a level's spoken lines are XA in file
//! 617 + database, indexed by file 585 + database. A line is a run of one
//! channel's sectors; the drive plays one line at a time.

use rrt::kit::adpcm::xa;

use crate::disc::GameDisc;

/// The index and XA files of database 0.
const INDEX_FILE: usize = 585;
const XA_FILE: usize = 617;
/// Lines from here on (the screens' "You did it!" and friends) may play
/// again; below it a line plays once unless its flag bit 2 is set.
const REPEATS_FROM: u16 = 2040;

#[derive(Default)]
pub struct Speech {
    /// Line ids (low 12 bits), sorted, and per line its flags (top 4
    /// bits, bit 1 set once played) and record: sector offset, length in
    /// 8-sector units, channel less one.
    ids: Vec<u16>,
    flags: Vec<u8>,
    records: Vec<(u32, u32, u8)>,
    base: u32,
}

impl Speech {
    /// 0x8008d8e8: database `db`'s lines; none when it has no index.
    pub fn load(disc: &GameDisc, db: usize) -> Speech {
        let (Ok(raw), Ok(base)) = (disc.file(INDEX_FILE + db), disc.file_lba(XA_FILE + db)) else {
            return Speech::default();
        };
        let Some(n) = raw.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize) else {
            return Speech::default();
        };
        let at = 4 + 2 * n + (n & 1) * 2;
        if raw.len() < at + 4 * n {
            return Speech::default();
        }
        let words: Vec<u16> = (0..n).map(|k| u16::from_le_bytes([raw[4 + 2 * k], raw[5 + 2 * k]])).collect();
        let records = (0..n)
            .map(|k| {
                let e = &raw[at + 4 * k..at + 4 * k + 4];
                (u16::from_le_bytes([e[0], e[1]]) as u32, e[2] as u32, e[3])
            })
            .collect();
        Speech {
            ids: words.iter().map(|w| w & 4095).collect(),
            flags: words.iter().map(|w| (w >> 12) as u8).collect(),
            records,
            base,
        }
    }

    /// 0x8008da4c: line `n`'s sound, or None when the set lacks it or it
    /// has played and may not again.
    pub fn line(&mut self, disc: &GameDisc, n: u16) -> Option<Vec<(i16, i16)>> {
        let k = self.ids.binary_search(&n).ok()?;
        if self.flags[k] & 3 == 1 {
            return None;
        }
        if n < REPEATS_FROM {
            self.flags[k] |= 1;
        }
        let (offset, len, channel) = self.records[k];
        let start = self.base + offset - 8;
        let mut player = xa::Player::new(xa::DOUBLE_SPEED);
        for lba in start..start + len * 8 {
            let Ok(s) = disc.sector(lba) else { break };
            let r = &s.raw;
            if r[18] & 0x64 == 0x64 && (r[16], r[17]) == (1, channel + 1) {
                player.sector(u64::from(lba - start), xa::Coding(r[19]), &r[24..24 + xa::SECTOR_AUDIO]);
            }
        }
        rrt::tracing::info!("speech {n}: {} samples", player.sound.len());
        Some(player.sound)
    }
}
