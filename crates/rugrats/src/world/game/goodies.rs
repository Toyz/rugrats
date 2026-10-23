//! Collectibles (bars, cookies and the levels' own) and the spawn spots.

use rugrats_data::placement::Spot;

use super::*;
use crate::actors::player::{DT, RADIUS};
use crate::rand::Rand;

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
    /// The placement it was made from.
    pub placement: usize,
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
pub fn spawn(spots: &mut [Spot], group: usize, rand: &mut Rand) -> Option<usize> {
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

/// A collectible taken this frame: its kind and +0x6d.
#[derive(Clone, Copy, Debug)]
pub struct Collected {
    pub kind: u16,
    pub value: i32,
}

impl Game {
    /// The collectibles the walk's probes meet (0x80063d3c over the nearby
    /// list): each not taken, not kind 16, its model's boxes (its first
    /// the cube of 375, 0x80048a44; kind 12's model 0) turned by its
    /// angles, sized as its set-up left it.
    pub(super) fn touchable(&self) -> Vec<crate::world::boxhit::Exact> {
        self.goodies
            .iter()
            .filter(|g| !g.taken && g.kind != 16)
            .filter_map(|g| {
                let shape = if g.kind == 12 { 0 } else { g.shape };
                let m = self.set.models.get(shape)?;
                let boxes = m.boxes.iter().map(|(a, b)| (a.map(|v| v as i32), b.map(|v| v as i32))).collect();
                let size = self.size_of(g.placement, shape);
                let solid = crate::world::boxhit::Solid { pos: boxes::ipos(g.pos), angles: boxes::angles_of(&self.trig, g.rot), bound: size, size, shape: crate::world::boxhit::Shape::Model(boxes) };
                Some(crate::world::boxhit::Exact { solid, climb: false, template: g.template })
            })
            .collect()
    }

    /// The collectible the `k`th of [`Self::touchable`] is.
    fn touchable_of(&self, k: usize) -> Option<usize> {
        self.goodies
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.taken && g.kind != 16 && self.set.models.get(if g.kind == 12 { 0 } else { g.shape }).is_some())
            .nth(k)
            .map(|(i, _)| i)
    }

    /// Touching collectibles (0x800514b0): taken, with their kind's sound.
    pub(super) fn touch(&mut self) {
        self.found.clear();
        self.collected.clear();
        self.bar_shown = (self.bar_shown - DT).max(0);
        let me = Vec3::new(self.player.pos[0] as f32, self.player.pos[1] as f32, self.player.pos[2] as f32);
        // A standing jump never eats a bone in level 11: the touch in the
        // air is skipped while 0x800c4bc4 is 1 (0x8005ed2c).
        let airborne = matches!(self.player.state, State::Jump | State::Fall);
        let no_bone = self.level == Level::CIRCUS && airborne && self.player.standing_jump;
        // On the ground the walk's probes touch it (`touchable`).
        let walked = self.player.touched.and_then(|k| self.touchable_of(k));
        // Kind 16 has no touch (0x80049244): decoration.
        for (i, g) in self.goodies.iter_mut().enumerate().filter(|(_, g)| !g.taken && g.kind != 16 && !(no_bone && g.kind == goodie::BONE)) {
            let r = self.set.models.get(g.shape).map_or(256.0, |m| m.radius as f32) + RADIUS as f32;
            let d = g.pos - me;
            let touched = if airborne {
                // (In the air, as near as the body's radius across.)
                Vec3::new(d.x, 0.0, d.z).length() < r && d.y.abs() < 1200.0
            } else {
                walked == Some(i)
            };
            if touched {
                g.taken = true;
                // Three sparks where it was (0x800514b0).
                let at = [g.pos.x as i32, g.pos.y as i32, g.pos.z as i32];
                self.flights.sparkle(g.template, at, g.value as i16, self.sprites.as_ref());
                self.collected.push(Collected { kind: g.kind, value: g.value });
                self.effects.push(Effect::new(goodie::sound(g.kind)));
                // Cookies and bones heal by their +0x6d (0x800514b0).
                if matches!(g.kind, goodie::COOKIE | goodie::BONE) {
                    self.player.heal(g.value);
                }
                // Messages 3 (a bar) and 7 (a cookie).
                match g.kind {
                    goodie::BAR => self.training.show(3),
                    goodie::COOKIE => {
                        self.training.show(7);
                        if self.level == Level::TRAINING {
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
