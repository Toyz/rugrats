//! Collectibles (bars, cookies and the levels' own) and the spawn spots.

use super::*;

/// A collectible (0x80048a44, touched by 0x800514b0).
pub struct Goodie {
    /// A bar's number in its level.
    pub number: usize,
    pub shape: u32,
    pub kind: u16,
    /// Its template and spawn group (LP +0x48).
    pub template: u16,
    pub group: u16,
    /// +0x6d: what a bone or a cookie heals.
    pub value: i32,
    pub pos: Vec3,
    pub rot: Mat3,
    pub taken: bool,
}

/// Collectible kinds and their sounds (0x800514b0).
pub mod goodie {
    pub const BAR: u16 = 3;
    pub const COOKIE: u16 = 4;
    pub const BONE: u16 = 14;
    pub fn sound(kind: u16) -> u8 {
        match kind {
            1 => 30,
            2 | 15 | 17 => 31,
            3 => 28,
            4 | 14 => 29,
            6..=11 => 113,
            _ => 30,
        }
    }
}

/// How long the bar counter shows after a bar (dt units; the health
/// meter's 1500).
pub(super) const COUNTER_TIME: i32 = 1500;

/// A spot of spawn group `group` (0x8004fe14): the groups are runs of
/// spots ending at threshold 100; r = (rand & 255) % 100 picks the first
/// unused spot whose threshold is above it, else retries with r = 0, at
/// most five rounds. The spot is used.
pub fn spawn(spots: &mut [rugrats_data::placement::Spot], group: usize, rand: &mut crate::rand::Rand) -> Option<usize> {
    let (mut start, mut n) = (0, 0);
    let mut range = None;
    for (i, s) in spots.iter().enumerate() {
        if s.threshold >= 100 {
            if n == group {
                range = Some(start..i + 1);
                break;
            }
            n += 1;
            start = i + 1;
        }
    }
    let range = range?;
    for round in 0..5 {
        let r = if round == 0 { (rand.next() & 255) % 100 } else { 0 } as u8;
        if let Some(i) = range.clone().find(|&i| !spots[i].used && spots[i].threshold > r) {
            spots[i].used = true;
            return Some(i);
        }
    }
    None
}

impl Game {
    /// Touching collectibles (0x800514b0): taken, with their kind's sound.
    pub(super) fn touch(&mut self) {
        self.found.clear();
        self.collected.clear();
        self.bar_shown = (self.bar_shown - crate::actors::player::DT).max(0);
        let me = Vec3::new(self.player.pos[0] as f32, self.player.pos[1] as f32, self.player.pos[2] as f32);
        // Kind 16 has no touch (0x80049244): decoration.
        for g in self.goodies.iter_mut().filter(|g| !g.taken && g.kind != 16) {
            let r =
                self.set.models.get(g.shape).map_or(256.0, |m| m.radius as f32) + crate::actors::player::RADIUS as f32;
            let d = g.pos - me;
            if Vec3::new(d.x, 0.0, d.z).length() < r && d.y.abs() < 1200.0 {
                g.taken = true;
                self.collected.push((g.kind, g.value));
                self.effects.push((goodie::sound(g.kind), crate::audio::sound::EFFECTS_VOLUME));
                // Cookies and bones heal by their +0x6d (0x800514b0).
                if matches!(g.kind, goodie::COOKIE | goodie::BONE) {
                    self.player.health = (self.player.health + g.value).min(255);
                }
                // Messages 3 (a bar) and 7 (a cookie).
                match g.kind {
                    goodie::BAR => self.training.show(3),
                    goodie::COOKIE => {
                        self.training.show(7);
                        if self.level == 16 {
                            self.speech.push(216);
                        }
                    }
                    _ => {}
                }
                if g.kind == goodie::BAR {
                    self.bars += 1;
                    self.bar_shown = COUNTER_TIME;
                    self.found.push(g.number);
                }
            }
        }
    }
}
