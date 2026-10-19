//! Class 10 objects (think SLUS_006.50:0x8004e0f4): each, a period after
//! it last went, calls its hit callback (+0x50, the table at 0x800b7eb0 by
//! template +0x09), which mostly sends out particles and sounds.
//!
//! The record the set-up builds for them (0x8004691c, +0x98, 16 bytes):
//! +0x00 gone last think, +0x02 the period in VSyncs (template +0x18),
//! +0x04 and +0x06 the cone's least and most (+0x1a, +0x1c, or 1), +0x08
//! how many (+0x1e), +0x0a a particle's life (+0x20), +0x0c its gravity
//! flags (+0x22 << 8), +0x0e its speed (+0x24).
//!
//! Each callback makes its launches (0x8008864c) about +y (-y for 7)
//! turned by the object's matrix (+0x04), most through 0x80052b60:
//! bounce 2048, no delay, damage the object's +0x6d (template +0x32).

use super::*;
use crate::world::flights::{self, Launch, Mark};

/// A class 10 object as its think keeps it.
#[derive(Clone, Copy, Debug)]
pub struct Emitter {
    pub placement: usize,
    /// Template +0x09: which callback.
    hit: u8,
    /// +0x64: the VSync it last went (or was made), and +0x98's +0x00.
    pub last: u32,
    pub gone: bool,
    /// When it was put on the update list (0x800c43b8): the list is walked
    /// from its head, where 0x80072ad4 puts the newest, so the latest
    /// scheduled thinks first.
    pub scheduled: u64,
    period: i32,
    min: i16,
    max: i16,
    count: i32,
    life: i32,
    gravity: u32,
    speed: i32,
}

/// The shared sound count's reset (0x800c4ba0).
const SOUND_EVERY: i32 = 100;
/// The face flags of most of the callbacks' sprites.
const FACE: u32 = 0xc080_0000;
/// The marks' half size.
const MARK_SIZE: u16 = 300;
/// How near the player 969 and 982 hurt as they go (squared, 0x8004e0f4).
const HURT_969: i64 = 0x90_0000;
const HURT_982: i64 = 0x40_0000;

/// The callbacks' seeds (`Seeds::emitters`) by callback.
const SEED_2: usize = 0;
const SEED_9: usize = 1;
const SEED_3: usize = 2;
const SEED_5: usize = 3;
const SEED_10: usize = 4;
const SEED_13: usize = 5;
const SEED_14: usize = 6;
const SEED_15: usize = 7;
const SEED_18: usize = 8;
const SEED_1: usize = 9;

/// The record (+0x98) the set-up builds for an object with a hit callback
/// (template +0x09, 0x8004691c), made at `vsync`.
pub(super) fn record(set: &Set, k: usize, vsync: u32) -> Option<Emitter> {
    let p = set.placements.get(k).filter(|p| p.kind == 1)?;
    let t = set.templates.get(p.template as usize).filter(|t| t.raw[9] != 0)?;
    let w = |at: usize| i16::from_le_bytes([t.raw[at], t.raw[at + 1]]);
    Some(Emitter {
        placement: k,
        hit: t.raw[9],
        last: vsync,
        gone: false,
        scheduled: 0,
        period: w(0x18) as i32,
        min: w(0x1a),
        max: if w(0x1c) == 0 { 1 } else { w(0x1c) },
        count: w(0x1e) as i32,
        life: w(0x20) as i32,
        gravity: (w(0x22) as u32) << 8,
        speed: w(0x24) as i32,
    })
}

impl Game {
    /// The class 10 objects of the level's loaded placements, made at
    /// `vsync` (template class +0x04 10 with a hit callback, +0x09).
    pub(super) fn emitters_of(set: &Set, vsync: u32) -> Vec<Emitter> {
        (0..set.placements.len())
            .filter(|&k| set.templates.get(set.placements[k].template as usize).is_some_and(|t| t.raw[4] == 10))
            .filter_map(|k| record(set, k, vsync))
            .collect()
    }

    /// A class 10 object's enter (0x8004cfd8): scheduled, its clock to
    /// start at its next think.
    pub(super) fn emitter_entered(&mut self, k: usize) {
        let stamp = self.emitters.iter().map(|e| e.scheduled).max().unwrap_or(0) + 1;
        if let Some(e) = self.emitters.iter_mut().find(|e| e.placement == k) {
            e.gone = true;
            e.scheduled = stamp;
        }
    }

    /// Each scheduled class 10 object's think (0x8004e0f4; the trigger
    /// pass's enter schedules it, its leave, 0x80049e5c, takes it off):
    /// gone last think, its clock starts again; past its period, its
    /// callback, and it has gone. 969 and 982 hurt the player by their
    /// +0x6d first when near.
    pub(super) fn emitter_thinks(&mut self) {
        // (VSync(-1) during the frame: `vsync_lag` before the frame's
        // end.)
        let now = self.vcount.wrapping_sub(self.vsync_lag);
        let mut order: Vec<usize> = (0..self.emitters.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(self.emitters[i].scheduled));
        for i in order {
            if self.trigger_active(Trigger::Placement(self.emitters[i].placement)) != Some(true) {
                continue;
            }
            let e = &mut self.emitters[i];
            if std::mem::take(&mut e.gone) {
                e.last = now;
            }
            if !(e.last < now && now - e.last > e.period as u32) {
                continue;
            }
            let e = self.emitters[i];
            let p = self.set.placements[e.placement];
            let near = match p.template {
                969 => Some(HURT_969),
                982 => Some(HURT_982),
                _ => None,
            };
            if let Some(near) = near {
                let d = [0, 1, 2].map(|k| (p.pos[k] - self.player.pos[k]) as i64);
                if d.iter().map(|v| v * v).sum::<i64>() <= near {
                    let damage = self.set.templates[p.template as usize].raw[0x32] as i32;
                    self.post_damage(damage);
                }
            }
            let angles = self.trig.placed_angles(p.rotation);
            let frame = self.placed_matrix(e.placement);
            self.emit(&e, p.pos, angles, frame);
            self.emitters[i].gone = true;
        }
    }

    /// Placement `k`'s object matrix (+0x04: 0x80020ee0 of its angles,
    /// MatrixNormal'd).
    pub fn placed_matrix(&self, k: usize) -> flights::Matrix {
        let angles = self.trig.placed_angles(self.set.placements[k].rotation);
        self.trig.matrix_normal(self.trig.euler(angles))
    }

    /// An object's hit callback (the table at 0x800b7eb0) from its record
    /// `e`: the object at `at` (+0x18), its angles (+0x24..+0x28)
    /// `angles` and matrix (+0x04) `frame`.
    pub(super) fn emit(&mut self, e: &Emitter, at: [i32; 3], angles: [i16; 3], mut frame: flights::Matrix) {
        let p = self.set.placements[e.placement];
        let damage = self.set.templates[p.template as usize].raw[0x32] as i16;
        let level = self.level.0;
        // 0x80052b60's part, and the record's own.
        let base = Launch { flags: e.gravity, at, life: e.life, speed: e.speed, bounce: 2048, damage, ..Launch::default() };
        let up = [0, 4096, 0];
        let speed_of = |seed: &mut u32| flights::draw15(seed) % e.speed.max(1);
        let mut out = Vec::new();
        match e.hit {
            // 0x80053604 (spinners 526, 738): the shared count's sound
            // (150, 178 in level 16; back to 1000); model 932 (flags
            // +0x0c | 0xb0002, 500 ms) in a cone about +z turned by the
            // object's first angle (0x80020bc0 of -(+0x24)) at a random
            // speed (0x800c41b8).
            1 => {
                self.shelf_sound -= DT;
                if self.shelf_sound <= 0 {
                    self.play_at(if level == 16 { 178 } else { 150 }, at);
                    self.shelf_sound = 1000;
                }
                let a = -(angles[0] as i32);
                let (c, s) = (self.trig.cos(a), self.trig.sin(a));
                let turn = [[c, 0, s], [0, 4096, 0], [-s, 0, c]];
                let radius = self.set.models.get(932).map_or(0, |m| m.radius as i32);
                let f = &mut self.flights;
                let dir = f.cone([0, 0, 4096], &turn, e.min, e.max);
                let speed = speed_of(&mut f.seeds.emitters[SEED_1]);
                out.push(Launch {
                    flags: ((e.gravity | 2) & 0xffff) | 0xb_0000,
                    radius,
                    life: 500,
                    dir,
                    speed,
                    model: Some(932),
                    ..base
                });
            }
            // 0x800537c4 (level 2's 20, each think): a drop (flags 0x505,
            // sprite 276, 50 by 200, 50 round, face 0xd0800000, 1000 ms,
            // spinning) at 50 about +y, up to 2048.
            8 => {
                let dir = self.flights.cone(up, &frame, 0, 2048);
                out.push(Launch {
                    flags: 0x505,
                    radius: 50,
                    life: 1000,
                    dir,
                    speed: 50,
                    spin: 3391,
                    sprite: Some(276),
                    size: [50, 200],
                    face: 0xd080_0000,
                    ..base
                });
            }
            // 0x80053b88 (level 5's robots 826, each think): a glow (flags
            // 0x2010705, sprite 374, 200 across and round, face 0xd2800000,
            // 1000 ms) straight up at 146 (its own LCG from 0x55fe7, one
            // draw, % 250).
            11 => {
                let dir = self.flights.cone(up, &frame, 0, 1);
                let mut seed = 0x55fe7;
                out.push(Launch {
                    flags: 0x201_0705,
                    radius: 200,
                    life: 1000,
                    dir,
                    speed: flights::draw15(&mut seed) % 250,
                    spin_axis: [2048; 3],
                    sprite: Some(374),
                    size: [200; 2],
                    face: 0xd280_0000,
                    ..base
                });
            }
            // 0x80052ba0 (577): `count` drops (sprite 267, 50 across) at
            // random speeds that leave mark 209. 0x800539b4 (982): sound
            // 81 at it, then the same 150 across with no mark. The
            // object's matrix loses its third row (+0x10-+0x14) first.
            2 | 10 => {
                frame[2] = [0; 3];
                let (seed, size, mark) = if e.hit == 2 { (SEED_2, 50, Some(209)) } else { (SEED_10, 150, None) };
                if e.hit == 10 {
                    self.play_at(81, at);
                }
                let f = &mut self.flights;
                for _ in 0..e.count {
                    let speed = speed_of(&mut f.seeds.emitters[seed]);
                    let dir = f.cone(up, &frame, e.min, e.max);
                    out.push(Launch {
                        flags: e.gravity | 0x20b_0005,
                        dir,
                        speed,
                        sprite: Some(267),
                        size: [size; 2],
                        face: FACE,
                        mark: mark.map(|sprite| Mark { sprite, size: MARK_SIZE, flags: [0, 1, 1] }),
                        ..base
                    });
                }
            }
            // 0x80052ed8 (611): sound 149 at it; a drop (sprite 264, 100
            // across and round, spinning) at a random speed that sounds
            // 133 (177 in level 16) and leaves mark 263.
            3 => {
                self.play_at(149, at);
                let f = &mut self.flights;
                let dir = f.cone(up, &frame, e.min, e.max);
                let speed = speed_of(&mut f.seeds.emitters[SEED_3]);
                out.push(Launch {
                    flags: e.gravity | 0x220f_0005,
                    radius: 100,
                    dir,
                    speed,
                    spin: 3391,
                    sprite: Some(264),
                    size: [100; 2],
                    face: FACE,
                    sound: if level == 16 { 177 } else { 133 },
                    mark: Some(Mark { sprite: 263, size: MARK_SIZE, flags: [0, 1, 0] }),
                    ..base
                });
            }
            // 0x80053090 (614): model 931 (kind 2), spinning.
            4 => {
                let radius = self.set.models.get(931).map_or(0, |m| m.radius as i32);
                let dir = self.flights.cone(up, &frame, e.min, e.max);
                out.push(Launch { flags: e.gravity | 0x3_0002, radius, dir, spin: 3391, model: Some(931), ..base });
            }
            // 0x80053170 (615): one of six sprites at random, 50 across
            // and round, spinning.
            5 => {
                let f = &mut self.flights;
                let sprite = [269, 270, 271, 273, 272, 274][(flights::draw15(&mut f.seeds.emitters[SEED_5]) % 6) as usize];
                let dir = f.cone(up, &frame, e.min, e.max);
                out.push(Launch {
                    flags: e.gravity | 0x7_0005,
                    radius: 50,
                    dir,
                    spin: 3391,
                    sprite: Some(sprite),
                    size: [50; 2],
                    face: FACE,
                    ..base
                });
            }
            // 0x80053344: none in level 12; a sound at it when the shared
            // count runs out (level 29: rand % 3 of 134, 143, 140; else
            // rand % 10 of 151-160), then a bubble (sprite 266, face
            // 0xd2800000, 150 across in levels 7 and 29, else 50).
            6 => {
                if level == 12 {
                    return;
                }
                self.shelf_sound -= DT;
                if self.shelf_sound <= 0 {
                    let r = self.rand.next() & 255;
                    let sound = if level == 29 { [134, 143, 140][(r % 3) as usize] } else { 151 + (r % 10) as u8 };
                    self.play_at(sound, at);
                    self.shelf_sound = SOUND_EVERY;
                }
                let size = if matches!(level, 7 | 29) { 150 } else { 50 };
                let dir = self.flights.cone(up, &frame, e.min, e.max);
                out.push(Launch {
                    flags: e.gravity | 0x3_0005,
                    dir,
                    sprite: Some(266),
                    size: [size; 2],
                    face: 0xd280_0000,
                    ..base
                });
            }
            // 0x800538bc (706): model 1040 (kind 2) down, sounding 147
            // and leaving mark 218 where it meets something.
            7 => {
                let dir = self.flights.cone([0, -4096, 0], &frame, e.min, e.max);
                out.push(Launch {
                    flags: e.gravity | 0x201_0002,
                    dir,
                    model: Some(1040),
                    sound: 147,
                    // (+0x4c: the object's +0x5c, 0x800b7ea4 by template
                    // +0x0b.)
                    on_hit: self.set.templates[p.template as usize].raw[0x0b],
                    mark: Some(Mark { sprite: 218, size: MARK_SIZE, flags: [1, 0, 0] }),
                    ..base
                });
            }
            // 0x80052d70 (682): model 1016 (kind 2) at a random speed,
            // leaving mark 212.
            9 => {
                let f = &mut self.flights;
                let dir = f.cone(up, &frame, e.min, e.max);
                let speed = speed_of(&mut f.seeds.emitters[SEED_9]);
                out.push(Launch {
                    flags: e.gravity | 0x20b_0002,
                    dir,
                    speed,
                    model: Some(1016),
                    mark: Some(Mark { sprite: 212, size: MARK_SIZE, flags: [0, 1, 1] }),
                    ..base
                });
            }
            // 0x80053ef8 (969): sound 79 at it; six of seven sprites at
            // random (381-383 225 across and 150 round, 384-387 150 and
            // 35), each at a random speed in a cone of random width,
            // starting the period in ms after.
            13 => {
                self.play_at(79, at);
                let f = &mut self.flights;
                for _ in 0..6 {
                    let pick = flights::draw15(&mut f.seeds.emitters[SEED_13]) % 7;
                    let (size, radius) = if pick < 3 { (225, 150) } else { (150, 35) };
                    let speed = speed_of(&mut f.seeds.emitters[SEED_13]);
                    let max = flights::draw15(&mut f.seeds.emitters[SEED_13]) % (e.max as i32).max(1);
                    let dir = f.cone(up, &frame, e.min, max as i16);
                    out.push(Launch {
                        flags: e.gravity | 0x1_0005,
                        radius,
                        dir,
                        speed,
                        spin: 3391,
                        delay: e.period,
                        sprite: Some(381 + pick as u32),
                        size: [size; 2],
                        face: FACE,
                        ..base
                    });
                }
            }
            // 0x80054238 (905: sound 83, model 1455) and 0x8005444c
            // (906-908: sound 82, model 1454, 1452 or 1453 at random): one
            // model (flags 0xb0602, 1000 ms) along (-4096, r, 0) or (0, r,
            // 4096), r a random -2048..-1649, turned by the object's
            // matrix (0x8009f6f8), at 80-119.
            14 | 15 => {
                let (sound, seed) = if e.hit == 14 { (83, SEED_14) } else { (82, SEED_15) };
                self.play_at(sound, at);
                let s = &mut self.flights.seeds.emitters[seed];
                let model = if e.hit == 14 { 1455 } else { [1454, 1452, 1453][(flights::draw15(s) % 3) as usize] };
                let r = flights::draw15(s) % 400 - 2048;
                let speed = flights::draw15(s) % 40 + 80;
                let along = if e.hit == 14 { [-4096, r, 0] } else { [0, r, 4096] };
                out.push(Launch {
                    flags: 0xb_0602,
                    at: at,
                    life: 1000,
                    dir: flights::turned(&frame, along),
                    speed,
                    model: Some(model),
                    damage,
                    ..Launch::default()
                });
            }
            // 0x80054d5c (1215): model 1628 (kind 2) at a random speed,
            // bounce 2500, starting the period in ms after.
            18 => {
                let f = &mut self.flights;
                let dir = f.cone(up, &frame, e.min, e.max);
                let speed = speed_of(&mut f.seeds.emitters[SEED_18]);
                out.push(Launch {
                    flags: e.gravity | 0xb_0002,
                    dir,
                    speed,
                    bounce: 2500,
                    delay: e.period,
                    model: Some(1628),
                    ..base
                });
            }
            _ => {}
        }
        for l in &out {
            self.flights.launch(l, self.sprites.as_ref());
        }
    }
}

impl Game {
    /// The characters' hit callbacks (+0x50) at their thinks' start
    /// (0x80084084), as `npc::step` found them: where they were as the
    /// think began, their body's matrix (0x80020d10 of the heading).
    pub(super) fn npc_hits(&mut self) {
        let now = self.vcount.wrapping_sub(self.vsync_lag);
        let thinks: Vec<(usize, [i32; 3])> =
            self.npc_events.iter().filter_map(|e| if let npc::Event::Think(k, at) = *e { Some((k, at)) } else { None }).collect();
        for (k, at) in thinks {
            let n = &self.npcs[k];
            let Some(e) = n.placement.and_then(|p| record(&self.set, p, now)) else { continue };
            let angles = [n.heading as i16, 0, 0];
            let frame = self.trig.rotation(angles);
            self.emit(&e, at, angles, frame);
        }
    }
}

impl Game {
    /// The collectibles' think (0x8004d570, from their draw 0x800324a0)
    /// for kind 16 (level 4's 1141) in `room` as it is drawn: dt off the
    /// shared count; at 0 or under, sound 146 at it and the count back to
    /// 500.
    pub(super) fn goodie_thinks(&mut self, room: usize) {
        for i in 0..self.goodies.len() {
            let g = &self.goodies[i];
            let at = [g.pos.x as i32, g.pos.y as i32, g.pos.z as i32];
            if g.kind != 16 || g.taken || self.room_of(at) != Some(room) {
                continue;
            }
            self.shelf_sound -= DT;
            if self.shelf_sound <= 0 {
                self.play_at(146, at);
                self.shelf_sound = 500;
            }
        }
    }
}
