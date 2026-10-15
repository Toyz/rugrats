//! Level numbers (the state's +0x00, the exit code 0x800c4918) and the
//! rules the engine keys on them.

use std::fmt;

/// A level number, 0-31: the hub, the episodes, the bonus levels and the
/// golf holes. 0 is none (the front end).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level(pub u8);

impl Level {
    pub const NONE: Level = Level(0);
    /// Pickles' Home.
    pub const HUB: Level = Level(1);
    pub const GRANDPAS_TEETH: Level = Level(2);
    pub const AISLE_SEVEN: Level = Level(3);
    pub const CYNTHIA: Level = Level(4);
    pub const OUTER_SPACE: Level = Level(5);
    pub const TOY_PALACE: Level = Level(6);
    pub const REPTAR_2010: Level = Level(7);
    /// The golf course as a whole: its record, piece and bars.
    pub const GOLF: Level = Level(8);
    pub const MR_FRIEND: Level = Level(9);
    pub const CHUCKIES_GLASSES: Level = Level(10);
    pub const CIRCUS: Level = Level(11);
    pub const LET_THERE_BE_LIGHT: Level = Level(12);
    pub const COOKIE_RACE: Level = Level(13);
    pub const RETURN_OF_MR_FRIEND: Level = Level(14);
    pub const MIRRORLAND: Level = Level(15);
    pub const TRAINING: Level = Level(16);
    pub const GOLD_RUSH: Level = Level(17);
    pub const EGG_HUNT: Level = Level(18);
    pub const NO_MORE_COOKIES: Level = Level(20);
    pub const TOUCHDOWN_TOMMY: Level = Level(21);
    /// The golf holes, 23-31; the hub's golf door leads to 25.
    pub const FIRST_HOLE: Level = Level(23);
    pub const GOLF_DOOR: Level = Level(25);
    pub const LAST_HOLE: Level = Level(31);

    /// For indexing tables by level.
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// The level's bit in the state's level masks (+0x34, +0x38, +0x40).
    pub fn bit(self) -> u32 {
        1 << (self.0 & 31)
    }

    pub fn is_hole(self) -> bool {
        (23..=31).contains(&self.0)
    }

    /// The two box toss levels (20, 21).
    pub fn is_box_toss(self) -> bool {
        matches!(self.0, 20 | 21)
    }

    /// The level whose record a finish keeps: the golf door's hole reads
    /// as the course (0x80094bc0).
    pub fn record(self) -> Level {
        if self == Level::GOLF_DOOR { Level::GOLF } else { self }
    }

    /// Levels 2-6 and 8-12: the episodes counted toward the bonus levels
    /// (+0x22).
    pub fn is_episode(self) -> bool {
        matches!(self.0, 2..=6 | 8..=12)
    }

    /// Levels with a puzzle piece of their own (0x800919b4): 1-6, 8-12
    /// and 15.
    pub fn has_piece(self) -> bool {
        matches!(self.0, 1..=6 | 8..=12 | 15)
    }

    /// Levels where Damage is dropped (the hub, 11, 17, the holes).
    pub fn no_damage(self) -> bool {
        matches!(self.0, 1 | 11 | 17 | 23..=31)
    }

    /// Levels that show the health meter.
    pub fn has_meter(self) -> bool {
        matches!(self.0, 2..=6 | 9 | 12 | 14 | 16)
    }

    /// The playable character's hierarchy (0x80018598): Spike (6) in 4
    /// and 11, else Tommy (1) but for 5, 6, 7, 10 and 17.
    pub fn character(self) -> usize {
        match self.0 {
            4 | 11 => 6,
            5 => 3,
            7 => 9,
            6 | 10 => 2,
            17 => 5,
            _ => 1,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Level> for usize {
    fn from(l: Level) -> usize {
        l.index()
    }
}
