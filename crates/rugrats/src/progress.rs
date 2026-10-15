//! The game state (SLUS_006.50:0x800c5678, 328 bytes): what a save
//! record holds. The fields the port keeps:
//!
//! ```text
//! +0x00  s32 the level        +0x04 s32 the level before
//! +0x08  s32 the character    +0x0c s32 music volume, +0x10 effects
//! +0x22  u8  main levels completed
//! +0x34  u32 a bit a level played      +0x38 completed
//! +0x40  u32 a bit a level whose piece was given
//! +0x44  u16 pieces earned, a bit each  +0x46 placed
//! +0x48  u32[32] Reptar bars found, a bit a bar, by level
//! +0xc8  u32[32] bars spent
//! ```

use crate::card::STATE_SIZE;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub block: [u8; STATE_SIZE],
}

impl Default for Progress {
    /// A new game (0x80091a44): level 1, Tommy, the volumes 90 and 100.
    fn default() -> Progress {
        let mut p = Progress { block: [0; STATE_SIZE] };
        p.set_u32(0x00, 1);
        p.set_u32(0x04, u32::MAX);
        p.set_u32(0x08, 1);
        p.set_u32(0x0c, 90);
        p.set_u32(0x10, 100);
        p
    }
}

impl Progress {
    fn u32(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.block[at..at + 4].try_into().unwrap())
    }

    fn set_u32(&mut self, at: usize, v: u32) {
        self.block[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// Whether bar `n` of `level` was found (+0x48).
    pub fn bar_found(&self, level: usize, n: usize) -> bool {
        level < 32 && n < 32 && self.u32(0x48 + 4 * level) >> n & 1 != 0
    }

    /// Bar `n` of `level` found (0x80094a58).
    pub fn find_bar(&mut self, level: usize, n: usize) {
        if level < 32 && n < 32 {
            let w = self.u32(0x48 + 4 * level) | 1 << n;
            self.set_u32(0x48 + 4 * level, w);
        }
    }

    /// Bars found less bars spent (0x80094a0c).
    pub fn bars_available(&self) -> u32 {
        let count = |at: usize| (0..32).map(|k| self.u32(at + 4 * k).count_ones()).sum::<u32>();
        count(0x48).saturating_sub(count(0xc8))
    }

    /// Level `level` played (+0x34) and the levels moved on.
    pub fn enter(&mut self, level: usize) {
        let now = self.u32(0x00);
        self.set_u32(0x04, now);
        self.set_u32(0x00, level as u32);
        if level < 32 {
            let w = self.u32(0x34) | 1 << level;
            self.set_u32(0x34, w);
        }
    }

    /// The level saved in (+0x00).
    pub fn level(&self) -> usize {
        self.u32(0x00).clamp(1, 31) as usize
    }

    /// Pieces earned (0x80094b8c).
    pub fn pieces(&self) -> u32 {
        (u16::from_le_bytes([self.block[0x44], self.block[0x45]]).count_ones()).min(12)
    }
}
