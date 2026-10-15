//! The player (object 0x800e2b48): the stick, the ground move, the floor
//! under it, and its base animation track.
//!
//! The update (0x8005a068) as far as it is ported: the stick (0x8005be24)
//! sets a wanted heading and a speed, the heading turns toward it
//! (0x8005bc80), the ground move adds speed * dt along the facing
//! (0x8005caac, 0x8005cb48), and the floor query (0x800627a0) puts the feet
//! on the floor or sends the player back to where the frame began.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::col::Col;
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::model::Models;
use rugrats_render::Vtx;
use rugrats_render::models::posed_triangles;

use crate::animator::Animator;
use crate::collide;
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::input::{self, Pads};

/// dt a frame at 30 fps (0x800c3f1c: 5 a vertical blank).
pub const DT: i32 = 10;
/// Speed factors (0x800c4200 running, 0x800c4204 walking).
const RUN: i32 = 17;
const WALK: i32 = 8;
/// The stick's dead zone (0x8005be24) and the widest turn taken at once
/// (0x800c42cc), and the speed above which any turn is (0x800c42c8).
const DEAD: i32 = 71;
const SNAP_TURN: i32 = 1024;
const MOVING: i32 = 100;
/// The turn rate (0x800c41fc): 1025 * dt >> 6 a frame.
const TURN: i32 = 1025;
/// The player's character: Tommy (0x800c5680 = 1).
const TOMMY: u16 = 1;
/// The jump's take-off sound.
const TAKE_OFF: u8 = 6;
/// Gravity (0x80061330): vy += 14 * dt, less 5 while jump is held.
const GRAVITY: i32 = 14;
const LIFT: i32 = -5;

/// Animation states (0x800572a8's argument) the port uses.
pub mod anim {
    pub const STAND: i16 = 1;
    pub const RUN: i16 = 2;
    pub const JUMP: i16 = 3;
    pub const LAND: i16 = 4;
    pub const FALL: i16 = 15;
    pub const JUMP_MOVING: i16 = 28;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stand,
    Walk,
    Run,
    Jump,
    Fall,
    Land,
}

pub struct Player {
    pub pos: [i32; 3],
    /// Heading (+0x24) and the one wanted (+0x2c), 4096 a turn.
    pub heading: i32,
    pub target: i32,
    pub speed: i32,
    /// Air velocity (+0x170): x, y, z.
    pub v: [i32; 3],
    pub state: State,
    pub floor: i32,
    /// Feet to origin (hierarchy +0x4c, the root's y offset).
    feet: i32,
    hierarchy: Hierarchy,
    pub animator: Animator,
    /// Sound effects asked for this frame (0x8006f644's numbers).
    pub effects: Vec<u8>,
}

impl Player {
    /// Puts the player at `start` with heading `heading` on the floor
    /// (0x800596e8), with `hierarchy`.
    pub fn new(disc: &GameDisc, hierarchy: Hierarchy, start: [i32; 3], heading: i32, col: &Col, anims: &Anims) -> Result<Player, String> {
        let animator = Animator::load(disc, TOMMY)?;
        let feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        let mut p = Player {
            pos: start,
            heading,
            target: heading,
            speed: 0,
            v: [0; 3],
            state: State::Stand,
            floor: start[1],
            feet,
            hierarchy,
            animator,
            effects: Vec::new(),
        };
        let q = collide::query(col, start);
        if q.tile.is_some() {
            p.floor = q.floor;
            p.pos[1] = q.floor + feet;
        }
        p.animator.set(anim::STAND, anims);
        Ok(p)
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left), for tools.
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        self.animator.track()
    }

    pub fn anm(&self, a: i16) -> Option<usize> {
        self.animator.anm(a)
    }

    /// One frame of play. `camera_heading` is the camera's (4096 a turn).
    pub fn frame(&mut self, pads: &Pads, camera_heading: i32, col: &Col, boxes: &[Box3], anims: &Anims, trig: &Trig) {
        self.effects.clear();
        let held = |b: usize| pads.held(0, b);
        let pressed = |b: usize| pads.pressed(0, b);
        let start = self.pos;
        // The stick: digital directions are full (4096) on their axis;
        // up is +z.
        let sx = if held(input::LEFT) { -4096 } else if held(input::RIGHT) { 4096 } else { 0 };
        let sz = if held(input::UP) { 4096 } else if held(input::DOWN) { -4096 } else { 0 };
        let magnitude = (((sx * sx + sz * sz) as f64).sqrt() as i32).min(4096);
        let on_ground = matches!(self.state, State::Stand | State::Walk | State::Run | State::Land);
        if magnitude >= DEAD {
            let dir = (((sx as f64).atan2(sz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
            let want = (dir + camera_heading).rem_euclid(4096);
            let factor = if held(input::SQUARE) { WALK } else { RUN };
            let turn = wrap(want - self.heading);
            if turn.abs() <= SNAP_TURN || self.speed >= MOVING || !on_ground {
                self.heading = want;
                self.target = want;
                self.speed = magnitude * factor >> 12;
            } else {
                self.target = want;
                self.speed = 0;
            }
        } else {
            self.speed = 0;
        }
        // Turn toward the wanted heading (0x8005bbf4).
        let step = TURN * DT >> 6;
        let d = wrap(self.target - self.heading);
        self.heading = (self.heading + d.clamp(-step, step)).rem_euclid(4096);
        let (s, c) = (trig.sin(self.heading), trig.cos(self.heading));
        if on_ground {
            // The ground move along the facing (0x8005caac).
            self.pos[0] += s * self.speed * DT >> 12;
            self.pos[2] += c * self.speed * DT >> 12;
            if pressed(input::CROSS) {
                // The launch (0x80060f1c): up 1000, and along the facing
                // 40 a unit of speed when running, else 12.
                let along = if self.speed > WALK { 40 } else { 12 } * self.speed;
                self.v = [s * along >> 12, -1000, c * along >> 12];
                self.state = State::Jump;
                // The take-off's sound (0x800562bc: effect 6).
                self.effects.push(TAKE_OFF);
            }
        } else {
            // The air (0x80061330): gravity 14, less 5 while jump is held
            // on the way up; drag 32/4096 of the speed.
            let lift = if held(input::CROSS) && self.v[1] < 0 { LIFT } else { 0 };
            let a = GRAVITY + lift;
            for k in [0, 2] {
                self.v[k] -= self.v[k] * 32 / 4096;
                self.pos[k] += self.v[k] * DT >> 6;
            }
            self.pos[1] += (self.v[1] * DT + a * DT * DT / 2) >> 6;
            self.v[1] = (self.v[1] + a * DT).min(4096);
        }
        // The floor (0x800627a0): the feet on it, or back where the frame
        // began when the point is not in the open.
        let probe = [self.pos[0], self.pos[1] - self.feet - 1, self.pos[2]];
        let q = collide::query(col, probe);
        if q.kind != 0 && q.tile.is_none() || q.kind == 2 {
            self.pos = start;
        } else if q.kind != 0 {
            // Into a wall or a step too high: back across, keep the height.
            let back = collide::query(col, [start[0], probe[1], start[2]]);
            self.pos = [start[0], self.pos[1], start[2]];
            if back.kind == 0 {
                self.floor = back.floor;
            }
        } else {
            self.floor = q.floor;
        }
        // Objects' boxes (0x80065210 against 0x8006a714): one the body
        // runs into sends it back across; one whose top is within a step
        // of the feet (or below them, falling) is floor.
        let start_feet = start[1] - self.feet;
        let feet_y = self.pos[1] - self.feet;
        for b in boxes {
            if !b.covers(self.pos, RADIUS) {
                continue;
            }
            let top = b.min_y;
            let head = feet_y - HEIGHT;
            if top >= start_feet.min(feet_y) - STEP_UP {
                // Standing room on it: a floor if higher than the one found.
                if top < self.floor {
                    self.floor = top;
                }
            } else if b.max_y > head && b.depth(self.pos, RADIUS) > b.depth(start, RADIUS) {
                self.pos[0] = start[0];
                self.pos[2] = start[2];
            }
        }
        let feet_y = self.pos[1] - self.feet;
        if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            if self.floor - feet_y > STEP_DOWN {
                self.state = State::Fall;
                self.v = [s * self.speed >> 6, 0, c * self.speed >> 6];
            } else {
                self.pos[1] = self.floor + self.feet;
            }
        } else if feet_y >= self.floor && self.v[1] >= 0 {
            self.pos[1] = self.floor + self.feet;
            self.v = [0; 3];
            self.state = State::Land;
        } else if self.v[1] > 0 {
            self.state = State::Fall;
        }
        if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            self.state = match self.speed {
                0 if self.state == State::Land && self.animator.playing(anim::LAND) => State::Land,
                0 => State::Stand,
                s if s <= WALK => State::Walk,
                _ => State::Run,
            };
        }
        let a = match self.state {
            State::Stand => anim::STAND,
            State::Walk => anim::STAND,
            State::Run => anim::RUN,
            State::Jump => if self.speed > 0 { anim::JUMP_MOVING } else { anim::JUMP },
            State::Fall => anim::FALL,
            State::Land => anim::LAND,
        };
        self.animator.set(a, anims);
        self.animator.step(anims);
    }

    /// The player's triangles, turned by its heading (0x80020d10).
    pub fn triangles(&self, models: &Models, anims: &Anims, trig: &Trig, out: &mut Vec<Vtx>) {
        let e = trig.euler([self.heading as i16, 0, 0]);
        let rows = Mat3::from_cols_array(&e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap()).transpose();
        let pos = Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32);
        posed_triangles(&self.hierarchy, models, &self.animator.locals(&self.hierarchy, anims), rows.transpose(), pos, out);
    }
}

/// How far below the feet a floor may drop and still be walked onto, and
/// how far above them a box's top may stand and still be stepped onto
/// (0x80066014 tests 384).
const STEP_DOWN: i32 = 384;
const STEP_UP: i32 = 384;
/// The body's radius and height for the object test.
const RADIUS: i32 = 160;
const HEIGHT: i32 = 600;

/// An object's collision box: its model box (`lo`, `hi`) in the object's
/// space, the object's rotation and position, and the world heights it
/// spans (y down: `min_y` its top).
#[derive(Clone, Copy, Debug)]
pub struct Box3 {
    pub lo: [i32; 3],
    pub hi: [i32; 3],
    pub rot: Mat3,
    pub pos: Vec3,
    pub min_y: i32,
    pub max_y: i32,
}

impl Box3 {
    /// Whether a circle of radius `r` at `p` (x, z) overlaps the box seen
    /// from above.
    pub fn covers(&self, p: [i32; 3], r: i32) -> bool {
        self.depth(p, r) > 0.0
    }

    /// How deep a circle of radius `r` at `p` is in the box seen from above:
    /// the distance to the nearest side, negative outside.
    pub fn depth(&self, p: [i32; 3], r: i32) -> f32 {
        let local = self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
        let r = r as f32;
        let x = (local.x - (self.lo[0] as f32 - r)).min(self.hi[0] as f32 + r - local.x);
        let z = (local.z - (self.lo[2] as f32 - r)).min(self.hi[2] as f32 + r - local.z);
        x.min(z)
    }
}

/// An angle difference in -2048..2048.
fn wrap(a: i32) -> i32 {
    (a + 2048).rem_euclid(4096) - 2048
}
