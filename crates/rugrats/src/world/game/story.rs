//! Cries, tries and the cinematics that end a level.

use super::*;
use crate::actors::player::anim::CRY;

/// What a character says when a thrown thing hits it (0x800511a8): Angelica
/// (3) 556 or 557, char 10 1065, char 11 1076 or 1079, Chuckie (2) 185 in
/// level 2.
pub fn hit_line(character: usize, level: Level, r: u32) -> Option<u16> {
    match character {
        3 => Some(if r != 0 { 556 } else { 557 }),
        10 => Some(1065),
        11 => Some(if r != 0 { 1076 } else { 1079 }),
        2 if level == Level::GRANDPAS_TEETH => Some(185),
        _ => None,
    }
}

/// The cinematics that end a level (0x8007737c, table 0x8001532c), and
/// whether it was won: 5 (level 2), 14 (3), 19 (9), 20 and 21 (13), 24
/// (18), 26 (17), 35 (8), 40 (5), 47 (6), 48 (4), 50 (21), 62 (11), 64
/// (20), 68 (12), 72 (15), 78 (10).
pub(super) const ENDINGS: [(usize, bool); 18] = [
    (5, true),
    (14, true),
    (19, true),
    (20, false),
    (21, true),
    // Egg Hunt's loss: its last try is gone before cine 23 plays.
    (23, false),
    (24, true),
    (26, true),
    (35, true),
    (40, true),
    (47, true),
    (48, true),
    (50, true),
    (62, true),
    (64, true),
    (68, true),
    (72, true),
    (78, true),
];

/// Tries a level gives (s8 by level).
pub(super) const TRIES: u32 = 0x800b_2920;

/// The cry's longest run (0x8007f920: 2100 dt).
pub(super) const CRY_TIME: i32 = 2100;

/// Cine 85: the end of the game (exit code -1).
pub(super) const REPTAR_DANCE: usize = 85;

/// A level's start cinematic (0x80074958).
pub fn start_cine(level: Level) -> Option<usize> {
    Some(match level.0 {
        1 => 7,
        2 => 1,
        3 => 2,
        4 => 28,
        5 => 37,
        6 => 45,
        7 => 79,
        9 => 17,
        10 => 44,
        11 => 52,
        12 => 66,
        13 => 6,
        15 => 73,
        17 => 12,
        18 => 25,
        20 => 63,
        21 => 49,
        // Golf: "Golf Cam n" by level; level 25 the intro (main plays
        // "Golf Cam 3", 31, when coming from a hole).
        23 => 29,
        24 => 30,
        25 => 34,
        26 => 32,
        27 => 33,
        28 => 81,
        29 => 82,
        30 => 83,
        31 => 84,
        _ => return None,
    })
}

impl Game {
    /// ForceCry (0x8007fd0c): the player cries (0x8007f920), then a try
    /// is gone.
    pub fn cry(&mut self) {
        if self.crying > 0 || self.exit.is_some() || self.cine.is_some() {
            return;
        }
        if !self.script.as_ref().is_none_or(|s| s.cry_animation()) {
            self.cried();
            return;
        }
        self.crying = CRY_TIME;
        self.player.act(CRY, &self.anims);
        // 0x8007f920: characters 1-5 say 2040 + character at once
        // (0x8008df78).
        if (1..=5).contains(&self.character) {
            self.say_now = Some(2040 + self.character as u16);
        }
    }

    /// After the cry: health back to 255; the level's own respawn, or by
    /// default the last checkpoint; with no tries left, the hub (not from
    /// levels 1, 20 and 21).
    pub(super) fn cried(&mut self) {
        self.player.health = 255;
        self.player.action = None;
        if self.with_script(|s, g| s.cry(g)) {
            return;
        }
        if self.tries > 0 {
            self.tries -= 1;
            if self.tries == 0 {
                if !matches!(self.level, Level::HUB | Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY) {
                    self.exit = Some(Exit::to(Level::HUB));
                }
                return;
            }
        }
        if let Some(Pose { pos: at, heading }) = self.checkpoint {
            self.player.pos = at;
            self.player.heading = heading;
            self.player.target = heading;
            self.player.speed = 0;
            self.yaw = heading as f32;
            self.eye = self.eye_goal();
        }
    }

    /// A cinematic asked for has played (CineFinish, 0x800783d8): the
    /// endings, and the level's own reaction.
    pub fn cine_finished(&mut self, n: usize) {
        self.room = None;
        self.eye = self.eye_goal();
        rrt::tracing::info!("level {}: cine {n} finished", self.level);
        // The ending cinematics (0x8007737c): back to the hub with the
        // level recorded won or lost; Reptar Dance ends the game.
        match ENDINGS.iter().find(|e| e.0 == n) {
            Some(&(_, won)) => {
                self.won = Some(won);
                self.exit = Some(Exit::to(Level::HUB));
            }
            // The end (0x8007737c case 84): exit -1, the credits.
            None if n == REPTAR_DANCE => self.ending = true,
            None => {}
        }
        self.with_script(|s, g| s.cine_finished(g, n));
    }
}
