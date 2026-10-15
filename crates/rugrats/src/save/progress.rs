//! The game state (SLUS_006.50:0x800c5678, 328 bytes): what a save
//! record holds. The fields the port keeps:
//!
//! ```text
//! +0x00  s32 the level        +0x04 s32 the level before
//! +0x08  s32 the character    +0x0c s32 music volume, +0x10 effects
//! +0x21  u8  activity play      +0x22 u8 main levels completed
//! +0x30  u32 flags: 1 use the respawn, 2 checkpoint, 16 Mirrorland
//!            done, 512 the final level reached
//! +0x34  u32 a bit a level played      +0x38 completed
//! +0x40  u32 a bit a level whose piece was given
//! +0x44  u16 pieces earned, a bit each  +0x46 placed
//! +0x48  u32[32] Reptar bars found, a bit a bar, by level
//! +0xc8  u32[32] bars spent
//! ```

use crate::levels::Level;
use crate::rand::Rand;
use crate::save::card::STATE_SIZE;
use crate::ui::audio_menu::Settings;

/// What finishing a level brought (0x80076d6c).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Finish {
    /// Bonus bars given ("Great Job! You have earned n Reptar bars!").
    pub bars: u32,
    /// A puzzle piece given: the puzzle screen shows.
    pub piece: bool,
    /// The level that starts next (SwitchEpisode), instead of the exit's.
    pub next: Option<Level>,
}

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

    /// The audio settings (+0x0c music, +0x10 effects, +0x14 mono).
    pub fn audio(&self) -> Settings {
        Settings { music: self.u32(0x0c) as i32, effects: self.u32(0x10) as i32, mono: self.u32(0x14) != 0 }
    }

    pub fn set_audio(&mut self, s: Settings) {
        self.set_u32(0x0c, s.music.clamp(0, 100) as u32);
        self.set_u32(0x10, s.effects.clamp(0, 100) as u32);
        self.set_u32(0x14, s.mono as u32);
    }

    /// Whether bar `n` of `level` was found (+0x48).
    pub fn bar_found(&self, level: Level, n: usize) -> bool {
        n < 32 && self.bars(level) >> n & 1 != 0
    }

    /// Bar `n` of `level` found (0x80094a58).
    pub fn find_bar(&mut self, level: Level, n: usize) {
        if n < 32 {
            self.set_bars(level, self.bars(level) | 1 << n);
        }
    }

    /// The bars found in `level` (+0x48), a bit a bar.
    fn bars(&self, level: Level) -> u32 {
        self.u32(0x48 + 4 * level.index())
    }

    fn set_bars(&mut self, level: Level, w: u32) {
        self.set_u32(0x48 + 4 * level.index(), w);
    }

    /// Bars found less bars spent (0x80094a0c).
    pub fn bars_available(&self) -> u32 {
        let count = |at: usize| (0..32).map(|k| self.u32(at + 4 * k).count_ones()).sum::<u32>();
        count(0x48).saturating_sub(count(0xc8))
    }

    /// Level `level` played (+0x34) and the levels moved on.
    pub fn enter(&mut self, level: Level) {
        let now = self.u32(0x00);
        self.set_u32(0x04, now);
        self.set_u32(0x00, level.0 as u32);
        let w = self.u32(0x34) | level.bit();
        self.set_u32(0x34, w);
    }

    /// Level `level` finished (0x80076d6c): played (+0x34), and when won
    /// the bonus bars, the level's puzzle piece, and the first completion
    /// (+0x38; levels 2-6 and 8-12 count to +0x22, the 2nd, 3rd, 5th and
    /// 7th leading to a bonus level). Nothing is kept in activity play.
    pub fn finish(&mut self, level: Level, won: bool, rand: &mut Rand) -> Finish {
        let mut f = Finish::default();
        if self.activity() {
            return f;
        }
        let played = self.u32(0x34) | level.bit();
        self.set_u32(0x34, played);
        if !won {
            return f;
        }
        // Bonus bars (0x80094b10): bits 31 down. The fourth bar some
        // levels give for a perfect run (0x80076ba8 and friends) is not
        // kept yet.
        let bars = match level {
            Level::COOKIE_RACE | Level::EGG_HUNT if self.won(level) => 0,
            Level::COOKIE_RACE | Level::RETURN_OF_MR_FRIEND => 2,
            Level::GOLD_RUSH | Level::EGG_HUNT | Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY => 3,
            _ => 0,
        };
        if bars > 0 {
            let w = (0..bars).fold(self.bars(level), |w, k| w | 1 << (31 - k));
            self.set_bars(level, w);
            f.bars = bars;
        }
        let mut all_placed = false;
        if self.pieces() < 12 {
            f.piece = self.give_piece(level, rand);
            all_placed = self.placed() == 12;
        }
        if all_placed && self.flags() & 512 == 0 {
            self.complete(level);
            f.next = Some(Level::REPTAR_2010);
        } else if !self.won(level) {
            self.complete(level);
            if level.is_episode() {
                self.block[0x22] = self.block[0x22].wrapping_add(1);
                let next = match self.block[0x22] {
                    2 => Some(Level::NO_MORE_COOKIES),
                    3 => Some(Level::MIRRORLAND),
                    5 => Some(Level::TOUCHDOWN_TOMMY),
                    7 => Some(Level::GOLD_RUSH),
                    _ => None,
                };
                if next.is_some() {
                    self.set_u32(0x30, self.flags() & !3);
                    f.next = next;
                }
            }
        }
        if level == Level::MIRRORLAND {
            self.set_u32(0x30, self.flags() | 16);
            f.next = Some(Level::HUB);
        }
        f
    }

    fn complete(&mut self, level: Level) {
        let done = self.u32(0x38) | level.bit();
        self.set_u32(0x38, done);
    }

    /// The end of the game (0x80018264): +0x22, +0x23, +0x30..+0x46 and
    /// both bar arrays cleared; then +0x30 = 4 (no Reptar Puzzle again).
    pub fn clear_for_ending(&mut self) {
        self.block[0x22] = 0;
        self.block[0x23] = 0;
        for b in &mut self.block[0x30..0x48] {
            *b = 0;
        }
        for b in &mut self.block[0x48..0x148] {
            *b = 0;
        }
        self.set_u32(0x30, 4);
    }

    /// The flags at +0x30.
    pub fn flags(&self) -> u32 {
        self.u32(0x30)
    }

    pub fn set_flags(&mut self, f: u32) {
        self.set_u32(0x30, f);
    }

    /// Activity play (+0x21): one level from the Activities menu.
    pub fn activity(&self) -> bool {
        self.block[0x21] != 0
    }

    pub fn set_activity(&mut self, on: bool) {
        self.block[0x21] = on as u8;
    }

    /// Pieces placed on the board (0x80091d28).
    pub fn placed(&self) -> u32 {
        (u16::from_le_bytes([self.block[0x46], self.block[0x47]]).count_ones()).min(12)
    }

    /// Piece `n` placed (0x80092754).
    pub fn place(&mut self, n: usize) {
        let w = u16::from_le_bytes([self.block[0x46], self.block[0x47]]) | 1 << n;
        self.block[0x46..0x48].copy_from_slice(&w.to_le_bytes());
    }

    /// Pieces earned (+0x44) and placed (+0x46), a bit each.
    pub fn piece_bits(&self) -> (u16, u16) {
        (
            u16::from_le_bytes([self.block[0x44], self.block[0x45]]),
            u16::from_le_bytes([self.block[0x46], self.block[0x47]]),
        )
    }

    /// The pieces on the board after the puzzle screen.
    pub fn set_placed_bits(&mut self, placed: u16) {
        self.block[0x46..0x48].copy_from_slice(&placed.to_le_bytes());
    }

    /// Whether piece `n` is earned (0x80091ca8).
    pub fn earned(&self, n: usize) -> bool {
        u16::from_le_bytes([self.block[0x44], self.block[0x45]]) >> n & 1 != 0
    }

    /// A random piece not yet earned (0x80091d50: rand % 12 until free,
    /// any with the final level reached).
    fn award_piece(&mut self, rand: &mut Rand) -> usize {
        loop {
            let n = (rand.next() % 12) as usize;
            if !self.earned(n) || self.flags() & 512 != 0 {
                let w = u16::from_le_bytes([self.block[0x44], self.block[0x45]]) | 1 << n;
                self.block[0x44..0x46].copy_from_slice(&w.to_le_bytes());
                return n;
            }
        }
    }

    /// 15 unspent bars buy a piece (0x80094924, 0x80091c14: the first
    /// found bars from level 1 up are marked spent).
    pub fn buy_piece(&mut self, rand: &mut Rand) -> bool {
        if self.pieces() >= 12 || self.bars_available() < 15 {
            return false;
        }
        let mut left = 15;
        'spend: for level in 1..32 {
            for n in 0..32 {
                let found = self.u32(0x48 + 4 * level) >> n & 1 != 0;
                let spent = self.u32(0xc8 + 4 * level) >> n & 1 != 0;
                if found && !spent {
                    let w = self.u32(0xc8 + 4 * level) | 1 << n;
                    self.set_u32(0xc8 + 4 * level, w);
                    left -= 1;
                    if left == 0 {
                        break 'spend;
                    }
                }
            }
        }
        self.award_piece(rand);
        true
    }

    /// Whether `level`'s own piece was given (0x80094bc0: level 25 reads
    /// as 8).
    pub fn piece_given(&self, level: Level) -> bool {
        let level = level.record();
        level.has_piece() && self.u32(0x40) & level.bit() != 0
    }

    /// The pieces a finished level gives (0x80094bfc): one bought with
    /// bars, and the level's own (levels 1-6, 8-12 and 15, 0x800919b4)
    /// once (+0x40). Whether the puzzle screen shows.
    pub fn give_piece(&mut self, level: Level, rand: &mut Rand) -> bool {
        if self.pieces() >= 12 {
            return false;
        }
        let mut given = self.buy_piece(rand);
        if level.has_piece() && self.u32(0x40) & level.bit() == 0 {
            let w = self.u32(0x40) | level.bit();
            self.set_u32(0x40, w);
            self.award_piece(rand);
            given = true;
        }
        given
    }

    /// Whether level `level` was won (+0x38).
    pub fn won(&self, level: Level) -> bool {
        self.u32(0x38) & level.bit() != 0
    }

    /// The level saved in (+0x00).
    pub fn level(&self) -> Level {
        Level(self.u32(0x00).clamp(1, 31) as u8)
    }

    /// Pieces earned (0x80094b8c).
    pub fn pieces(&self) -> u32 {
        (u16::from_le_bytes([self.block[0x44], self.block[0x45]]).count_ones()).min(12)
    }
}
