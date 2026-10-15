//! Things that can be picked up, carried, thrown and put down, their
//! flights, and the breakables.

use super::*;
use crate::actors::npc::Mode;
use crate::actors::player::anim::{PICK_UP, PUT_DOWN, THROW};
use crate::actors::player::{Box3, DT, State};
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
    pub shape: u32,
    pub pos: Vec3,
    pub rot: Mat3,
    pub state: ThingState,
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

/// How near an object must be to be touched (the player's radius and some).
pub(super) const REACH: f32 = 520.0;

/// A throw (0x80060b44): direction (0, -384, 1024) turned by the heading,
/// speed 170; the velocity is dir * (speed * 65536 / 6000) >> 12
/// (0x8008864c); a put-down is speed 10 along (0, 0, 4096).
pub(super) const THROW_SPEED: f32 = 170.0 * 65536.0 / 6000.0;

pub(super) const DROP_SPEED: f32 = 10.0 * 65536.0 / 6000.0;

/// How near a flight passes a character to strike it (the radii, guess).
pub(super) const STRIKE: i32 = 450;

/// Thrown objects fall at 13 (flight flags 0x700).
pub(super) const THING_GRAVITY: i32 = 13;

/// A thrown thing that struck a character: the NPC, the thing and its
/// +0x6d.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    pub npc: usize,
    pub thing: usize,
    pub damage: i32,
}

impl Game {
    /// The throw's direction at the aim target (0x80059b34, 0x80061c98):
    /// the nearest character within 512 of the player's heading.
    pub(super) fn aim_at(&self) -> Option<Vec3> {
        let p = self.player.pos;
        let h = self.player.heading;
        let target = self
            .npcs
            .iter()
            .filter(|n| !n.hidden && !matches!(n.mode, Mode::Ride { .. }))
            .filter(|n| {
                let (dx, dz) = ((n.pos[0] - p[0]) as f64, (n.pos[2] - p[2]) as f64);
                let yaw = ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32;
                ((yaw - h + 2048).rem_euclid(4096) - 2048).abs() < 513
            })
            .min_by_key(|n| {
                (n.pos[0] - p[0]) as i64 * (n.pos[0] - p[0]) as i64
                    + (n.pos[2] - p[2]) as i64 * (n.pos[2] - p[2]) as i64
            })?;
        let d = Vec3::new((target.pos[0] - p[0]) as f32, 0.0, (target.pos[2] - p[2]) as f32).normalize_or(Vec3::Z);
        Some(Vec3::new(d.x, -300.0 / 4096.0, d.z))
    }

    /// Throws the carried thing with velocity `v` (a level's own throw).
    pub fn throw_carried(&mut self, v: [i32; 3]) {
        if let Some(k) = self.carried.take() {
            self.things[k].state = ThingState::Flying(v);
            self.player.carrying = false;
        }
    }

    /// Puts thing `k` in the player's hand (0x8005ff88 mode 0 with no
    /// pick-up animation).
    pub fn give(&mut self, k: usize) {
        if k >= self.things.len() {
            return;
        }
        let parts = self.player.parts(&self.anims, &self.trig);
        let rot = self.player.rotation(&self.trig);
        let at = |slot| self.player.slot_part(slot).and_then(|i| parts.get(i)).map(|p| p.1);
        self.grip = match (at(HAND), at(OTHER_HAND)) {
            (Some(a), Some(b)) => rot.transpose() * ((a + b) / 2.0 - a),
            _ => Vec3::new(0.0, -300.0, 200.0),
        };
        self.things[k].state = ThingState::Carried;
        self.carried = Some(k);
        self.player.carrying = true;
    }

    /// Puts the carried thing down where the player stands (a cine's
    /// drop).
    pub fn drop_carried(&mut self) {
        if let Some(k) = self.carried.take() {
            let p = self.player.pos;
            self.things[k].pos = Vec3::new(p[0] as f32, (p[1] - self.player.feet()) as f32 - 100.0, p[2] as f32);
            self.things[k].state = ThingState::Rest;
            self.player.carrying = false;
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
        let p = &self.player;
        let standing = matches!(p.state, State::Stand | State::Walk | State::Run) && p.action.is_none();
        if !standing {
            return;
        }
        let me = Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32);
        let rot = p.rotation(&self.trig);
        if let Some(k) = self.carried {
            // The carry's animation done (0x80060b44): message 6, 27 for
            // the pie.
            self.training.show(if self.things[k].template == PIE { 27 } else { 6 });
            let throw = pads.pressed(0, input::TRIANGLE) && !self.things[k].keep;
            // GRAB never puts down in levels 5, 20 and 21 (0x8005d11c).
            let put_down = pads.pressed(0, input::CIRCLE)
                && !matches!(self.level, Level::OUTER_SPACE | Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY);
            if throw || put_down {
                // Level 9 aims (0x80061c98): at the nearest visible character
                // within 512 of the facing, the direction's y -300.
                let aimed = if throw && self.aim { self.aim_at() } else { None };
                let (dir, speed, anim) = if let Some(d) = aimed {
                    (d, THROW_SPEED, THROW)
                } else if throw {
                    (rot * Vec3::new(0.0, -384.0, 1024.0), THROW_SPEED, THROW)
                } else {
                    (rot * Vec3::new(0.0, 0.0, 4096.0), DROP_SPEED, PUT_DOWN)
                };
                let v = if aimed.is_some() { dir * speed } else { dir.normalize() * speed };
                self.things[k].state = ThingState::Flying([v.x as i32, v.y as i32, v.z as i32]);
                self.carried = None;
                self.player.carrying = false;
                self.player.act(anim, &self.anims);
                if throw {
                    self.player.jump_lock = 300;
                } else if self.things[k].template == HIGH_BOX {
                    // Put down (0x80060910): message 8.
                    self.training.show(8);
                }
            }
            return;
        }
        if !pads.pressed(0, input::CIRCLE) {
            return;
        }
        let near = self
            .things
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == ThingState::Rest && !t.hidden)
            .map(|(k, t)| (k, Vec3::new(t.pos.x - me.x, 0.0, t.pos.z - me.z).length(), (t.pos.y - me.y).abs()))
            .filter(|&(_, d, dy)| d < REACH && dy < 1200.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((k, _, _)) = near else { return };
        // The grip (0x8005ff88 mode 0): the hands' midpoint less hand 15,
        // in the player's frame, pushed forward by half the object's depth.
        let parts = self.player.parts(&self.anims, &self.trig);
        let at = |slot| self.player.slot_part(slot).and_then(|i| parts.get(i)).map(|p| p.1);
        let depth = self
            .set
            .models
            .get(self.things[k].shape)
            .map_or(0.0, |m| m.boxes.iter().map(|b| (b.1[2] - b.0[2]) as f32).fold(0.0, f32::max));
        self.grip = match (at(HAND), at(OTHER_HAND)) {
            (Some(a), Some(b)) => rot.transpose() * ((a + b) / 2.0 - a) + Vec3::new(0.0, 0.0, depth / 2.0),
            _ => Vec3::new(0.0, -300.0, depth / 2.0 + 200.0),
        };
        self.things[k].state = ThingState::Carried;
        self.carried = Some(k);
        self.player.carrying = true;
        self.player.act(PICK_UP, &self.anims);
        // The grab (0x800601e4): message 5.
        self.training.show(5);
    }

    /// Flights (0x80087c5c): gravity 13, pos += (v dt + a dt^2 / 2) >> 6,
    /// until the object's bottom reaches the floor.
    pub(super) fn fly(&mut self) {
        let dt = DT;
        self.struck.clear();
        // What a flight may land on besides the floor: the boxes of things
        // at rest (stacking, 0x8006c7a0).
        let resting: Vec<(usize, Box3)> = self
            .things
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == ThingState::Rest && !t.hidden)
            .filter_map(|(k, t)| self.set.models.get(t.shape).map(|m| (k, m, t)))
            .flat_map(|(k, m, t)| world_boxes(m, t.rot, t.pos).into_iter().map(move |b| (k, b)))
            .collect();
        for (k, t) in self.things.iter_mut().enumerate() {
            let ThingState::Flying(mut v) = t.state else { continue };
            // A flight that reaches a character (0x80086ef0): its touch
            // (0x800511a8) with the thing's +0x6d; the thing drops.
            if v[0] != 0 || v[2] != 0 {
                let hit = self.npcs.iter().position(|n| {
                    let (dx, dz) = (n.pos[0] as f32 - t.pos.x, n.pos[2] as f32 - t.pos.z);
                    !n.hidden
                        && !matches!(n.mode, Mode::Ride { .. })
                        && dx * dx + dz * dz < (STRIKE * STRIKE) as f32
                        && (t.pos.y - (n.pos[1] - n.feet() / 2) as f32).abs() < 900.0
                });
                if let Some(n) = hit {
                    self.struck.push(Strike { npc: n, thing: k, damage: t.damage });
                    let c = self.npcs[n].character;
                    let r = self.rand.next() & 1;
                    self.speech.extend(hit_line(c, self.level, r));
                    v[0] = 0;
                    v[2] = 0;
                }
            }
            let bottom =
                self.set.models.get(t.shape).and_then(|m| m.boxes.iter().map(|b| b.1[1]).max()).unwrap_or(0) as f32;
            let before = t.pos;
            for k in [0, 2] {
                t.pos[k] += (v[k] * dt >> 6) as f32;
            }
            // The wall test (0x80060810): a step into a wall or a higher
            // column is taken back, and the thing drops where it was.
            let side = collide::query(&self.col, [t.pos.x as i32, (t.pos.y + bottom - 1.0) as i32, t.pos.z as i32]);
            if side.kind != 0 || side.tile.is_none() {
                t.pos.x = before.x;
                t.pos.z = before.z;
                v[0] = 0;
                v[2] = 0;
            }
            t.pos.y += ((v[1] * dt + THING_GRAVITY * dt * dt / 2) >> 6) as f32;
            v[1] += THING_GRAVITY * dt;
            let q = collide::query(&self.col, [t.pos.x as i32, (t.pos.y - 1.0) as i32, t.pos.z as i32]);
            let mut floor = if q.tile.is_some() { q.floor as f32 } else { f32::MAX };
            // Another thing's top under it, crossed this step.
            let at = [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32];
            for (_, b) in resting.iter().filter(|(j, _)| *j != k) {
                let top = b.min_y as f32;
                if b.covers(at, 0) && top >= before.y + bottom - 1.0 && top < floor {
                    floor = top;
                }
            }
            if q.kind == 2 || t.pos.y + bottom >= floor {
                if floor < f32::MAX {
                    t.pos.y = floor - bottom;
                }
                t.state = ThingState::Rest;
            } else {
                t.state = ThingState::Flying(v);
            }
        }
    }
}
