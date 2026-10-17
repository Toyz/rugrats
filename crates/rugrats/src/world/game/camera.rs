//! The follow camera (0x80027304): an eye on a spring behind the player,
//! a look-at that trails the player and leads it as it runs, and the
//! reset behind the player (0x80026ddc).

use super::*;
use crate::world::collide::{self, Segment, fix, length, normalize};
use crate::world::flights::Matrix;

/// The aim above the player's origin (0x800c3f58; 400 for Spike).
const AIM_UP: i32 = 768;
const AIM_UP_SPIKE: i32 = 400;

/// The follow distance (0x800c4a4c; 6400 in level 7) and the height
/// (0x800c3f5c, set by level and room).
pub(super) const BEHIND: i32 = 4096;
pub(super) const BEHIND_LEVEL_7: i32 = 6400;

const EYE_UP: i32 = 1024;

/// The eye's farthest from the player (0x800c4a68; 7200 in level 7) and
/// nearest (0x80027304 at 0x800289a0).
const FAR: i32 = 4800;
const FAR_LEVEL_7: i32 = 7200;
const NEAR: i32 = 1024;

/// The bands the eye's height and the look-at's keep about their goals
/// (0x800c3f54, 0x800c3f50).
const EYE_BAND: i32 = 1512;
const LOOK_BAND: i32 = 512;

/// The eye's top speed (0x80027304 at 0x8002880c).
const TOP_SPEED: i32 = 2796;

/// The integration step (0x800c3f1c: 5 a vsync, two a frame), and the
/// timers' (0x800c3f20: the step * 4096 / 300).
const STEP: i32 = 10;
const TICK: i32 = 136;

/// The reset's offsets from the aim point (0x800b6e18: x, z, and whether
/// R1 may pick it), turned by the player's heading: behind, in front, to
/// either side, then the diagonals.
const RESETS: [(i32, i32, bool); 8] = [
    (0, 2048, true),
    (0, -2048, true),
    (2048, 0, true),
    (-2048, 0, true),
    (2048, 2048, false),
    (-2048, 2048, false),
    (2048, -2048, false),
    (-2048, -2048, false),
];

/// 0x800c4a3c's follow bits.
mod bit {
    /// Swinging aside to the left or right (the player faces the
    /// camera), and the side last taken.
    pub const SWING_LEFT: u32 = 4;
    pub const SWING_RIGHT: u32 = 8;
    pub const SWUNG_LEFT: u32 = 0x8000;
    pub const SWUNG_RIGHT: u32 = 0x1_0000;
    /// At rest; round behind the player; at the follow distance.
    pub const REST: u32 = 64;
    pub const BEHIND: u32 = 128;
    pub const AT_DISTANCE: u32 = 256;
    /// The look-at's height catching up.
    pub const LOOK_RISING: u32 = 1024;
    /// The eye at its height.
    pub const LEVEL: u32 = 2048;
    /// The sign of the last push by axis, x, y and z positive and the same
    /// negative, and an axis held still after its push turned.
    pub const PLUS: [u32; 3] = [0x10_0000, 0x20_0000, 0x40_0000];
    pub const MINUS: [u32; 3] = [0x100_0000, 0x200_0000, 0x400_0000];
    pub const STILL: [u32; 3] = [0x800_0000, 0x1000_0000, 0x2000_0000];
}

/// The follow camera's state (0x80027304).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Follow {
    /// The eye (0x800de530) and its velocity (camera +0x98).
    eye: [i32; 3],
    vel: [i32; 3],
    /// What it looks at (0x800de454).
    look: [i32; 3],
    /// 0x800c4a3c's follow bits ([`bit`]).
    flags: u32,
    /// The swing's time left (0x800c4a58), the look-ahead and its speed
    /// (0x800c4a5c, 0x800c4a38).
    swing: i32,
    ahead: i32,
    ahead_speed: i32,
    /// The room the camera is in (camera +0x80).
    room: Option<usize>,
}

/// A look aside (0x8002729c): L2 and R2 swing the view 341 left or right
/// about the eye, L1 tilts it 341 up; it eases out and back by itself
/// (0x800247fc, the offsets at 0x800de430/434, the targets at 438/43c,
/// 0x800c4a3c bits 16 and 32).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Look {
    pub yaw: i32,
    pub pitch: i32,
    to_yaw: i32,
    to_pitch: i32,
    /// Under way (bit 16) and on the way back (bit 32).
    on: bool,
    back: bool,
}

impl Look {
    /// A look toward (yaw, pitch); none while one is under way.
    pub fn start(&mut self, to_yaw: i32, to_pitch: i32) {
        if !self.on {
            *self = Look { to_yaw, to_pitch, on: true, ..Look::default() };
        }
    }

    /// R1 (0x80026ddc) drops it.
    pub fn cancel(&mut self) {
        *self = Look::default();
    }

    /// One frame (0x800247fc): the step is (rsin(offset * 2048 / target)
    /// / 16 + 128) * dt (136) / 4096, slow at the ends and fast midway;
    /// at the target it turns back, at 0 it ends.
    pub fn step(&mut self, trig: &Trig) {
        if !self.on {
            return;
        }
        let (off, to) = if self.to_yaw != 0 { (&mut self.yaw, self.to_yaw) } else { (&mut self.pitch, self.to_pitch) };
        if to == 0 {
            return;
        }
        let s = trig.sin((*off << 11) / to);
        let v = ((if s < 0 { s + 15 } else { s } >> 4) + 128) * TICK;
        let step = if v < 0 { v + 4095 } else { v } >> 12;
        if self.back {
            *off += if to > 0 { -step } else { step };
            if (to > 0 && *off <= 0) || (to < 0 && *off >= 0) {
                *self = Look::default();
            }
        } else {
            *off += if to > 0 { step } else { -step };
            if (to > 0 && *off >= to) || (to < 0 && *off <= to) {
                *off = to;
                self.back = true;
            }
        }
    }
}

/// The follow camera's height by level (0x800290a4, 0x800291f8: 512 in
/// levels 4 and 15, -1000 in 7).
pub(super) fn default_eye_up(level: Level) -> i32 {
    match level {
        Level::CYNTHIA | Level::MIRRORLAND => 512,
        Level::REPTAR_2010 => -1000,
        _ => EYE_UP,
    }
}

fn add(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|k| a[k].wrapping_add(b[k]))
}

fn sub(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|k| a[k].wrapping_sub(b[k]))
}

fn flat(v: [i32; 3]) -> [i32; 3] {
    [v[0], 0, v[2]]
}

fn vec3(v: [i32; 3]) -> Vec3 {
    Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32)
}

/// `v` scaled to length `len` (0x80076944): VectorNormal, then each
/// component times `len` >> 12 toward zero.
fn set_length(v: [i32; 3], len: i32) -> [i32; 3] {
    normalize(v).map(|c| fix(c, len))
}

/// |(x, z)| (0x8002495c), 0 when either is 65536 or more.
fn flat_length(v: [i32; 3]) -> i32 {
    length(flat(v))
}

/// `a / 2^n` rounded toward zero, as the game's `(a + 2^n - 1) >> n`.
fn shr0(a: i32, n: u32) -> i32 {
    if a < 0 { (a + (1 << n) - 1) >> n } else { a >> n }
}

/// m v (ApplyMatrixLV, 0x8009f6f8): each component split at bit 15, the
/// high parts times m unshifted (then times 8), the low parts times m
/// >> 12.
fn apply(m: &Matrix, v: [i32; 3]) -> [i32; 3] {
    let split = |c: i32| if c < 0 { (-((-c) >> 15), -((-c) & 0x7fff)) } else { (c >> 15, c & 0x7fff) };
    let parts = v.map(split);
    [0, 1, 2].map(|r| {
        let hi: i64 = (0..3).map(|k| m[r][k] as i64 * parts[k].0 as i64).sum();
        let lo: i64 = (0..3).map(|k| m[r][k] as i64 * parts[k].1 as i64).sum();
        ((lo >> 12) + hi * 8) as i32
    })
}

fn transpose(m: &Matrix) -> Matrix {
    [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]))
}

/// The player's rotation from its heading `a` (0x80020d10 with angles
/// (a, 0, 0), then MatrixNormal 0x8009f398): forward (0, 0, 1) turns to
/// (sin a, 0, cos a).
fn heading_matrix(trig: &Trig, a: i32) -> Matrix {
    let (s, c) = (trig.sin(a), trig.cos(a));
    [normalize([c, 0, s]), [0, 4096, 0], normalize([-s, 0, c])]
}

/// pos and vel stepped by accel `a` over `dt` (0x8004fbb4): pos += (a dt^2
/// / 2 + v dt) >> 6, v += a dt.
fn integrate(dt: i32, a: [i32; 3], vel: &mut [i32; 3], pos: &mut [i32; 3]) {
    for k in 0..3 {
        let ad = a[k].wrapping_mul(dt);
        pos[k] = pos[k].wrapping_add(((ad.wrapping_mul(dt) >> 1).wrapping_add(vel[k].wrapping_mul(dt))) >> 6);
        vel[k] = vel[k].wrapping_add(ad);
    }
}

impl Game {
    /// The room's fixed camera post in use (kind 3), if any.
    pub fn fixed_camera(&self) -> Option<usize> {
        self.post_camera
    }

    /// The projection distance in play.
    pub fn focal(&self) -> f32 {
        self.focal
    }

    /// The follow camera's eye (0x800de530) and look-at (0x800de454).
    pub fn follow_camera(&self) -> ([i32; 3], [i32; 3]) {
        (self.follow.eye, self.follow.look)
    }

    /// The follow camera's whole state, for comparing with the original:
    /// eye, its velocity (camera +0x98), look-at, the follow bits
    /// (0x800c4a3c), the swing's time (0x800c4a58), the look-ahead and its
    /// speed (0x800c4a5c, 0x800c4a38).
    pub fn follow_state(&self) -> ([i32; 3], [i32; 3], [i32; 3], u32, i32, i32, i32) {
        let f = &self.follow;
        (f.eye, f.vel, f.look, f.flags, f.swing, f.ahead, f.ahead_speed)
    }

    /// Puts the follow camera behind heading `yaw` at once, as the follow
    /// setup does for the player's (0x80025e90: straight behind, then the
    /// reset 0x80026ddc); for levels and tools.
    pub fn set_camera_yaw(&mut self, yaw: f32) {
        let yaw = yaw.round() as i32;
        self.look.cancel();
        let at = self.aim_point(false);
        let m = heading_matrix(&self.trig, yaw);
        let behind = add(at, apply(&m, [0, 0, -self.behind]));
        self.follow = Follow {
            eye: [behind[0], at[1] - self.eye_up, behind[2]],
            look: [at[0], at[1] - self.look_lift(), at[2]],
            room: self.follow.room,
            ..Follow::default()
        };
        let body = self.aim_point(true);
        self.reset_camera(body, yaw, false);
        self.follow_done();
        self.draw();
    }

    /// The heading the stick is turned by (0x8005be24): the camera
    /// matrix's, ratan2(-m[0][2], m[2][2]) (0x800a0328), with the look
    /// aside in it.
    pub fn camera_heading(&self) -> i32 {
        let m = self.camera_matrix();
        self.flights.angles.ratan2(-m[0][2], m[2][2]).rem_euclid(4096)
    }

    /// The last frame's world-to-camera rows, 4096 = 1 (for tools).
    pub fn camera_rows(&self) -> Matrix {
        self.camera_matrix()
    }

    /// The heading rows `m` give, as the stick reads it (0x800a0328 of
    /// -m02 and m22; for tools reading the original's +0x224).
    pub fn heading_of_rows(&self, m: &Matrix) -> i32 {
        self.flights.angles.ratan2(-m[0][2], m[2][2]).rem_euclid(4096)
    }

    /// The last frame's world-to-camera rows, 4096 = 1.
    fn camera_matrix(&self) -> Matrix {
        if let Some(m) = self.rows {
            return m;
        }
        let rows = self.view.1;
        [0, 1, 2].map(|r| rows.row(r).to_array().map(|v| (v * 4096.0).round() as i32))
    }

    /// The follow camera's rows (0x8001e838): from the eye as drawn toward
    /// `target` (0x800211e8), then the look aside about the eye, the swing
    /// (0x80020c30) if there is one, else the tilt (0x80020ca0), put before
    /// them (MulMatrix0, 0x8009f488).
    pub(super) fn follow_rows(&self, target: Vec3) -> Matrix {
        let int = |v: Vec3| [v.x as i32, v.y as i32, v.z as i32];
        let m = self.trig.look_at(int(self.eye), int(target));
        let (yaw, pitch) = (self.look.yaw, self.look.pitch);
        let r = if yaw != 0 {
            let (s, c) = (self.trig.sin(yaw), self.trig.cos(yaw));
            [[c, 0, -s], [0, 4096, 0], [s, 0, c]]
        } else if pitch != 0 {
            let (s, c) = (self.trig.sin(pitch), self.trig.cos(pitch));
            [[4096, 0, 0], [0, c, s], [0, -s, c]]
        } else {
            return m;
        };
        // MVMVA per column, sf 1, saturated to 16 bits.
        [0, 1, 2].map(|i| {
            [0, 1, 2].map(|j| {
                let sum: i64 = (0..3).map(|k| r[i][k] as i64 * m[k][j] as i64).sum();
                ((sum >> 12) as i32).clamp(-0x8000, 0x7fff)
            })
        })
    }

    /// The aim above the player (0x800c3f58).
    fn aim(&self) -> i32 {
        if self.character == 6 { AIM_UP_SPIKE } else { AIM_UP }
    }

    /// The aim point (0x80027304's local_150): the player's position less
    /// the aim; floating, from the depth anchor (+0x65c); in level 7 from
    /// the floor under him (+0x148), not the stomp's bob. The reset aims
    /// from the body (`body`, 0x80026ddc).
    fn aim_point(&self, body: bool) -> [i32; 3] {
        let p = self.player.pos;
        // (The body's height as the original's +0x1c has it: with the
        // animation's root.)
        let y_body = p[1] + self.player.root_y;
        let y = match self.player.float {
            _ if body => y_body,
            Some((anchor, _)) => anchor,
            None if self.level == Level::REPTAR_2010 => self.player.floor,
            None => y_body,
        };
        [p[0], y - self.aim(), p[2]]
    }

    /// The aim as a point, for the fixed and shaft cameras.
    pub(super) fn target(&self) -> Vec3 {
        vec3(self.aim_point(false))
    }

    pub(super) fn follow_look(&self) -> Vec3 {
        vec3(self.follow.look)
    }

    /// 0x800c3f60: 256 in level 15, else 0.
    fn look_lift(&self) -> i32 {
        if self.level == Level::MIRRORLAND { 256 } else { 0 }
    }

    /// The line test (0x80090554): the objects in the rooms along the
    /// segment (0x80090340 -> 0x8006a9cc with 0x8000: not template flag 16
    /// or 8, no characters; here their boxes, `solid`), then the height
    /// field (0x8002cd7c); the nearer stop wins.
    fn blocked(&self, from: [i32; 3], to: [i32; 3]) -> Option<Segment> {
        let mut hit = Segment::default();
        let ground = (collide::segment(&self.col, from, to, &mut hit) != 0).then_some(hit);
        let span = |p: [i32; 3]| {
            let d: [f32; 3] = std::array::from_fn(|k| (p[k] - from[k]) as f32);
            d.iter().map(|v| v * v).sum::<f32>()
        };
        let object = self
            .solid
            .iter()
            .filter_map(|b| b.segment_hit(from, to))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, point, normal)| Segment { depth: 0, point, normal, ..Segment::default() });
        match (ground, object) {
            (Some(g), Some(o)) => Some(if span(o.point) < span(g.point) { o } else { g }),
            (g, o) => g.or(o),
        }
    }

    /// The reset (0x80026ddc): the eye to the first offset of [`RESETS`]
    /// (R1's: those it may pick, at the follow distance, then at 2048)
    /// whose segment from `at` is clear, or, `nearest`, to the clear one
    /// nearest the eye; the look-at to the player and the camera at rest
    /// (bits 0x9c0). False when every segment is blocked.
    fn reset_camera(&mut self, at: [i32; 3], heading: i32, nearest: bool) -> bool {
        let m = heading_matrix(&self.trig, heading);
        let (dist, height, eye) = (self.behind, self.eye_up, self.follow.eye);
        let mut best: Option<(usize, [i32; 3], u32)> = None;
        for scaled in [true, false] {
            for (k, &(x, z, r1)) in RESETS.iter().enumerate() {
                if !(nearest || r1) {
                    continue;
                }
                let o = if scaled { [x.signum() * dist, height, z.signum() * dist] } else { [x, height, z] };
                let c = sub(at, apply(&m, o));
                if self.blocked(at, c).is_some() {
                    continue;
                }
                let (dx, dz) = (c[0].wrapping_sub(eye[0]), c[2].wrapping_sub(eye[2]));
                let d2 = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz)) as u32;
                if nearest {
                    if d2 < best.map_or(0x7fff_ffff, |b| b.2) {
                        best = Some((k, c, d2));
                    }
                    continue;
                }
                // R1: the first clear at the distance; at 2048 the first
                // clear only when none was, or when it is the one behind
                // and the first was not.
                if scaled || best.is_none_or(|b| k == 0 && b.0 != 0) {
                    best = Some((k, c, d2));
                }
                break;
            }
        }
        let Some((_, c, _)) = best else { return false };
        let p = self.player.pos;
        let look = [p[0], p[1] - self.aim() - self.look_lift(), p[2]];
        let keep = if self.level.0 >= 22 { 512 } else { 0x4200 };
        let f = &mut self.follow;
        *f = Follow { eye: c, look, flags: (f.flags & keep) | 0x9c0, room: f.room, ..Follow::default() };
        true
    }

    /// R1 (0x80026ddc with no aim): back behind the player, the look
    /// aside dropped.
    pub(super) fn camera_behind(&mut self) {
        self.look.cancel();
        let at = self.aim_point(true);
        self.reset_camera(at, self.player.heading, false);
    }

    /// An eye in a wall resets to the clear offset nearest it (0x80027304
    /// into 0x80026ddc, 0x800c3f4c = 1).
    fn unstick(&mut self, at: [i32; 3]) -> bool {
        self.reset_camera(at, self.player.heading, true)
    }

    /// The wall probes (0x800263c4) from the eye: 768 toward the push (or
    /// the player) and 768 to either side; a wall ahead pushes 37 off it,
    /// a wall aside up to 74 the nearer it is. True when one was hit.
    fn wall_push(&self, accel: &mut [i32; 3], eye: [i32; 3], at: [i32; 3]) -> bool {
        let mut push = [0; 3];
        let mut hit = false;
        let n = normalize(flat(if *accel == [0; 3] { sub(at, eye) } else { *accel }));
        let ahead = n.map(|c| shr0(c * 768, 12));
        if let Some(h) = self.blocked(eye, add(eye, ahead)) {
            let w = normalize(flat(h.normal));
            push = set_length([2 * w[0] + n[0], 0, 2 * w[2] + n[2]], 37);
            hit = true;
        }
        let side = [ahead[2], 0, -ahead[0]];
        for to in [add(eye, side), sub(eye, side)] {
            if let Some(h) = self.blocked(eye, to) {
                let near = length(sub(h.point, eye)).clamp(0, 768);
                let k = 2048 - (near << 11) / 768;
                let half = [h.normal[0] / 2, 0, h.normal[2] / 2];
                push = add(push, set_length(half, (k << 16) / 1_800_000));
                hit = true;
            }
        }
        *accel = add(*accel, flat(push));
        hit
    }

    /// The view test (0x80026914): from the player (512 ahead of it while
    /// it runs) to the eye; a wall between pushes 18 off it. True when
    /// blocked. (Its side tests only pick between two answers 0x80027304
    /// treats alike.)
    fn view_blocked(&self, accel: &mut [i32; 3], eye: [i32; 3], at: [i32; 3], m: &Matrix) -> bool {
        let mut from = at;
        if self.follow.flags & bit::REST == 0 && self.player.speed != 0 {
            let to = add(at, apply(m, [0, 0, 512]));
            from = match self.blocked(at, to) {
                Some(h) => {
                    let back = [0, 1, 2].map(|k| h.point[k] + shr0(at[k] - h.point[k], 8));
                    if back == h.point { at } else { back }
                }
                None => to,
            };
            // Outside every room (0x8002b660): from the player after all.
            if self.room_from(from, self.room).is_none() {
                from = at;
            }
        }
        let Some(h) = self.blocked(from, eye) else { return false };
        *accel = add(*accel, flat(set_length(h.normal, 18)));
        true
    }

    /// One frame of the follow camera (0x80027304), `moved` when the
    /// player moved this frame (+0x188 bit 32).
    pub(super) fn follow(&mut self, moved: bool) {
        let floating = self.player.float.is_some();
        let level7 = self.level == Level::REPTAR_2010;
        let m = heading_matrix(&self.trig, self.player.heading);
        let mt = transpose(&m);
        let (dist, height, lift) = (self.behind, self.eye_up, self.look_lift());
        let far = if level7 { FAR_LEVEL_7 } else { FAR };
        let at = self.aim_point(false);
        let eye = self.follow.eye;
        // The goal: `dist` behind, pulled in front of walls (0x8002cd7c).
        let mut goal = add(at, apply(&m, [0, 0, -dist]));
        let mut hit = Segment::default();
        if collide::segment(&self.col, at, goal, &mut hit) == 1 {
            goal = hit.point;
        }
        let to_goal = sub(goal, eye);
        let to_player = sub(at, eye);
        let d = flat_length(to_player);
        if self.follow.swing > 0 {
            self.follow.swing -= TICK;
        }
        self.follow.flags &= !0x1000;
        let moving = self.player.speed != 0 || moved;
        let mut accel = [0; 3];
        let mut rest = false;
        let mut left = false;
        let mut seen = true;
        if !moving && self.follow.flags & (bit::REST | bit::LEVEL) == bit::REST | bit::LEVEL {
            // At rest with the player still: round behind it at the follow
            // distance, at 512 (0x800284f4).
            if self.follow.flags & bit::AT_DISTANCE == 0 {
                accel = set_length(flat(to_player), if d < dist { -512 } else { 512 });
            }
            if self.follow.flags & bit::BEHIND == 0 {
                left = apply(&mt, to_player)[0] <= 0;
                let side = if left { [-to_player[2], 0, to_player[0]] } else { [to_player[2], 0, -to_player[0]] };
                accel = add(accel, set_length(side, 512));
            }
            if self.wall_push(&mut accel, eye, at) || self.view_blocked(&mut accel, eye, at, &m) {
                accel = [0; 3];
                self.follow.flags |= bit::BEHIND | bit::AT_DISTANCE;
            }
            self.follow.vel = accel;
            accel = [0; 3];
        } else {
            self.follow.flags &= !(bit::REST | bit::BEHIND | bit::AT_DISTANCE);
            let swinging = bit::SWING_LEFT | bit::SWING_RIGHT;
            // The player's facing in the camera's frame: toward the camera
            // (z <= 0) and near, the eye swings aside (0x80027728).
            let facing = apply(&self.camera_matrix(), apply(&m, [0, 0, 2048]));
            if facing[2] <= 0 || self.follow.flags & swinging != 0 {
                if d < 2049 {
                    if level7 {
                        if self.unstick(at) {
                            return self.follow_done();
                        }
                    } else {
                        if self.follow.flags & swinging == 0 {
                            self.start_swing(facing[0], at, &m);
                        }
                        let x = if self.follow.flags & bit::SWING_LEFT != 0 { -74 } else { 74 };
                        accel = apply(&m, [x, 0, if floating { -37 } else { -18 }]);
                    }
                } else {
                    self.follow.flags &= !swinging;
                    if d < 4097 {
                        // Too near: away along the player's axis.
                        let push = if floating { 9 } else { 18 };
                        accel = apply(&m, [0, 0, if apply(&mt, to_player)[2] > 0 { -push } else { push }]);
                    }
                }
            }
            if d > dist {
                accel = add(accel, set_length(flat(to_goal), if floating { 10 } else { 16 }));
            }
            accel[1] = 0;
            let Some(rise) = self.height_push(eye, at, floating, level7) else { return self.follow_done() };
            accel[1] += rise;
            self.wall_push(&mut accel, eye, at);
            if self.view_blocked(&mut accel, eye, at, &m) {
                seen = false;
                accel = add(accel, set_length(flat(to_goal), if floating { 20 } else { 32 }));
            }
            let f = &mut self.follow;
            // Drag: |v| / 32 against the velocity.
            let speed = length(f.vel);
            if speed > 0 {
                let drag = set_length(f.vel, shr0(-(speed << 7), 12));
                if drag != [0; 3] {
                    accel = add(accel, drag);
                } else if accel == [0; 3] {
                    f.vel = [0; 3];
                }
            }
            if moving {
                // The pushes' signs; z's set y's bits (0x80028250).
                f.flags &= 0xc08f_ffff;
                for (a, k) in [(accel[0], 0), (accel[1], 1), (accel[2], 1)] {
                    if a > 0 {
                        f.flags |= bit::PLUS[k];
                    } else if a < 0 {
                        f.flags |= bit::MINUS[k];
                    }
                }
            } else {
                // Standing: an axis whose push turned holds still.
                for k in 0..3 {
                    if f.flags & bit::STILL[k] != 0 {
                        accel[k] = 0;
                        continue;
                    }
                    let now = match accel[k].signum() {
                        1 => bit::PLUS[k],
                        -1 => bit::MINUS[k],
                        _ => 0,
                    };
                    let was = f.flags & (bit::PLUS[k] | bit::MINUS[k]);
                    if now != 0 && was != 0 && was != now {
                        f.flags = (f.flags & !was) | bit::STILL[k];
                        accel[k] = 0;
                    } else {
                        f.flags = (f.flags & !was) | now;
                    }
                }
                if accel == [0; 3] {
                    rest = true;
                    f.vel = [0; 3];
                }
            }
        }
        // The step (0x8004fbb4).
        let f = &mut self.follow;
        let before = f.vel;
        integrate(STEP, accel, &mut f.vel, &mut f.eye);
        let rel = sub(f.eye, at);
        if f.flags & bit::REST != 0 {
            // Round far enough: at the distance, or past the player's back.
            let now = flat_length(rel);
            if f.flags & bit::AT_DISTANCE == 0 && ((d < dist && now >= dist) || (d > dist && now <= dist)) {
                f.flags |= bit::AT_DISTANCE;
            }
            let x = apply(&mt, rel)[0];
            if f.flags & bit::BEHIND == 0 && ((left && x <= 0) || (!left && x > 0)) {
                f.flags |= bit::BEHIND | bit::AT_DISTANCE;
            }
        } else if rest {
            f.flags |= bit::REST;
        }
        if length(f.vel) as u32 > TOP_SPEED as u32 {
            f.vel = set_length(f.vel, TOP_SPEED);
        }
        // Turned back on itself: stop.
        if seen {
            let (a, b) = (normalize(before), normalize(f.vel));
            let dot = (0..3).fold(0i32, |s, k| s.wrapping_add(a[k].wrapping_mul(b[k])));
            if shr0(dot, 12) < 0 {
                f.vel = [0; 3];
                f.flags |= bit::REST;
            }
        }
        if accel == [0; 3] && f.vel == [0; 3] {
            f.flags |= bit::REST;
        }
        // Between 1024 and the farthest from the player, across.
        let d = flat_length(rel);
        if d > far {
            let n = normalize(flat(rel));
            f.eye[0] = at[0] + fix(n[0], far);
            f.eye[2] = at[2] + fix(n[2], far);
        } else if d < NEAR {
            let n = normalize(flat(rel));
            f.eye[0] = at[0] + shr0(n[0], 2);
            f.eye[2] = at[2] + shr0(n[2], 2);
        }
        // Within 1512 of its height over the look-at.
        let level = f.look[1] + lift - height;
        if level + EYE_BAND < f.eye[1] {
            f.eye[1] = level + EYE_BAND;
        } else if f.eye[1] < level - EYE_BAND {
            f.eye[1] = level - EYE_BAND;
        }
        self.look_ahead(at, d);
        self.follow_done();
    }

    /// The swing's side (0x80027774): the side the player faces across
    /// the view (or the last one, while the swing timer runs), unless a
    /// wall stands 2048 that way. (0x80062a08 also sets the player's
    /// +0x188 bit 2048: not ported.)
    fn start_swing(&mut self, across: i32, at: [i32; 3], m: &Matrix) {
        let f = &mut self.follow;
        let right = if f.swing <= 0 {
            f.flags &= !(bit::SWUNG_LEFT | bit::SWUNG_RIGHT);
            across >= 0
        } else {
            f.flags & bit::SWUNG_RIGHT != 0
        };
        f.swing = 4096;
        let side = apply(m, [2048, 0, 0]);
        let to = if right { add(at, side) } else { sub(at, side) };
        let mut hit = Segment::default();
        let clear = collide::segment(&self.col, at, to, &mut hit) == 0;
        self.follow.flags |=
            if clear == right { bit::SWUNG_RIGHT | bit::SWING_RIGHT } else { bit::SWUNG_LEFT | bit::SWING_LEFT };
    }

    /// The eye's vertical push (0x80027a94): toward the look-at less the
    /// height, kept 256 off the floor and ceiling at the eye and under
    /// anything over the eye or the player; 9 a frame (4 floating, 18 in
    /// level 15), none when the way across at that height is blocked.
    /// Blocked across at the eye's height but not lower, down to the floor
    /// less the height. None when the eye was in a wall and reset.
    fn height_push(&mut self, eye: [i32; 3], at: [i32; 3], floating: bool, level7: bool) -> Option<i32> {
        let height = self.eye_up;
        let row = |y: i32| ([eye[0], y, eye[2]], [at[0], y, at[2]]);
        let (a, b) = (row(eye[1] - 256), row(at[1] - self.aim() / 2));
        let walls = !floating && self.level != Level::MIRRORLAND;
        if walls && self.blocked(a.0, a.1).is_some() && self.blocked(b.0, b.1).is_none() {
            self.follow.flags &= !bit::LEVEL;
            let q = collide::query(&self.col, eye);
            if q.kind != 0 {
                return if self.unstick(at) { None } else { Some(0) };
            }
            let down = if level7 { -i32::from(q.ceiling + 256 < eye[1]) } else { i32::from(eye[1] < q.floor - height) };
            return Some(9 * down);
        }
        let mut goal = self.follow.look[1] + self.look_lift() - height;
        let q = collide::query(&self.col, eye);
        if q.kind == 0 {
            goal = goal.min(q.floor - 256).max(q.ceiling + 256);
            if !level7 && let Some(h) = self.blocked(eye, [eye[0], eye[1] - height, eye[2]]) {
                goal = goal.max(h.point[1]);
            }
        } else if self.unstick(at) {
            return None;
        }
        if !level7 && let Some(h) = self.blocked(at, [at[0], at[1] - height, at[2]]) {
            goal = goal.max(h.point[1] + 256);
        }
        if (goal - eye[1]).abs() <= 64 {
            self.follow.flags |= bit::LEVEL;
            return Some(0);
        }
        self.follow.flags &= !bit::LEVEL;
        let push = if floating {
            4
        } else if self.level == Level::MIRRORLAND {
            18
        } else {
            let (a, b) = row(goal);
            if self.blocked(a, b).is_some() || self.blocked(b, a).is_some() { 0 } else { 9 }
        };
        Some(if goal < eye[1] { -push } else { push })
    }

    /// The look-at (0x80028b10): the player's x and z plus a look-ahead
    /// of up to 150 a unit of speed along its heading (less nearer than
    /// 5192), eased by 1/4; its height after the player's at 298 (223
    /// floating) once 512 (256) off, and within 512.
    fn look_ahead(&mut self, at: [i32; 3], d: i32) {
        let (aim, lift) = (self.aim(), self.look_lift());
        let p = &self.player;
        let floating = p.float.is_some();
        let want = p.speed * 150;
        let f = &mut self.follow;
        let drag = -shr0(f.ahead_speed, 5);
        let (mut v, mut x) = ([0, 0, f.ahead_speed], [0, 0, f.ahead]);
        if f.ahead < want {
            integrate(STEP, [0, 0, drag + 9], &mut v, &mut x);
            (f.ahead, f.ahead_speed) = (x[2].min(want), v[2]);
        } else if f.ahead > want {
            integrate(STEP, [0, 0, drag - 9], &mut v, &mut x);
            (f.ahead, f.ahead_speed) = (x[2], v[2]);
            if f.ahead <= 0 {
                f.ahead = 0;
                if want == 0 {
                    f.ahead_speed = 0;
                }
            }
        }
        let lead = if d < 5192 { f.ahead.wrapping_mul(d) / 5192 } else { f.ahead };
        let lead = apply(&heading_matrix(&self.trig, p.heading), [0, 0, lead]);
        // (Level 11 with +0x188 bit 512 takes it at once: not ported.)
        f.look[0] += shr0(lead[0] + at[0] - f.look[0], 2);
        f.look[2] += shr0(lead[2] + at[2] - f.look[2], 2);
        let base = p.float.map_or(p.pos[1] + p.root_y, |(anchor, _)| anchor);
        let goal = base - aim - lift;
        let off = goal - f.look[1];
        if off.abs() > if floating { 256 } else { 512 } {
            f.flags |= bit::LOOK_RISING;
        }
        if f.flags & bit::LOOK_RISING != 0 {
            let rate = if floating { 223 } else { 298 };
            f.look[1] += ((if off < 0 { -rate } else { rate }) * STEP) >> 6;
            if (off >= 0) != (f.look[1] < goal) {
                f.look[1] = goal;
                f.flags &= !bit::LOOK_RISING;
            }
        }
        f.look[1] = f.look[1].clamp(goal - LOOK_BAND, goal + LOOK_BAND);
    }

    /// The eye as drawn (0x8002953c): the camera's room found from it
    /// (0x8002b660); outside every room it is drawn clamped into that
    /// room, or with none into the player's, which the eye then keeps.
    fn follow_done(&mut self) {
        let e = self.follow.eye;
        let clamp = |k: usize| {
            let r = &self.set.room_info[k];
            [0, 1, 2].map(|i| e[i].max(r.min[i]).min(r.max[i]))
        };
        let shown = match (self.room_from(e, self.follow.room), self.follow.room, self.room) {
            (Some(k), ..) => {
                self.follow.room = Some(k);
                e
            }
            (None, Some(k), _) => clamp(k),
            (None, None, Some(k)) => {
                let c = clamp(k);
                self.follow = Follow { eye: c, room: Some(k), ..self.follow };
                c
            }
            (None, None, None) => e,
        };
        self.eye = vec3(shown);
    }
}
