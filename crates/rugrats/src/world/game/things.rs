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
    /// What its flight last hit (record +0x98): an NPC, or the player as
    /// `PLAYER_HIT`; and whether it has bounced off anything.
    pub last_hit: Option<usize>,
    pub bounced: bool,
}

/// A throw asked for, waiting for its release: the carried thing, aimed
/// at release as a plain or level 9 throw, or with a level's velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PendingThrow {
    Aimed(usize),
    Given(usize, [i32; 3]),
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

/// The bounce off a surface of normal `n` (0x8007666c, 0x80087594): back
/// out along the reflection at (|v| - 95) * bounce >> 12; None when
/// moving away from it, without a bounce, or too slow.
fn rebound(v: [i32; 3], n: [i32; 3], bounce: i32) -> Option<[i32; 3]> {
    let u = collide::unit(v);
    let c = collide::fix(n[0], u[0]) + collide::fix(n[1], u[1]) + collide::fix(n[2], u[2]);
    let speed = collide::length(v) - 95;
    if bounce == 0 || c >= 0 || speed <= 0 {
        return None;
    }
    let r = [0, 1, 2].map(|k| u[k] - (c.wrapping_mul(n[k]) >> 11));
    let s = (speed * bounce) >> 12;
    Some(r.map(|r| collide::fix(r, s)))
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

    /// Throws the carried thing with velocity `v` (a level's own throw),
    /// letting go at the throw animation's release.
    pub fn throw_carried(&mut self, v: [i32; 3]) {
        if let Some(k) = self.carried {
            self.pending_throw = Some(PendingThrow::Given(k, v));
        }
    }

    /// Lets go of a thrown thing (0x80060b44's release): the direction
    /// and speed of the plain or aimed throw as they are now, or the
    /// level's own velocity.
    fn release(&mut self, throw: PendingThrow) {
        let (k, v) = match throw {
            PendingThrow::Given(k, v) => (k, v),
            PendingThrow::Aimed(k) => {
                let rot = self.player.rotation(&self.trig);
                let v = match if self.aim { self.aim_at() } else { None } {
                    Some(d) => d * THROW_SPEED,
                    None => (rot * Vec3::new(0.0, -384.0, 1024.0)).normalize() * THROW_SPEED,
                };
                (k, [v.x as i32, v.y as i32, v.z as i32])
            }
        };
        if self.carried == Some(k) {
            self.things[k].state = ThingState::Flying(v);
            self.carried = None;
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
            if anim.events & 16 != 0 || !anim.playing(THROW) {
                self.pending_throw = None;
                self.release(throw);
            }
            return;
        }
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
            if throw {
                // The stand handler asks for 18 at 0x2000 (0x8005d11c).
                self.player.act_at(THROW, THROW_ANIM_SPEED, &self.anims);
                self.player.jump_lock = 300;
                self.pending_throw = Some(PendingThrow::Aimed(k));
                return;
            }
            if put_down {
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
        // State 7: the pick-up's animation, the object taken at its grab
        // event (2048, 0x800601e4).
        self.player.act(PICK_UP, &self.anims);
        self.pending_grab = Some(k);
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

    /// The landing's sounds (state 10's entry, 0x8005b624): 102 when the
    /// object in focus is template 505 (level 11), 43 on template 494
    /// (level 3), 57 on water in level 4.
    pub(super) fn land_sounds(&mut self, on: u16) {
        let p = &self.player;
        let (at, feet) = (p.pos, p.pos[1] - p.feet());
        if boxes::focus(&self.set, at, at[1], feet) == Some(505) {
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
        let rot = self.player.rotation(&self.trig);
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
            // A flight that reaches a character (0x80086ef0, 0x80087594):
            // its touch (0x800511a8) with the thing's +0x6d, then the bounce
            // off it by the template's bounce (none: the thing drops).
            let bounce = bounce_of(&self.set.templates, t.template);
            if v[0] != 0 || v[2] != 0 {
                let last = t.last_hit;
                let hit = self.npcs.iter().enumerate().position(|(i, n)| {
                    let (dx, dz) = (n.pos[0] as f32 - t.pos.x, n.pos[2] as f32 - t.pos.z);
                    last != Some(i)
                        && !n.hidden
                        && !matches!(n.mode, Mode::Ride { .. })
                        && dx * dx + dz * dz < (STRIKE * STRIKE) as f32
                        && (t.pos.y - (n.pos[1] - n.feet() / 2) as f32).abs() < 900.0
                });
                if let Some(n) = hit {
                    self.struck.push(Strike { npc: n, thing: k, damage: t.damage });
                    let c = self.npcs[n].character;
                    let r = self.rand.next() & 1;
                    self.speech.extend(hit_line(c, self.level, r));
                    t.last_hit = Some(n);
                    // The face of its bounds cube (+0x78..+0x84, an axis-
                    // aligned cube about it) the thing came at.
                    let p = self.npcs[n].pos;
                    let (dx, dz) = (t.pos.x as i32 - p[0], t.pos.z as i32 - p[2]);
                    let out = if dx.abs() > dz.abs() { [dx.signum() * 4096, 0, 0] } else { [0, 0, dz.signum() * 4096] };
                    match rebound(v, out, bounce) {
                        Some(b) => {
                            v = b;
                            t.bounced = true;
                        }
                        None => {
                            v[0] = 0;
                            v[2] = 0;
                        }
                    }
                }
                // Coming back into Tommy after a bounce: 0x80087594 hurts
                // any class 1 object (Damage(player, player, +0x8e)).
                let me = self.player.pos;
                let (dx, dz) = (me[0] as f32 - t.pos.x, me[2] as f32 - t.pos.z);
                if t.bounced
                    && t.last_hit != Some(PLAYER_HIT)
                    && hurts_player(&self.set.templates, t.template)
                    && dx * dx + dz * dz < (STRIKE * STRIKE) as f32
                    && (t.pos.y - (me[1] - self.player.feet() / 2) as f32).abs() < 900.0
                {
                    t.last_hit = Some(PLAYER_HIT);
                    self.player.hurt(t.damage);
                    v[0] = 0;
                    v[2] = 0;
                }
            }
            let bottom =
                self.set.models.get(t.shape).and_then(|m| m.boxes.iter().map(|b| b.1[1]).max()).unwrap_or(0) as f32;
            let before = t.pos;
            for k in [0, 2] {
                t.pos[k] += ((v[k] * dt) >> 6) as f32;
            }
            // The wall test (0x80060810): a step into a wall or a higher
            // column is taken back, and the thing drops where it was. A
            // step onto level ground is not one, even with the thing's
            // bottom a little under it (a lob from the hand).
            let y = (t.pos.y + bottom - 1.0) as i32;
            let side = collide::query(&self.col, [t.pos.x as i32, y, t.pos.z as i32]);
            let here = collide::query(&self.col, [before.x as i32, y, before.z as i32]);
            let higher = side.kind != 0 && (here.kind == 0 || side.floor < here.floor);
            if side.kind == 2 || side.tile.is_none() || higher {
                // The bounce (0x80087594): off the wall's normal from the
                // segment test.
                let mut hit = collide::Segment::default();
                let u = collide::unit(v);
                let from = [before.x as i32, y, before.z as i32];
                let to = [t.pos.x as i32 + collide::fix(u[0], 10), y, t.pos.z as i32 + collide::fix(u[2], 10)];
                let n = if collide::segment(&self.col, from, to, &mut hit) == 1 {
                    hit.normal
                } else if v[0].abs() >= v[2].abs() {
                    [-v[0].signum() * 4096, 0, 0]
                } else {
                    [0, 0, -v[2].signum() * 4096]
                };
                // Slower than 95 + 382 on a floor it stops (0x80087414).
                let speed = collide::length(v) - 95;
                let on_floor = here.kind == 0 && (here.floor as f32 - (before.y + bottom)).abs() < 100.0;
                match rebound(v, n, bounce).filter(|_| !(on_floor && (0..382).contains(&speed))) {
                    Some(b) => {
                        v = b;
                        t.bounced = true;
                    }
                    None => {
                        v[0] = 0;
                        v[2] = 0;
                    }
                }
                t.pos.x = before.x;
                t.pos.z = before.z;
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
                t.last_hit = None;
                t.bounced = false;
            } else {
                t.state = ThingState::Flying(v);
            }
        }
    }
}
