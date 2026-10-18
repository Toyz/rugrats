//! Things that can be picked up, carried, thrown and put down, their
//! flights, and the breakables.

use super::*;
use crate::actors::npc::Mode;
use crate::actors::player::anim::{PICK_UP, PUT_DOWN, THROW};
use crate::actors::player::{DT, State};
use crate::world::collide;

/// A breakable (class 14, 0x8004d7c4): touched, its model becomes shape +
/// 1 and its template t + 1 (the rubble, which with template flag 8 no
/// longer blocks); template 1240 is removed.
pub struct Breakable {
    pub template: u16,
    pub shape: u32,
    pub pos: Vec3,
    pub rot: Mat3,
    pub broken: bool,
    pub gone: bool,
}

/// A model that can be picked up (object flag 0x10000).
#[derive(Clone)]
pub struct Thing {
    /// Its placement, template and registry id; +0x6d (template +0x32).
    pub placement: usize,
    pub template: u16,
    pub id: u16,
    pub damage: i32,
    /// Hidden (off in the world, 0x80078bc4 or a level's pool).
    pub hidden: bool,
    /// Carried but not thrown (class 13: object bit 0x1000000): ACTION
    /// is the level's (the remote's zap).
    pub keep: bool,
    /// Its flights meet the world only (0x80061f88: levels 20 and 21's
    /// box or bottle, flags 0x9010700); other thrown things meet objects
    /// too (0x9070700).
    pub world_only: bool,
    pub shape: u32,
    pub pos: Vec3,
    pub rot: Mat3,
    pub state: ThingState,
    /// What its flight last hit (record +0x98): an NPC, or the player as
    /// `PLAYER_HIT`; and whether it has bounced off anything.
    pub last_hit: Option<usize>,
    pub bounced: bool,
    /// Its flight as the original's record keeps it (0x8008864c): a throw,
    /// a lob or a put-down.
    pub flown: Option<Flown>,
}

/// A thrown thing's flight record (0x8008864c from 0x80061f88): where it
/// is (the object's position, which the flight writes, +0x30 of the
/// record), its radius (the larger of the first box's y extents) and
/// bounce (+0x28: the template's +0x36, at most 4095), and whether it
/// meets objects (flag 0x20000; levels 20 and 21's lobs meet the world
/// only), and whether it rolls (flag 0x10000000, the object's 0x800000:
/// template +0x30 of 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flown {
    pub pos: [i32; 3],
    pub radius: i32,
    pub bounce: i32,
    pub objects: bool,
    pub rolls: bool,
    /// Its gravity (flags & 0xf00, 0x80087c5c): 13 for 0x700, 1 for the
    /// pies' 0x400.
    pub gravity: i32,
    /// Done at a hit (flag 0x2000000 and result bit 1: the pies).
    pub ends_on_hit: bool,
    /// What it meets is met by a segment of its reach's radius (flag
    /// 0x40000000: the pies).
    pub grows: bool,
}

/// The pies' mark (0x80061c98: desc +0x46 284, +0x68 900).
const PIE_MARK: u32 = 284;
const PIE_MARK_SIZE: i32 = 900;

/// A pie toss (levels 3 and 16, +0x30 bit 8): the six pies (class 9,
/// 0x800e2770) thrown one after another from the pie spot.
#[derive(Clone, Copy, Debug)]
pub struct PieToss {
    /// The pie in hand (0x800c4134, 0..5, among the pies).
    pub next: usize,
    /// The pies' home: (0, 0, 512) turned by the player plus his place
    /// at the start (0x80049818).
    pub spot: [i32; 3],
    /// Whose distance picks the speed (0x80061c98): level 3's Larry (75,
    /// in 3D) or level 16's Angelica (600, across).
    pub target: u16,
    pub flat: bool,
}

/// A flight done at a hit (flag 0x2000000): the thing, where it stopped
/// and the normal it met (its +0x5c and record +0x80 callbacks).
#[derive(Clone, Copy, Debug)]
pub struct Ended {
    pub thing: usize,
    pub at: [i32; 3],
    pub normal: [i32; 3],
    /// The point it met, and the tile there if it was the world.
    pub point: [i32; 3],
    pub world: bool,
}

/// The mark a flight leaves where it hits (0x8008652c with record +0x2a,
/// the pies' 284): sprite `sprite` laid on the face met, a square of
/// half side `size` (+0x8c), turned by (x, y) = (-asin(-ny),
/// ratan2(-nx, -nz)) of the normal (0x8008e6c8), grey the COL tile's
/// shade there (+5) or 128. A pool entry without +0x08 bit 1
/// (0x8008e36c), it is drawn the frame it is made and freed (0x8008e694).
#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub sprite: u32,
    pub at: [i32; 3],
    pub angles: [i16; 3],
    pub size: i32,
    pub grey: u8,
}

/// Who a flight's segment can meet (0x80086340's room lists): an object
/// by its template, a character, or the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Met {
    Object(u16),
    Npc(usize),
    Player,
}

/// A throw asked for, waiting for its release: the carried thing, aimed
/// at release as a plain or level 9 throw, or with a level's velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PendingThrow {
    Aimed(usize),
    Given(usize, [i32; 3]),
    /// Put down (state 8, 0x80060910): let go at the animation's event
    /// 2048, where it has carried the thing out in front, then dropped
    /// ahead (0x80062ac4).
    Down(usize),
}

/// The throw's animation speed (0x8005d11c: 0x800572a8(player, 18,
/// 0x2000)).
pub const THROW_ANIM_SPEED: i32 = 0x2000;

/// [`Thing::last_hit`] for the player.
pub(super) const PLAYER_HIT: usize = usize::MAX;

/// A template's bounce (+0x36, into the object's +0xa2, clamped to 4095).
fn bounce_of(templates: &[Template], template: u16) -> i32 {
    templates.get(template as usize).map_or(0, |t| u16::from_le_bytes([t.raw[0x36], t.raw[0x37]]).min(4095) as i32)
}

/// Whether a template's flights take 0x80087594's path (template +0x30
/// not 1: flags without 0x10000000), which hurts any class 1 object,
/// the player too.
fn hurts_player(templates: &[Template], template: u16) -> bool {
    templates.get(template as usize).is_some_and(|t| t.raw[0x30] != 1)
}

impl Game {
    /// What a flight's segment can meet (0x80086340 -> 0x8006a824 over the
    /// rooms' lists): the objects and characters as the player's probes
    /// see them, but those of type flag 8, and the player by its
    /// hierarchy's box.
    fn flight_targets(&self, solids: &[crate::world::boxhit::Exact], npc_of: &[Option<usize>]) -> Vec<(crate::world::boxhit::Solid, Met)> {
        let mut out: Vec<_> = solids
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match npc_of.get(i).copied().flatten() {
                Some(n) => Some((e.solid.clone(), Met::Npc(n))),
                None if self.set.templates.get(e.template as usize).is_some_and(|t| t.flags & 8 != 0) => None,
                None => Some((e.solid.clone(), Met::Object(e.template))),
            })
            .collect();
        let h = |at: usize| self.player.hierarchy.header.get(at..at + 2).map_or(0, |v| i16::from_le_bytes([v[0], v[1]]) as i32);
        let shape = crate::world::boxhit::Shape::Body([h(4), h(6), h(8)], [h(0xc), h(0xe), h(0x10)]);
        let me = crate::world::boxhit::Solid { pos: self.player.pos, angles: [self.player.heading as i16, 0, 0], bound: 1773, size: 8192, shape };
        out.push((me, Met::Player));
        out
    }

    /// The step from `from` to `to` against the targets and the world
    /// (0x80086668): the nearest target crossed (0x8006a714: past the
    /// start, the first of the nearest), then the world's segment
    /// (0x8002cd7c), kept over the target only when nearer on all three
    /// axes. What was met, the point and the normal.
    fn flight_meets(&self, from: [i32; 3], to: [i32; 3], objects: bool, grow: i32, targets: &[(crate::world::boxhit::Solid, Met)]) -> Option<(Option<(Met, [i32; 3])>, [i32; 3], [i32; 3])> {
        let mut met: Option<(Met, [i32; 3], [i32; 3], [i32; 3])> = None;
        if objects {
            let mut best = 0x1001;
            for (solid, who) in targets {
                let mut n = [0; 3];
                if let Some((_, h)) = solid.meet_grown(&self.trig, from, to, grow, &mut n)
                    && h.t > 0
                    && h.t < best
                {
                    best = h.t;
                    met = Some((*who, solid.pos, h.point, n));
                }
            }
        }
        let mut seg = collide::Segment::default();
        if collide::segment(&self.col, from, to, &mut seg) != 0 {
            let nearer = |p: [i32; 3]| (0..3).all(|k| (from[k] - seg.point[k]).abs() < (from[k] - p[k]).abs());
            if met.is_none_or(|m| nearer(m.2)) {
                return Some((None, seg.point, seg.normal));
            }
        }
        met.map(|(who, at, p, n)| (Some((who, at)), p, n))
    }

    /// A shot's step from `from` to `to` (0x80090554, the shots after the
    /// flights): the nearest of this frame's solids its segment crosses
    /// (0x8006a9cc -> 0x8006a714: not the player, its owner, nor a
    /// template of flag 16; the boxes not grown; the first of the
    /// nearest, from the start on), unless the world's segment (0x8002cd7c)
    /// is nearer on all three axes. The character met, if one was.
    pub fn shot_meets(&self, from: [i32; 3], to: [i32; 3]) -> Option<usize> {
        let mut met: Option<(Option<usize>, [i32; 3])> = None;
        let mut best = i32::MAX;
        for (e, npc) in self.solids.iter().zip(&self.npc_of) {
            if npc.is_none() && self.set.templates.get(e.template as usize).is_some_and(|t| t.flags & 16 != 0) {
                continue;
            }
            let mut n = [0; 3];
            if let Some((_, h)) = e.solid.meet_grown(&self.trig, from, to, 0, &mut n)
                && h.t >= 0
                && (met.is_none() || h.t < best)
            {
                best = h.t;
                met = Some((*npc, h.point));
            }
        }
        let (npc, point) = met?;
        let mut seg = collide::Segment::default();
        if collide::segment(&self.col, from, to, &mut seg) != 0
            && (0..3).all(|k| (from[k] - seg.point[k]).abs() < (from[k] - point[k]).abs())
        {
            return None;
        }
        npc
    }

    /// One frame of a thrown thing's flight (0x80087c5c): gravity 13
    /// (flags 0x700) by 0x8004fbb4 (p += ((a dt dt >> 1) + v dt) >> 6, v
    /// += a dt), then 0x80087594: the step on by the radius (10 at least)
    /// along the velocity met (0x80086668); what it meets touched or hurt;
    /// off a surface met head on (0x8007666c: over 1024) it backs off the
    /// radius from the point and goes back along the reflection at (|v| -
    /// 95) bounce >> 12, or stops slower than 95 + 382 with something
    /// within its radius under it (0x80087414); stopped, the flight ends
    /// (flag 0x1000000).
    fn fly_exact(&mut self, k: usize, targets: &[(crate::world::boxhit::Solid, Met)]) -> Option<[i32; 3]> {
        let dt = DT;
        let (Some(mut f), ThingState::Flying(mut v)) = (self.things[k].flown, self.things[k].state) else { return None };
        if f.rolls {
            return self.roll_exact(k, targets);
        }
        let mut touched = None;
        let old = f.pos;
        let a = [0, f.gravity, 0];
        for i in 0..3 {
            let ad = a[i] * dt;
            f.pos[i] += (((ad * dt) >> 1) + v[i] * dt) >> 6;
            v[i] += ad;
        }
        let reach = f.radius.max(10);
        let u = collide::unit(v);
        let to = [0, 1, 2].map(|i| f.pos[i] + ((u[i] * reach) >> 12));
        let mut bits = 0;
        let mut normal = [0; 3];
        let mut point = f.pos;
        let mut met_world = false;
        if let Some((who, p, n)) = self.flight_meets(old, to, f.objects, if f.grows { reach } else { 0 }, targets) {
            (point, normal) = (p, n);
            met_world = who.is_none();
            match who {
                None => bits |= 9,
                Some((who, at)) => {
                    bits |= 1;
                    let damage = self.things[k].damage;
                    match who {
                        // Its touch (+0x58, flag 0x40000): what it knocks
                        // off.
                        Met::Object(template) => {
                            let touch = self.set.templates.get(template as usize).map_or(0, |tp| tp.raw[0x0a]);
                            if (1..=7).contains(&touch) {
                                let models = &self.set.models;
                                let radius_of = |m: u32| models.get(m).map_or(0, |m| m.radius as i32);
                                if self.flights.touch_parts(touch, at, p, radius_of, self.sprites.as_ref()) {
                                    touched = Some(at);
                                }
                            }
                        }
                        // A character struck (its touch, Damage by the
                        // thing's +0x6d for class 1, 0x800780ac).
                        Met::Npc(n) => {
                            self.struck.push(Strike { npc: n, thing: k, damage });
                            let c = self.npcs[n].character;
                            let r = self.rand.next() & 1;
                            self.speech.extend(hit_line(c, self.level, r));
                            self.things[k].last_hit = Some(n);
                        }
                        // Damage(player, player, +0x8e), posted (0x800780ac).
                        Met::Player => {
                            if damage > 0 && !self.no_damage {
                                self.post_damage(damage);
                            }
                            self.things[k].last_hit = Some(PLAYER_HIT);
                        }
                    }
                }
            }
        }
        if bits & 1 != 0 {
            let (c, r) = crate::world::flights::reflect(&self.flights.angles, v, normal);
            if c < 1025 {
                bits &= !1;
            } else {
                f.pos = [0, 1, 2].map(|i| point[i] - ((u[i] * reach) >> 12));
                let speed = collide::length(v) - 95;
                let below = [f.pos[0], f.pos[1] + f.radius, f.pos[2]];
                if speed < 382 && self.flight_meets(f.pos, below, f.objects, if f.grows { reach } else { 0 }, targets).is_some() {
                    v = [0; 3];
                    bits |= 2;
                } else {
                    let s = (speed * f.bounce) >> 12;
                    v = r.map(|r| (r * s) >> 12);
                }
                self.things[k].bounced = true;
            }
        }
        let t = &mut self.things[k];
        t.pos = Vec3::new(f.pos[0] as f32, f.pos[1] as f32, f.pos[2] as f32);
        // Done at a hit (flag 0x2000000): +0x6b 255, and its think takes
        // it out of its room (0x8004eb10, 0x80049380).
        if bits & 1 != 0 && f.ends_on_hit {
            t.state = ThingState::Rest;
            t.flown = None;
            t.last_hit = None;
            t.bounced = false;
            t.hidden = true;
            self.ended.push(Ended { thing: k, at: f.pos, normal, point, world: met_world });
            return touched;
        }
        if bits & 2 != 0 {
            t.state = ThingState::Rest;
            t.flown = None;
            t.last_hit = None;
            t.bounced = false;
        } else {
            t.state = ThingState::Flying(v);
            t.flown = Some(f);
        }
        touched
    }

    /// A rolling flight's frame (0x80086ef0, flag 0x10000000), after the
    /// same move (0x8004fbb4): the segment from where it was to its
    /// leading edge (reach = radius, at least 10), turned along the floor
    /// it was on (0x80086b58: -v along the plane, as long as the segment,
    /// 0x80075f58), meets as a flight does. Off a surface met head on it
    /// backs off the reach from the point and goes back along the
    /// reflection at |v| bounce >> 12. Then it rolls (0x80086cd0).
    fn roll_exact(&mut self, k: usize, targets: &[(crate::world::boxhit::Solid, Met)]) -> Option<[i32; 3]> {
        let dt = DT;
        let (Some(mut f), ThingState::Flying(mut v)) = (self.things[k].flown, self.things[k].state) else { return None };
        let mut touched = None;
        let old = f.pos;
        let a = [0, f.gravity, 0];
        for i in 0..3 {
            let ad = a[i] * dt;
            f.pos[i] += (((ad * dt) >> 1) + v[i] * dt) >> 6;
            v[i] += ad;
        }
        let reach = f.radius.max(10);
        let u = collide::unit(v);
        let lead = [0, 1, 2].map(|i| f.pos[i] + ((u[i] * reach) >> 12));
        let d = {
            let dd = [0, 1, 2].map(|i| old[i] - lead[i]);
            if dd.iter().any(|c| c.unsigned_abs() >= 0x4000) {
                0x4000
            } else {
                dd.iter().map(|&c| c.wrapping_mul(c) as u32).fold(0u32, u32::wrapping_add).isqrt() as i16 as i32
            }
        };
        let bend = collide::along_plane(v.map(|c| -c), collide::floor_normal(&self.col, old));
        let to = [0, 1, 2].map(|i| old[i] + ((bend[i] * d) >> 12));
        let mut bits = 0;
        let mut normal = [0; 3];
        let mut point = f.pos;
        if let Some((who, p, n)) = self.flight_meets(old, to, f.objects, 0, targets) {
            (point, normal) = (p, n);
            bits |= 1;
            if let Some((who, at)) = who {
                let damage = self.things[k].damage;
                match who {
                    Met::Object(template) => {
                        let touch = self.set.templates.get(template as usize).map_or(0, |tp| tp.raw[0x0a]);
                        if (1..=7).contains(&touch) {
                            let models = &self.set.models;
                            let radius_of = |m: u32| models.get(m).map_or(0, |m| m.radius as i32);
                            if self.flights.touch_parts(touch, at, p, radius_of, self.sprites.as_ref()) {
                                touched = Some(at);
                            }
                        }
                    }
                    // A character (object flag 0x4000) is struck
                    // (0x800780ac), and not kept as the last hit.
                    Met::Npc(n) => {
                        self.struck.push(Strike { npc: n, thing: k, damage });
                        let c = self.npcs[n].character;
                        let r = self.rand.next() & 1;
                        self.speech.extend(hit_line(c, self.level, r));
                    }
                    Met::Player => {}
                }
            }
        }
        if bits & 1 != 0 {
            let (c, r) = crate::world::flights::reflect(&self.flights.angles, v, normal);
            if c >= 1025 {
                f.pos = [0, 1, 2].map(|i| point[i] - ((u[i] * reach) >> 12));
                let s = (collide::length(v) * f.bounce) >> 12;
                v = r.map(|r| (r * s) >> 12);
                self.things[k].bounced = true;
            }
        }
        let rolling = self.roll_on(&mut f.pos, &mut v, reach, dt);
        let t = &mut self.things[k];
        t.pos = Vec3::new(f.pos[0] as f32, f.pos[1] as f32, f.pos[2] as f32);
        if rolling {
            t.state = ThingState::Flying(v);
            t.flown = Some(f);
        } else {
            t.state = ThingState::Rest;
            t.flown = None;
            t.last_hit = None;
            t.bounced = false;
        }
        touched
    }

    /// The roll (0x80086cd0): on the floor under it (its centre `reach`
    /// over it), level, slowed by the friction (D_800c4460, 98, by dt)
    /// and pushed down a slope (0x80086a20). Without a floor, or too slow
    /// on the level, it stops (false).
    pub(crate) fn roll_on(&self, pos: &mut [i32; 3], v: &mut [i32; 3], reach: i32, dt: i32) -> bool {
        let friction = (ROLL_FRICTION * dt) >> 6;
        v[1] = 0;
        let s = collide::length(*v);
        let u = collide::unit(*v);
        let q = collide::query(&self.col, *pos);
        let Some((c, t)) = q.tile else {
            *v = [0; 3];
            return false;
        };
        let n = self
            .col
            .columns
            .get(c)
            .and_then(|c| c.tiles.get(t))
            .and_then(|t| self.col.floor_normals.get(t[1] as usize))
            .map_or([0, -4096, 0], |n| n.map(|v| v as i32));
        pos[1] = q.floor - reach;
        let push = self.slope_push(n[1], dt);
        if push == 0 && friction >= s {
            *v = [0; 3];
            return false;
        }
        *v = [collide::fix(u[0], s - friction), 0, collide::fix(u[2], s - friction)];
        if push != 0 {
            let down = collide::unit([n[0], 0, n[2]]);
            v[0] += collide::fix(down[0], push);
            v[2] += collide::fix(down[2], push);
            if collide::length(*v) <= friction {
                *v = [0; 3];
                return false;
            }
        }
        true
    }

    /// A slope's push (0x80086a20) from the floor normal's y: by its
    /// tilt |asin y + 1024|, none flat (D_800c446c, 0), 1190 (D_800c4468)
    /// at 768 (D_800c4470) or steeper, between ((99 tilt >> 6) + 2) dt >> 6.
    fn slope_push(&self, ny: i32, dt: i32) -> i32 {
        let a = self.trig.asin(ny as i16 as i32);
        let tilt = if -a - 1024 >= 0 { -a - 1024 } else { a + 1024 };
        let k = ((0 - 1190) << 6) / (0 - 768);
        if tilt >= 768 {
            1190
        } else if tilt <= 0 {
            0
        } else {
            ((((k * tilt) >> 6) + (1190 - ((k * 768) >> 6))) * dt) >> 6
        }
    }
}

/// ApplyMatrixSV (0x8009fc38, 0x8009fbe8): `m` times the short vector
/// `v` through the GTE (>> 12, kept to 16 bits).
pub(super) fn apply_sv(m: &[[i32; 3]; 3], v: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|r| (((0..3).map(|k| m[r][k] as i64 * v[k] as i64).sum::<i64>() >> 12) as i32).clamp(-0x8000, 0x7fff))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThingState {
    Rest,
    Carried,
    /// In flight (0x80087c5c): velocity.
    Flying([i32; 3]),
}

/// Hand slots of the grip (0x8005ff88): the object rides slot 15.
pub(super) const HAND: usize = 15;

pub(super) const OTHER_HAND: usize = 12;
/// Where floor dust rises from (0x80044fc8(player, 16)).
const DUST_PART: usize = 16;


/// Thrown objects fall at 13 (flight flags 0x700).
pub(super) const THING_GRAVITY: i32 = 13;

/// A roll's friction a 64th of dt (D_800c4460).
const ROLL_FRICTION: i32 = 98;

/// A thrown thing that struck a character: the NPC, the thing and its
/// +0x6d.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    pub npc: usize,
    pub thing: usize,
    pub damage: i32,
}

impl Game {
    /// 0x80059b34, which each character's update (0x80082230) runs for
    /// itself: the player's aim target (+0x1f4) is the nearest character
    /// within 512 of the heading and 768 of the pitch, by the squared
    /// distance (0x8007681c: none past 32767 on an axis). One held stays
    /// until a nearer one passes or it leaves the bounds itself. In levels
    /// 20-21 only the babies count (templates 2, 6, 7, 11).
    pub fn consider_aim(&mut self, k: usize) {
        let n = &self.npcs[k];
        if n.hidden || matches!(n.mode, Mode::Ride { .. }) {
            return;
        }
        let box_toss = matches!(self.level, Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY);
        let wrap = |a: i32| {
            let a = a as i16 as i32;
            (if a > 2048 { a - 4096 } else if a < -2048 { a + 4096 } else { a }) as i16 as i32
        };
        let p = self.player.pos;
        let d = [n.pos[0] - p[0], n.pos[1] - p[1], n.pos[2] - p[2]];
        let dist = |d: [i32; 3]| {
            if d.iter().any(|c| c.unsigned_abs() >> 15 != 0) {
                u32::MAX
            } else {
                d.iter().map(|&c| c.wrapping_mul(c) as u32).fold(0u32, u32::wrapping_add)
            }
        };
        let yaw = wrap(self.trig.ratan2(d[0], d[2]) - self.player.heading);
        let pitch = wrap(-self.trig.asin(d[1] as i16 as i32));
        let within = (!box_toss || crate::levels::box_toss::BABIES.contains(&n.id)) && yaw.abs() < 513 && pitch.abs() < 769;
        if !within {
            if self.aim_target == Some(k) {
                self.aim_target = None;
            }
            return;
        }
        match self.aim_target {
            Some(t) if t != k => {
                let q = self.npcs[t].pos;
                if dist(d) < dist([q[0] - p[0], q[1] - p[1], q[2] - p[2]]) {
                    self.aim_target = Some(k);
                }
            }
            _ => self.aim_target = Some(k),
        }
    }

    /// Level 9's throw (0x80061c98): toward the aim target, (0, -384,
    /// 1024) (D_800b7fe0) turned by the way to it (0x800760f4: ratan2 of
    /// the unit way, minus the arc sine of its height) when that is within
    /// 512 of the heading, else by the player's own angles; made a unit,
    /// its height then -300, at speed 170 (D_800b7fec).
    pub(super) fn aimed_dir(&self) -> Option<[i32; 3]> {
        let t = self.aim_target?;
        // (The character's height as its +0x1c has it: with its root.)
        let n = &self.npcs[t];
        let (p, q) = (self.player.pos, [n.pos[0], n.pos[1] + n.root.y, n.pos[2]]);
        let way = collide::unit([q[0] - p[0], q[1] - p[1], q[2] - p[2]]);
        let angles = [self.trig.ratan2(way[0], way[2]) as i16, -self.trig.asin(way[1] as i16 as i32) as i16, 0];
        let off = (angles[0] as i32 - self.player.heading) as i16 as i32;
        let off = if off < -2047 { off + 4096 } else if off > 2048 { off - 4096 } else { off };
        let by = if off.abs() < 513 { angles } else { [self.player.heading as i16, 0, 0] };
        let dir = collide::unit(self.trig.turn_by(by, [0, -384, 1024]));
        Some([dir[0], -300, dir[2]])
    }

    /// Throws the carried thing with velocity `v` (a level's own throw),
    /// letting go at the throw animation's release.
    pub fn throw_carried(&mut self, v: [i32; 3]) {
        if let Some(k) = self.carried {
            self.pending_throw = Some(PendingThrow::Given(k, v));
        }
    }

    /// Thing `k` set flying at `v` from where it is, its record as the
    /// throw builds it (0x80061f88, 0x8008864c): its radius, the larger of
    /// its first box's y extents; its template's bounce; objects met
    /// unless it meets the world only; rolling when its template says so.
    pub fn start_flight(&mut self, k: usize, v: [i32; 3]) {
        let t = &self.things[k];
        let radius = self.set.models.get(t.shape).and_then(|m| m.boxes.first()).map_or(10, |b| (b.0[1] as i32).abs().max((b.1[1] as i32).abs()));
        let rolls = !hurts_player(&self.set.templates, t.template);
        let f = Flown {
            pos: [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32],
            radius,
            bounce: bounce_of(&self.set.templates, t.template),
            objects: !t.world_only,
            rolls,
            gravity: THING_GRAVITY,
            ends_on_hit: false,
            grows: false,
        };
        self.things[k].flown = Some(f);
        self.things[k].state = ThingState::Flying(v);
    }

    /// Lets go of a thrown thing (0x80060b44's release): the direction
    /// and speed of the plain or aimed throw as they are now, or the
    /// level's own velocity.
    fn release(&mut self, throw: PendingThrow) {
        // The carry runs before the throw's handler: the thing let go of
        // where the hand as last drawn holds it.
        self.carry_place();
        let (k, v) = match throw {
            PendingThrow::Given(k, v) => (k, v),
            PendingThrow::Down(k) => {
                // The drop flight (0x80062ac4): (0, 0, 4096) turned by the
                // player's angles and made a unit, at speed 10 (0x80061f88,
                // 0x8008864c).
                if self.things[k].template == HIGH_BOX {
                    // Put down (0x80060910): message 8.
                    self.training.show(8);
                }
                let dir = collide::unit(self.trig.turn_by([self.player.heading as i16, 0, 0], [0, 0, 4096]));
                let speed = (10 << 16) / 6000;
                (k, dir.map(|d| (d * speed) >> 12))
            }
            // (0, -384, 1024) (D_800b7fe0) turned by the player's angles
            // (0x80075ee8) and made a unit (0x8007652c), or level 9's aim, at
            // speed 170 (D_800b7fec): dir * (170 << 16) / 6000 >> 12
            // (0x8008864c).
            // The pie (0x80061c98, +0x30 bit 8): (0, 35, 1024) turned by
            // the player and made a unit, at 225 with the target within
            // 13000, else 450.
            PendingThrow::Aimed(k) if self.pie_toss.is_some() && self.things[k].template == PIE => {
                let toss = self.pie_toss.unwrap();
                let dir = collide::unit(self.trig.turn_by([self.player.heading as i16, 0, 0], [0, 35, 1024]));
                let me = self.player.pos;
                let near = self.npcs.iter().find(|n| n.id == toss.target).is_some_and(|n| {
                    let d = if toss.flat {
                        crate::actors::npc::approx_dist(me, n.pos) as i64
                    } else {
                        collide::length([n.pos[0] - me[0], n.pos[1] - me[1], n.pos[2] - me[2]]) as i64
                    };
                    d < 13000
                });
                let speed = (if near { 225 } else { 450 } << 16) / 6000;
                (k, dir.map(|d| (d * speed) >> 12))
            }
            PendingThrow::Aimed(k) => {
                let plain = || collide::unit(self.trig.turn_by([self.player.heading as i16, 0, 0], [0, -384, 1024]));
                let dir = if self.aim { self.aimed_dir() } else { None }.unwrap_or_else(plain);
                let speed = (170 << 16) / 6000;
                (k, dir.map(|d| (d * speed) >> 12))
            }
        };
        // A throw, a lob or a put-down flies as the original's record
        // (0x80087594's path, or 0x80086ef0's roll for template +0x30 of 1).
        if self.carried == Some(k) {
            self.start_flight(k, v);
            if self.pie_toss.is_some()
                && self.things[k].template == PIE
                && let Some(f) = &mut self.things[k].flown
            {
                // Flags 0x4a070400: gravity 1, done at a hit, met as a
                // segment of its radius.
                f.gravity = 1;
                f.ends_on_hit = true;
                f.grows = true;
            }
            self.carried = None;
            self.player.carrying = false;
            // The arms as they were after (0x80060910, 0x80060b44:
            // 0x80057244(player, 17, 0)).
            self.player.animator.set_after(17, false);
            if self.pie_toss.is_some() && self.things[k].template == PIE {
                self.next_pie();
            }
        }
    }

    /// The shade of the COL tile at `p` (its byte +5, which 0x800869cc
    /// reads for a mark; the player's brightness eases toward it).
    fn shade_at(&self, p: [i32; 3]) -> Option<u8> {
        let (c, t) = collide::query(&self.col, p).tile?;
        self.col.columns.get(c)?.tiles.get(t).map(|tile| tile[5])
    }

    /// The pies, in their order (0x800e2770).
    fn pies(&self) -> Vec<usize> {
        (0..self.things.len()).filter(|&k| self.things[k].template == PIE).collect()
    }

    /// A pie toss starts (0x8007e680 / level 3's): the six pies' home is
    /// the pie spot, (0, 0, 512) turned by the player plus his place
    /// (0x80049818), and they wait there; pie 0 is in hand (0x80049464,
    /// +0x1f0). `target` and `flat` pick the throw's speed.
    pub fn start_pie_toss(&mut self, target: u16, flat: bool) {
        let ahead = self.trig.turn_by([self.player.heading as i16, 0, 0], [0, 0, 512]);
        let me = self.player.pos;
        let spot = [me[0] + ahead[0], me[1] + ahead[1], me[2] + ahead[2]];
        self.pie_toss = Some(PieToss { next: 0, spot, target, flat });
        for k in self.pies() {
            let t = &mut self.things[k];
            t.pos = Vec3::new(spot[0] as f32, t.pos.y, spot[2] as f32);
            t.state = ThingState::Rest;
            t.flown = None;
            t.hidden = false;
        }
        if let Some(&k) = self.pies().first() {
            self.hold(k);
        }
    }

    /// The pie toss over: the pie in hand put away with the others
    /// (guess: the end's pies are not read).
    pub fn end_pie_toss(&mut self) {
        self.pie_toss = None;
        if self.carried.is_some_and(|k| self.things[k].template == PIE) {
            self.carried = None;
            self.player.carrying = false;
        }
        for k in self.pies() {
            if !matches!(self.things[k].state, ThingState::Flying(_)) {
                self.things[k].state = ThingState::Rest;
                self.things[k].hidden = true;
            }
        }
    }

    /// The thrown pie's first think (0x8004eb10): the next pie (0x800c4134
    /// + 1, mod 6) back at the pie spot, in its room and in hand.
    fn next_pie(&mut self) {
        let pies = self.pies();
        let Some(toss) = &mut self.pie_toss else { return };
        if pies.is_empty() {
            return;
        }
        toss.next = (toss.next + 1) % pies.len();
        let (k, spot) = (pies[toss.next], toss.spot);
        let t = &mut self.things[k];
        t.pos = Vec3::new(spot[0] as f32, t.pos.y, spot[2] as f32);
        t.state = ThingState::Rest;
        t.flown = None;
        t.hidden = false;
        self.hold(k);
    }

    /// The carry (0x8005ff88 mode 1): the thing carried at hand 15 as last
    /// drawn, plus the grip turned by the player's matrix (ApplyMatrixSV).
    pub(super) fn carry_place(&mut self) {
        let (parts, m) = self.drawn_pose();
        if let Some(k) = self.carried
            && let Some(hand) = self.player.slot_part(HAND).and_then(|i| parts.get(i))
        {
            let g = apply_sv(&m, self.grip);
            self.things[k].pos = Vec3::new((hand.joint[0] + g[0]) as f32, (hand.joint[1] + g[1]) as f32, (hand.joint[2] + g[2]) as f32);
            self.things[k].rot = self.player.rotation(&self.trig);
        }
    }

    /// Forgets the player's pose as last drawn: another body has become
    /// the player (each body's matrices are its own), or one is handed a
    /// thing without its animation.
    pub fn forget_pose(&mut self) {
        self.pose_before = None;
    }

    /// The player's part matrices (+0x158) and matrix (+4) now.
    pub(super) fn pose_now(&self) -> (Vec<crate::actors::pose::PartMatrix>, [[i32; 3]; 3]) {
        (self.player.posed(&self.anims, &self.trig).0, self.trig.rotation([self.player.heading as i16, 0, 0]))
    }

    /// The player's part matrices and matrix as the grip reads them: as
    /// last drawn (the original draws before it updates), else as now.
    pub fn drawn_pose(&self) -> (Vec<crate::actors::pose::PartMatrix>, [[i32; 3]; 3]) {
        self.pose_before.clone().unwrap_or_else(|| self.pose_now())
    }

    /// The grip (0x8005ff88 mode 0): the hands' midpoint (halved toward
    /// zero) less hand 15, into the player's frame (ApplyMatrixSV of the
    /// matrix transposed), its z on by half the object's deepest box
    /// (0x8005fd58), as last drawn.
    fn grip_of(&self, depth: i32) -> [i32; 3] {
        let (parts, m) = self.drawn_pose();
        let at = |slot| self.player.slot_part(slot).and_then(|i| parts.get(i)).map(|p| p.joint);
        let (Some(a), Some(b)) = (at(HAND), at(OTHER_HAND)) else { return [0, -300, (depth >> 1) + 200] };
        let d = [0, 1, 2].map(|k| ((a[k] + b[k]) / 2 - a[k]) as i16 as i32);
        let t = [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]));
        let g = apply_sv(&t, d);
        [g[0], g[1], (g[2] + ((depth << 11) >> 12)) as i16 as i32]
    }

    /// Puts thing `k` in the player's hand (0x8005ff88 mode 0 with no
    /// pick-up animation).
    pub fn give(&mut self, k: usize) {
        if k >= self.things.len() {
            return;
        }
        self.forget_pose();
        self.grip = self.grip_of(0);
        self.things[k].state = ThingState::Carried;
        self.things[k].flown = None;
        self.carried = Some(k);
        self.player.carrying = true;
        self.player.carry_climbs = !self.things[k].keep;
    }

    /// Puts thing `k` in the player's hand as +0x1f0 alone does (the pie
    /// toss, 0x8007e680, 0x8004eb10): the grip kept as it was.
    pub fn hold(&mut self, k: usize) {
        if k >= self.things.len() {
            return;
        }
        self.things[k].state = ThingState::Carried;
        self.things[k].flown = None;
        self.carried = Some(k);
        self.player.carrying = true;
        self.player.carry_climbs = !self.things[k].keep;
        // No grab's animation: the arms play on as they were.
        self.player.animator.mark_asked(crate::actors::player::anim::CARRY_IDLE);
    }

    /// Puts the carried thing down where the player stands (a cine's
    /// drop).
    pub fn drop_carried(&mut self) {
        if let Some(k) = self.carried.take() {
            let p = self.player.pos;
            self.things[k].pos = Vec3::new(p[0] as f32, (p[1] - self.player.feet()) as f32 - 100.0, p[2] as f32);
            self.things[k].state = ThingState::Rest;
            self.things[k].flown = None;
            self.player.carrying = false;
            self.player.animator.set_after(17, false);
        }
    }

    /// The thing carried.
    pub fn carried(&self) -> Option<usize> {
        self.carried
    }

    /// GRAB and ACTION in the stand state (0x8005d11c): Circle picks the
    /// touched object up (state 7) or puts the carried one down (state 8);
    /// Triangle throws it (0x80060b44).
    pub(super) fn carry(&mut self, pads: &Pads) {
        // A throw asked for lets go at its animation's release event (16,
        // 0x80060b44), or when the animation ends without one.
        if let Some(k) = self.pending_grab {
            let anim = &self.player.animator;
            if anim.events & 2048 != 0 || !anim.playing(PICK_UP) {
                self.pending_grab = None;
                if self.things[k].state == ThingState::Rest && !self.things[k].hidden {
                    self.grab(k);
                }
            }
            return;
        }
        if let Some(throw) = self.pending_throw {
            let anim = &self.player.animator;
            let (event, playing) = match throw {
                PendingThrow::Down(_) => (2048, PUT_DOWN),
                _ => (16, THROW),
            };
            if anim.events & event != 0 {
                self.pending_throw = None;
                self.release(throw);
            } else if !anim.playing(playing) {
                self.pending_throw = None;
                // The put-down lets go only at its event (0x80060910: +0xe0
                // bit 2048); run out without it, the state ends with the
                // thing still in hand (the oracle's Chuckie at level 6's
                // shelf). The throw's own end still lets go.
                if !matches!(throw, PendingThrow::Down(_)) {
                    self.release(throw);
                }
            }
            return;
        }
        let p = &self.player;
        let standing = matches!(p.state, State::Stand | State::Walk | State::Run) && p.action.is_none();
        if !standing {
            return;
        }
        if let Some(k) = self.carried {
            // The carry's animation done this frame (0x80060b44 at
            // 0x80060ef8, +0xe0 bit 0x8000000): message 6, 27 for the pie.
            if self.player.animator.events & 0x800_0000 != 0 {
                self.training.show(if self.things[k].template == PIE { 27 } else { 6 });
            }
            let throw = pads.pressed(0, input::TRIANGLE) && !self.things[k].keep;
            // GRAB never puts down in levels 5, 20 and 21 (0x8005d11c).
            let put_down = pads.pressed(0, input::CIRCLE)
                && !matches!(self.level, Level::OUTER_SPACE | Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY);
            if throw {
                // The stand handler asks for 18 at 0x2000 (0x8005d11c).
                self.player.act_at(THROW, THROW_ANIM_SPEED, &self.anims);
                self.player.jump_lock = 300;
                self.pending_throw = Some(PendingThrow::Aimed(k));
                return;
            }
            // Never the pie; only with room for it (0x8005d11c).
            if put_down && self.things[k].template != PIE && self.room_to_put(k) {
                self.player.act(PUT_DOWN, &self.anims);
                self.pending_throw = Some(PendingThrow::Down(k));
            }
            return;
        }
        if !pads.pressed(0, input::CIRCLE) {
            return;
        }
        let Some(k) = self.touched() else { return };
        // State 7: the pick-up's animation, the object taken at its grab
        // event (2048, 0x800601e4).
        self.player.act(PICK_UP, &self.anims);
        self.pending_grab = Some(k);
    }

    /// Whether carried thing `k` may be put down (0x80063888): the middle
    /// of its first model box, where it is held, outside every model box
    /// of the objects about (the nearby list) grown by the larger of its
    /// box's half-width and half-depth, each in its object's frame; a
    /// character's body likewise.
    pub(super) fn room_to_put(&self, k: usize) -> bool {
        let t = &self.things[k];
        let Some(&(lo, hi)) = self.set.models.get(t.shape).and_then(|m| m.boxes.first()) else { return true };
        let mid = Vec3::new((lo[0] + hi[0]) as f32, (lo[1] + hi[1]) as f32, (lo[2] + hi[2]) as f32) / 2.0;
        let at = t.pos + t.rot * mid;
        let grow = ((hi[0] - lo[0]) / 2).max((hi[2] - lo[2]) / 2) as f32;
        !self.bodies.iter().any(|b| {
            if b.character {
                (at.x - b.pos.x).abs() <= 200.0 + grow
                    && (at.z - b.pos.z).abs() <= 200.0 + grow
                    && at.y >= b.min_y as f32 - grow
                    && at.y <= b.max_y as f32 + grow
            } else {
                let l = b.rot.transpose() * (at - b.pos);
                (0..3).all(|i| l[i] >= b.lo[i] as f32 - grow && l[i] <= b.hi[i] as f32 + grow)
            }
        })
    }

    /// The thing GRAB takes (0x800631d8, oracle-checked in level 6): of
    /// those in the player's nearby list (0x80062ed4 / 0x8006abbc: his
    /// cylinder, 3 x his body box's larger half-extent across and 3 x its
    /// height's, meets the thing's, its farthest model box corner
    /// round its place; a box of level 6, 577, is taken from 1324 straight
    /// ahead and not from 1330), within 511 below to 4095 above him, his
    /// box's depth + 1800 off, and strictly within a quarter turn of his
    /// heading (D_800c430a; none from behind), the nearest; not the one he
    /// stands on.
    pub(super) fn touched(&self) -> Option<usize> {
        let p = &self.player;
        let (lo, hi) = p.body_box();
        let half = |i: usize| lo[i].abs().max(hi[i].abs());
        let across = 3 * half(0).max(half(2));
        let tall = 3 * half(1);
        let reach = half(2) + 1800;
        let me = p.pos;
        let off = |a: i32| (a + 2048).rem_euclid(4096) - 2048;
        self.things
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == ThingState::Rest && !t.hidden)
            .filter_map(|(k, t)| {
                let radius = self.set.models.get(t.shape).map_or(0.0, |m| {
                    m.boxes
                        .iter()
                        .flat_map(|(lo, hi)| (0..8).map(move |c| {
                            Vec3::new(
                                if c & 1 == 0 { lo[0] } else { hi[0] } as f32,
                                if c & 2 == 0 { lo[1] } else { hi[1] } as f32,
                                if c & 4 == 0 { lo[2] } else { hi[2] } as f32,
                            )
                            .length()
                        }))
                        .fold(0.0f32, f32::max)
                }) as i32;
                let (dx, dy, dz) = (t.pos.x as i32 - me[0], t.pos.y as i32 - me[1], t.pos.z as i32 - me[2]);
                let d = ((dx as f32).hypot(dz as f32)) as i32;
                let bearing = ((dx as f32).atan2(dz as f32) * 4096.0 / std::f32::consts::TAU) as i32;
                let stood = p.support.is_some_and(|b| self.bodies.get(b).is_some_and(|b| b.template == t.template && b.covers(me, 0)));
                (d <= across + radius
                    && dy.abs() <= tall + radius
                    && (-4095..512).contains(&dy)
                    && d <= reach
                    && off(bearing - p.heading).abs() < 1024
                    && !stood)
                    .then_some((k, d))
            })
            .min_by_key(|&(_, d)| d)
            .map(|(k, _)| k)
    }

    /// The walk's effects (0x8005e1f4): on a water tile (COL byte 4 bit
    /// 0) and not on an object, a splash each frame (0x80050770). At a
    /// step (+0xe0 bit 256): character 34 sounds 108; in water Spike sounds
    /// 57 on every fifth step in level 4 (0x800c42e4), 176 in level 16 and
    /// 27 elsewhere, the others 27 but in level 1. Out of water the step
    /// asks 0x8007013c for the floor's footstep, whose table (0x800c4ec4)
    /// is empty in play (oracle): nothing. Reptar (template 8) at a step
    /// puffs between his feet (0x80050c48: x and z between parts 3 and 7).
    /// Placement `k`'s object breaks into its pieces (0x8004cdfc), flying
    /// from its own place at `speed`.
    pub fn shatter(&mut self, k: usize, speed: i32) {
        let q = self.set.placements[k];
        let Some(m) = self.set.templates.get(q.template as usize).and_then(|t| self.set.models.get(t.shape as u32))
        else {
            return;
        };
        let rot = models::quat_matrix(q.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(q.pos[0] as f32, q.pos[1] as f32, q.pos[2] as f32);
        self.flights.shatter(m, rot, pos, q.pos, speed, &mut self.rand, self.sprites.as_ref());
    }

    /// The object in focus (player +0x1e8, 0x8006af4c), as its placement:
    /// the body's cylinder (its box's widest across and tallest) and the
    /// segment down from the part the hierarchy names (header +0x24).
    pub fn focus(&self) -> Option<usize> {
        let p = &self.player;
        let (lo, hi) = p.body_box();
        let radius = lo[0].abs().max(hi[0].abs()).max(lo[2].abs().max(hi[2].abs()));
        let half = lo[1].abs().max(hi[1].abs());
        let part = p.hierarchy.header.get(0x24).copied().unwrap_or(0) as usize;
        let at = p.part_at(part, &self.anims, &self.trig);
        boxes::focus_at(&self.set, &self.trig, p.pos, radius, half, at)
    }

    /// The landing's sounds (state 10's entry, 0x8005b624): 102 when the
    /// object in focus is template 505 (level 11), 43 on template 494
    /// (level 3), 57 on water in level 4.
    pub(super) fn land_sounds(&mut self, on: u16) {
        let at = self.player.pos;
        let feet = at[1] - self.player.feet();
        if self.focus().map(|k| self.set.placements[k].template) == Some(505) {
            self.play(102);
        } else if on == 494 {
            self.play_at(43, at);
        } else if self.level == Level::CYNTHIA {
            let q = collide::query(&self.col, [at[0], feet - 1, at[2]]);
            if collide::floor_flags(&self.col, &q) & 1 != 0 {
                self.play_at(57, at);
            }
        }
    }

    /// The walk's touches (0x8005e1f4, the player's own walk): the
    /// object in focus is touched each frame (touches 12, 13 and 15 are
    /// the floor's); on a step, a rug of 1160-1163 in focus sounds (73,
    /// 75, 76, 74 at it); on floor flag 4, not on an object, a puff at a
    /// foot (slot 7 or 3 by the C rand's low bit).
    fn floor_touches(&mut self, q: &collide::Query) {
        let p = &self.player;
        let at = p.pos;
        let step = p.animator.events & 256 != 0;
        if let Some(k) = self.focus() {
            let place = self.set.placements[k];
            let template = self.set.templates.get(place.template as usize);
            let damage = template.map_or(0, |t| t.raw[0x32] as i16);
            match template.map_or(0, |t| t.raw[0x0a]) {
                12 => self.flights.touch_12(at, self.sprites.as_ref()),
                13 => self.flights.touch_13(place.pos, damage, self.sprites.as_ref()),
                15 => self.piano(&place),
                _ => {}
            }
            let rug = match place.template {
                1160 => Some(73),
                1161 => Some(75),
                1162 => Some(76),
                1163 => Some(74),
                _ => None,
            };
            if let Some(s) = rug.filter(|_| step) {
                self.play_at(s, place.pos);
            }
        }
        let on_object = self.player.support.is_some() || self.mover_under_player().is_some();
        if collide::floor_flags(&self.col, q) & 4 != 0 && !on_object {
            self.crand = self.crand.wrapping_mul(0x41c6_4e6d).wrapping_add(0x3039);
            let slot = if (self.crand >> 16) & 1 != 0 { 7 } else { 3 };
            let parts = self.player.parts(&self.anims, &self.trig);
            if let Some(&(_, foot)) = self.player.slot_part(slot).and_then(|i| parts.get(i)) {
                let at = [foot.x as i32, foot.y as i32, foot.z as i32];
                self.flights.floor_puff(at, self.sprites.as_ref());
            }
        }
    }

    /// A piano key walked on (0x800522ec): its note (0x8006f3a4: program
    /// 4, tone 3, note key - 999 + 78, unless it was the last), then its
    /// sprites; keys 1046 and 1047 drop one from one of objects 960-966.
    fn piano(&mut self, key: &Placement) {
        let n = (key.template as i32 - 999) as u8;
        if self.last_note != Some(n) {
            self.last_note = Some(n);
            self.key_at([4, 3, n.wrapping_add(78)], key.pos);
        }
        let drop = if matches!(key.template, 1046 | 1047) {
            let id = 960 + (self.flights.floor_draw(2) % 7) as u16;
            self.set.placements.iter().find(|p| p.kind == 1 && p.id == id).map(|p| p.pos)
        } else {
            None
        };
        let at = self.player.pos;
        self.flights.piano(at, drop, self.sprites.as_ref());
    }

    pub(super) fn walk_effects(&mut self) {
        let p = &self.player;
        if !matches!(p.state, State::Stand | State::Walk | State::Run) {
            return;
        }
        let feet = p.pos[1] - p.feet();
        let q = collide::query(&self.col, [p.pos[0], feet - 1, p.pos[2]]);
        // Dust (0x80050b5c): standing (0x8005d11c) or moving (0x8005e1f4)
        // on a floor of flag 16, half the frames by each state's own draw,
        // from slot 16's part 512 up; never in level 4.
        let stand = p.state == State::Stand;
        let seed = if stand { &mut self.dust_seeds[0] } else { &mut self.dust_seeds[1] };
        let roll = seed.next() & 1 != 0;
        if roll && collide::floor_flags(&self.col, &q) & 16 != 0 && self.level != Level::CYNTHIA {
            let parts = self.player.parts(&self.anims, &self.trig);
            if let Some(&(_, at)) = self.player.slot_part(DUST_PART).and_then(|i| parts.get(i)) {
                let radius = self.set.models.get(crate::world::flights::DUST).map_or(0, |m| m.radius as i32);
                let at = [at.x as i32, at.y as i32 - 512, at.z as i32];
                self.flights.dust(at, radius, self.sprites.as_ref());
            }
        }
        if stand {
            return;
        }
        self.floor_touches(&q);
        let p = &self.player;
        let step = p.animator.events & 256 != 0;
        let on_object = self.mover_under_player().is_some() || (q.kind == 0 && feet < q.floor - 48);
        let water = collide::floor_flags(&self.col, &q) & 1 != 0 && !on_object;
        let at = p.pos;
        if water {
            self.flights.splash(at, self.sprites.as_ref());
        }
        if step {
            let sound = match (self.character, water) {
                (34, _) => Some(108),
                (6, true) if self.level == Level::CYNTHIA => {
                    self.water_steps -= 1;
                    (self.water_steps < 0).then(|| {
                        self.water_steps = 4;
                        57
                    })
                }
                (6, true) if self.level == Level::TRAINING => Some(176),
                (_, true) if self.character == 6 || self.level != Level::HUB => Some(27),
                _ => None,
            };
            if let Some(s) = sound {
                self.play_at(s, at);
            }
        }
        if step && self.level == Level::REPTAR_2010 {
            let parts = self.player.parts(&self.anims, &self.trig);
            if let (Some(a), Some(b)) = (parts.get(3), parts.get(7)) {
                let at = [((a.1.x + b.1.x) / 2.0) as i32, b.1.y as i32, ((a.1.z + b.1.z) / 2.0) as i32];
                self.flights.stomp(at, self.sprites.as_ref());
            }
        }
    }

    /// The grab (0x800601e4 at +0xe0 bit 2048): the object in hand, held
    /// where the hands are now.
    fn grab(&mut self, k: usize) {
        let depth =
            self.set.models.get(self.things[k].shape).map_or(0, |m| m.boxes.iter().map(|b| b.1[2] as i32 - b.0[2] as i32).fold(0, i32::max));
        self.grip = self.grip_of(depth);
        self.grabbed_on = Some(self.frames);
        self.things[k].state = ThingState::Carried;
        self.things[k].flown = None;
        self.carried = Some(k);
        self.player.carrying = true;
        self.player.carry_climbs = !self.things[k].keep;
        // The arms go on to the carry when the pick-up is done (0x800601e4:
        // 0x80057244(player, 20, 0)), but for template 933, level 20 and
        // 21's 886 and Spike.
        let template = self.things[k].template;
        if template != 933 && !(matches!(self.level.0, 20 | 21) && template == 886) && self.character != 6 {
            self.player.animator.set_after(20, false);
        }
        // The grab (0x800601e4): message 5.
        self.training.show(5);
    }

    /// Flights (0x80087c5c): each thrown, lobbed or put down thing flown as
    /// the original's record flies it (`fly_exact`, `roll_exact`).
    pub(super) fn fly(&mut self, solids: &[crate::world::boxhit::Exact], npc_of: &[Option<usize>]) {
        self.struck.clear();
        self.ended.clear();
        self.marks.clear();
        let mut touched = Vec::new();
        if self.things.iter().any(|t| t.flown.is_some()) {
            let targets = self.flight_targets(solids, npc_of);
            for k in 0..self.things.len() {
                if self.things[k].flown.is_some() && matches!(self.things[k].state, ThingState::Flying(_)) {
                    touched.extend(self.fly_exact(k, &targets));
                }
            }
        }
        // A flight done at a hit: its object's callback (+0x5c, 0x8005282c
        // for a template whose +0x0b is 1: five splats) and the record's
        // mark (+0x80, 0x8008652c: the pies' 284 sounds 46 at the player).
        for e in self.ended.clone() {
            let template = self.things[e.thing].template;
            if self.set.templates.get(template as usize).is_some_and(|tp| tp.raw[0x0b] == 1) {
                self.flights.splat(e.normal, e.at, self.sprites.as_ref());
            }
            if template == PIE {
                let me = self.player.pos;
                self.play_at(46, me);
                let n = e.normal;
                let angles = [self.trig.ratan2(-n[0], -n[2]) as i16, (-self.trig.asin(-n[1])) as i16, 0];
                let grey = if e.world { self.shade_at(e.point).unwrap_or(128) } else { 128 };
                self.marks.push(Mark { sprite: PIE_MARK, at: e.point, angles, size: PIE_MARK_SIZE, grey });
            }
        }
        // Touch 1's sound at the object, once its count runs out
        // (0x80050140: 0x800c4ba0 less dt a touch; at 0, 50 and 100 again).
        for at in touched {
            self.shelf_sound -= DT;
            if self.shelf_sound <= 0 {
                self.play_at(50, at);
                self.shelf_sound = 100;
            }
        }
    }
}
