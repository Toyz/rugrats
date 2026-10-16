//! Effect flights (SLUS_006.50:0x8008864c, stepped by 0x8008851c ->
//! 0x80087c5c, drawn by 0x800884ac -> 0x80088388): sprites and particles
//! thrown into the world. A launch descriptor makes a record; each frame
//! the record falls by its gravity, spins, grows, fades, may bounce off
//! the world and runs out its lifetime. Kinds 3 and 5-11 are drawn as
//! camera-facing quads (0x8008afc4, 0x8008a954); 7, 9, 10 and 11 (and 3
//! and 8) show an animated sprite (hud::Sequences).
//!
//! Thrown objects are flights in the original too; the port keeps those
//! apart (world::game::things). Kinds 1, 2 and 4 (a model part,
//! 0x8006d948, 0x80087afc) are not drawn here.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::col::Col;
use rugrats_render::Vtx;

use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::ui::hud::{self, Sequences, Sprites};
use crate::world::collide::{self, Segment};

/// The flight pool (0x80085d1c: 64 records, 0x800c4d00), shared in the
/// original with thrown objects.
const POOL: usize = 64;
/// The camera-facing quads queued a frame (0x8008b088(64), 0x8008afc4).
const QUADS: usize = 64;
/// ms a frame by dt (0x800b44d4).
const FRAME_MS: u32 = 0x800b_44d4;
/// acos over -1..1 in 4097 steps (0x800b4598, read by 0x80021460 and
/// 0x80021500) and atan over 0..1 in 1025 (0x800c1710, by 0x800a0328).
const ACOS: u32 = 0x800b_4598;
const ATAN: u32 = 0x800c_1710;

/// The LCG every random draw here takes (x 214013 + 0x269ec3); a draw is
/// the new value's high half.
fn lcg(x: &mut u32) -> u32 {
    *x = x.wrapping_mul(214013).wrapping_add(0x269ec3);
    *x
}

/// Flag bits of a launch (descriptor +0x00, record +0x08).
pub mod flag {
    /// The kind: what is drawn (0x80088388) and how it is set up
    /// (0x80086114).
    pub const KIND: u32 = 0xf;
    /// Gravity (0x80087c5c): 0x100 -6, 0x200 -3, 0x400 1, 0x500 3,
    /// 0x600 6, 0x700 13, 0x800 27, else none.
    pub const GRAVITY: u32 = 0xf00;
    /// Fading (0x80087c5c): the colour's value less 19, 38, 57, 153 or 9
    /// (0x1000-0x5000) times dt / 64 a frame.
    pub const FADE: u32 = 0x7000;
    /// Collision: 0x10000 the world (0x80086668's segment), 0x20000
    /// objects, 0x80000 damage to what it hits.
    pub const WORLD: u32 = 0x1_0000;
    pub const COLLIDE: u32 = 0xf_0000;
    /// Set by 0x8008864c when the descriptor has damage.
    pub const DAMAGE: u32 = 0x8_0000;
    /// Ends on: the floor (a hit's bit 2), a wall (bit 1), any world hit
    /// (bit 8).
    pub const END_FLOOR: u32 = 0x100_0000;
    pub const END_WALL: u32 = 0x200_0000;
    pub const END_HIT: u32 = 0x2000_0000;
    /// A random spin axis (0x8008864c).
    pub const RANDOM_SPIN: u32 = 0x400_0000;
    /// Objects' collision instead (0x80086ef0).
    pub const OBJECTS: u32 = 0x1000_0000;
}

/// A launch descriptor (0x8008864c's argument), the fields the effect
/// kinds use.
#[derive(Clone, Copy, Debug, Default)]
pub struct Launch {
    /// +0x00: see [`flag`].
    pub flags: u32,
    /// +0x04: where it starts.
    pub at: [i32; 3],
    /// +0x10: its radius (no less than 0); the quad's half size for kinds
    /// 7 and 9-11.
    pub radius: i32,
    /// +0x18: the quad's roll (4096 a turn).
    pub roll: i16,
    /// +0x28: lifetime, ms (0 or less: none).
    pub life: i32,
    /// +0x2c: direction (4096 a unit) and +0x38 speed: the velocity is
    /// dir * (speed * 65536 / 6000) >> 12.
    pub dir: [i32; 3],
    pub speed: i32,
    /// +0x3c: the spin's axis (its x turns the roll) and +0x42 its rate
    /// (64 / 300 of it a dt).
    pub spin_axis: [i16; 3],
    pub spin: i16,
    /// +0x44: what a bounce keeps (4096 all).
    pub bounce: i16,
    /// +0x48: ms before it starts.
    pub delay: i32,
    /// +0x54: its colour (kinds 1, 7 and 9-11; the rest are 255).
    pub colour: [u8; 3],
    /// +0x58: its TEXPOS entry, +0x5c the quad's half size, +0x60 its
    /// face flags (hud::FACE_SEMI, hud::FACE_BLEND).
    pub sprite: Option<u32>,
    pub size: [i16; 2],
    pub face: u32,
    /// +0x6a: damage to what it hits.
    pub damage: i16,
}

/// What a record's quad shows (record +0x74).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skin {
    None,
    /// A TEXPOS entry.
    Sprite(u32),
    /// A running sequence's entry.
    Sequence(usize),
}

/// A flight record (0x8008864c).
#[derive(Clone, Debug)]
pub struct Flight {
    /// +0x08 flags, +0x0c lifetime left (ms).
    pub flags: u32,
    pub life: i32,
    /// +0x10 position, +0x1c radius, +0x24 the quad's roll.
    pub pos: [i32; 3],
    pub radius: i32,
    pub roll: i32,
    /// +0x28 bounce.
    pub bounce: i32,
    /// +0x3c velocity.
    pub v: [i32; 3],
    /// +0x4c spin axis, +0x52 spin rate.
    pub spin_axis: [i16; 3],
    pub spin: i32,
    /// +0x6c colour, +0x70 face flags, +0x74 texture, +0x78 half size.
    pub colour: [u8; 3],
    pub face: u32,
    pub skin: Skin,
    pub size: [i16; 2],
    /// +0x7c the sequence it holds (let go when it ends), +0x90 how fast
    /// its size grows, +0x94 ms before it starts.
    pub sequence: Option<usize>,
    pub grow: i32,
    pub delay: i32,
}

impl Flight {
    pub fn kind(&self) -> u32 {
        self.flags & flag::KIND
    }
}

/// The game's fixed-point angle tables.
#[derive(Clone, Debug, Default)]
pub struct Angles {
    acos: Vec<u16>,
    atan: Vec<i16>,
}

impl Angles {
    pub fn load(disc: &GameDisc) -> Result<Angles, String> {
        let halves = |at: u32, n: usize| -> Result<Vec<u16>, String> {
            Ok(disc.exe_bytes(at, 2 * n)?.as_chunks::<2>().0.iter().map(|h| u16::from_le_bytes(*h)).collect())
        };
        Ok(Angles { acos: halves(ACOS, 4097)?, atan: halves(ATAN, 1025)?.into_iter().map(|h| h as i16).collect() })
    }

    fn acos_at(&self, k: i32) -> i32 {
        self.acos.get(k as usize).copied().unwrap_or(0) as i32
    }

    /// asin of `a` (4096 = 1) in 4096 a turn (0x80021460).
    pub fn asin(&self, a: i32) -> i32 {
        let a = (a as i16 as i32).clamp(-4096, 4096);
        let r = match a {
            a if a < 0 => self.acos_at(a + 4096) - 1024,
            0 => 0,
            a => 1024 - self.acos_at(4096 - a),
        };
        r as i16 as i32
    }

    /// acos (0x80021500).
    pub fn acos(&self, a: i32) -> i32 {
        let a = (a as i16 as i32).clamp(-4096, 4096);
        let r = match a {
            a if a < 0 => 2048 - self.acos_at(a + 4096),
            0 => 1024,
            a => self.acos_at(4096 - a) as i16 as i32,
        };
        r as i16 as i32
    }

    /// atan2(a, b) in 4096 a turn (ratan2, 0x800a0328): the angle of the
    /// point (b, a) from +b toward +a, by a 1025-entry table; 0 for (0, 0).
    pub fn ratan2(&self, a: i32, b: i32) -> i32 {
        let (neg_b, b) = (b < 0, b.wrapping_abs());
        let (neg_a, a) = (a < 0, a.wrapping_abs());
        if a == 0 && b == 0 {
            return 0;
        }
        let t = |k: i32| self.atan.get(k as usize).copied().unwrap_or(0) as i32;
        let mut r = if a < b {
            let q = if a & 0x7fe0_0000 != 0 { a.wrapping_div(b >> 10) } else { (a << 10).wrapping_div(b) };
            t(q)
        } else {
            let q = if b & 0x7fe0_0000 != 0 { b.wrapping_div(a >> 10) } else { (b << 10).wrapping_div(a) };
            1024 - t(q)
        };
        if neg_b {
            r = 2048 - r;
        }
        if neg_a { -r } else { r }
    }
}

/// A 3 x 3 matrix as the GTE takes it, 4096 = 1, rows.
pub type Matrix = [[i32; 3]; 3];

pub const IDENTITY: Matrix = [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]];

/// a b (MulMatrix0, 0x8009f488): each element's sum >> 12, kept to 16
/// bits.
fn mul(a: &Matrix, b: &Matrix) -> Matrix {
    let mut m = [[0; 3]; 3];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, e) in row.iter_mut().enumerate() {
            let sum: i64 = (0..3).map(|k| a[r][k] as i64 * b[k][c] as i64).sum();
            *e = ((sum >> 12) as i32).clamp(-32768, 32767);
        }
    }
    m
}

/// m v >> 12 (ApplyMatrixLV, 0x8009f724, for v under 32768 a component).
fn apply(m: &Matrix, v: [i32; 3]) -> [i32; 3] {
    [0, 1, 2].map(|r| ((0..3).map(|k| m[r][k] as i64 * v[k] as i64).sum::<i64>() >> 12) as i32)
}

fn transpose(m: &Matrix) -> Matrix {
    [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]))
}

fn rot_x(trig: &Trig, a: i32) -> Matrix {
    let (s, c) = (trig.sin(a), trig.cos(a));
    [[4096, 0, 0], [0, c, -s], [0, s, c]]
}

fn rot_y(trig: &Trig, a: i32) -> Matrix {
    let (s, c) = (trig.sin(a), trig.cos(a));
    [[c, 0, s], [0, 4096, 0], [-s, 0, c]]
}

fn rot_z(trig: &Trig, a: i32) -> Matrix {
    let (s, c) = (trig.sin(a), trig.cos(a));
    [[c, -s, 0], [s, c, 0], [0, 0, 4096]]
}

/// The random draws (the executable's initial seeds): directions and
/// random spin axes (0x800c445c), debris (0x800c419c), a pie's splat
/// (0x800c41a4).
#[derive(Clone, Copy, Debug)]
pub struct Seeds {
    pub direction: u32,
    pub debris: u32,
    pub pie: u32,
}

impl Default for Seeds {
    fn default() -> Seeds {
        Seeds { direction: 1_092_843, debris: 48732, pie: 48732 }
    }
}

/// The flights in the air, their animated sprites and the draws they take.
#[derive(Clone, Debug)]
pub struct Flights {
    /// Flying (list 0x800c4ce8) and waiting out a delay (0x800c4cf8); each
    /// list is newest first, kept here newest last.
    flying: Vec<Flight>,
    waiting: Vec<Flight>,
    pub sequences: Sequences,
    pub seeds: Seeds,
    pub angles: Angles,
    trig: Trig,
    frame_ms: Vec<i32>,
}

/// The broken buildings' debris sprites (0x8005202c, by rand mod 10).
const DEBRIS: [u32; 10] = [392, 393, 394, 395, 396, 397, 399, 401, 403, 405];

/// Reptar's flame (0x8005cd74): flags 0x8030b (kind 11: sequence 0x1017,
/// no gravity, damage), radius 750 (0x800c41e0), 1500 ms (0x800c41e4),
/// speed 70 (0x800c41e8), grey 128 (0x800c42d0), damage 1000.
pub const FLAME: Launch = Launch {
    flags: 0x8030b,
    at: [0; 3],
    radius: 750,
    roll: 0,
    life: 1500,
    dir: [0; 3],
    speed: 70,
    spin_axis: [0; 3],
    spin: 0,
    bounce: 0,
    delay: 0,
    colour: [128; 3],
    sprite: None,
    size: [0; 2],
    face: 0,
    damage: 1000,
};

impl Flights {
    pub fn load(disc: &GameDisc) -> Result<Flights, String> {
        let ms = disc.exe_bytes(FRAME_MS, 4 * 64)?.as_chunks::<4>().0.iter().map(|w| i32::from_le_bytes(*w)).collect();
        Ok(Flights {
            flying: Vec::new(),
            waiting: Vec::new(),
            sequences: Sequences::load(disc)?,
            seeds: Seeds::default(),
            angles: Angles::load(disc)?,
            trig: Trig::load(disc)?,
            frame_ms: ms,
        })
    }

    /// ms in a frame of `dt` (0x800b44d4[dt]).
    pub fn ms(&self, dt: i32) -> i32 {
        self.frame_ms.get(dt as usize).copied().unwrap_or(0)
    }

    /// The flights in the air, newest first.
    pub fn iter(&self) -> impl Iterator<Item = &Flight> {
        self.flying.iter().rev()
    }

    /// How many are flying or waiting.
    pub fn len(&self) -> usize {
        self.flying.len() + self.waiting.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A direction no more than `max` (4096 a turn) off `axis` and no less
    /// than `min`, spun at random about it, then turned by `frame`
    /// (0x80088bac -> 0x80085f44). Two draws from the direction seed.
    pub fn cone(&mut self, axis: [i32; 3], frame: &Matrix, min: i16, max: i16) -> [i32; 3] {
        let trig = &self.trig;
        let n = if max > 0 { max as i32 } else { 1 };
        let tilt = ((lcg(&mut self.seeds.direction) >> 16) & 4095) as i32 % n;
        let tilt = if tilt < min as i32 { min as i32 } else { tilt };
        let spin = ((lcg(&mut self.seeds.direction) >> 16) & 4095) as i32;
        // 0x80085b9c: Rx(tilt) Rz(spin), loaded transposed (0x80010bb4).
        let m = mul(&rot_x(trig, tilt), &mul(&rot_z(trig, spin), &IDENTITY));
        let v = apply(&transpose(&m), [0, 0, 4096]);
        // The axis's angles (0x800760f4) as 0x80024424 turns by them:
        // Ry(yaw) Rx(pitch) Rz(0).
        let yaw = self.angles.ratan2(axis[0], axis[2]);
        let pitch = -self.angles.asin(axis[1]) as i16 as i32;
        let m = mul(&rot_y(trig, yaw), &mul(&rot_x(trig, pitch), &mul(&rot_z(trig, 0), &IDENTITY)));
        let v = apply(&m, v);
        apply(&transpose(frame), v)
    }

    /// Makes a flight from `l` (0x8008864c, then 0x80086114 by kind).
    /// Returns false when the pool is full.
    ///
    /// Not here: the room lookup (0x8002b660: no room, no flight) and the
    /// start kept inside its room's box; the callback (+0x4c), sound
    /// (+0x64) and the model (+0x50) of other kinds.
    pub fn launch(&mut self, l: &Launch, sprites: Option<&Sprites>) -> bool {
        if self.len() >= POOL {
            return false;
        }
        let mut flags = l.flags;
        if l.damage != 0 {
            flags |= flag::DAMAGE;
        }
        let speed = (l.speed << 16).wrapping_div(6000);
        let v = l.dir.map(|d| d.wrapping_mul(speed) >> 12);
        let spin_axis = if flags & flag::RANDOM_SPIN != 0 {
            [0; 3].map(|_| (((lcg(&mut self.seeds.direction) >> 16) & 0x1fff) as i32 - 4096) as i16)
        } else {
            l.spin_axis
        };
        let mut f = Flight {
            flags,
            life: l.life,
            pos: l.at,
            radius: l.radius.max(0),
            roll: l.roll as i32,
            bounce: l.bounce as i32,
            v,
            spin_axis,
            spin: ((l.spin as i32) << 6) / 300,
            colour: [255; 3],
            face: l.face,
            skin: l.sprite.map_or(Skin::None, Skin::Sprite),
            size: l.size,
            sequence: None,
            grow: 0,
            delay: l.delay,
        };
        // Kinds 3 and 8 share one running sequence (0x8008a2a4); 0x80086114
        // then forgets it, so it is never let go.
        let kind = flags & flag::KIND;
        if let Some(id) = match kind {
            3 => Some(0x101b),
            8 => Some(0x101d),
            _ => None,
        } && let Some(slot) = self.sequences.start_shared(id, sprites)
        {
            f.skin = Skin::Sequence(slot);
            self.sequences.activate(slot);
        }
        // 0x80086114.
        let sequence = match kind {
            7 => Some((0x1016, 0xd2a0_0000)),
            9 => Some((0x1018, 0xc2a0_0000)),
            10 => Some((0x1021, 0xd2a0_0000)),
            11 => Some((0x1017, 0xd2a0_0000)),
            _ => None,
        };
        match kind {
            1 => f.colour = l.colour,
            6 => {
                f.face = 0xc080_0000;
                f.size = [225, 225];
                f.skin = Skin::Sprite(298);
            }
            _ => {}
        }
        if let Some((id, face)) = sequence {
            if let Some(slot) = self.sequences.start(id, sprites) {
                f.skin = Skin::Sequence(slot);
                f.sequence = Some(slot);
                self.sequences.activate(slot);
                f.size = [f.radius as i16; 2];
            }
            f.face = face;
            f.colour = l.colour;
            if kind == 11 {
                // 0x800c41ec << 6.
                f.grow = 4 << 6;
            }
        }
        if f.delay > 0 {
            self.waiting.push(f);
        } else {
            self.flying.push(f);
        }
        true
    }

    /// One frame of `dt` (0x8008851c): the waiting count down and join the
    /// flying; each flying one moves (0x80087c5c) and those done are let
    /// go; then the sequences step (0x8008a358).
    pub fn step(&mut self, col: &Col, sprites: Option<&Sprites>, dt: i32) {
        let ms = self.ms(dt);
        // 0x80088310: the list head first, each started one to the front
        // of the flying.
        let mut k = self.waiting.len();
        while k > 0 {
            k -= 1;
            self.waiting[k].delay -= ms;
            if self.waiting[k].delay <= 0 {
                let f = self.waiting.remove(k);
                self.flying.push(f);
            }
        }
        let mut done = Vec::new();
        for (k, f) in self.flying.iter_mut().enumerate() {
            if step_one(f, col, &self.angles, ms, dt) {
                done.push(k);
            }
        }
        for k in done.into_iter().rev() {
            let f = self.flying.remove(k);
            if let Some(slot) = f.sequence {
                self.sequences.release(slot);
            }
        }
        self.sequences.step(ms, sprites);
    }

    /// The quads of kinds 3 and 5-11 (0x80088388 -> 0x8008afc4, drawn by
    /// 0x8008a954), the newest first and no more than the queue holds: a
    /// quad of half size (w, h) facing the camera, turned by its roll,
    /// none behind the eye or 65536 or more ahead. `view` is the eye and
    /// the world-to-camera rows.
    ///
    /// Not here: the screen-edge cull (0x8008a954 draws a quad only when
    /// one of its first three corners is near the screen) and the split
    /// into four quads with face flag 0x8000000.
    pub fn draw(&self, view: (Vec3, Mat3), sprites: Option<&Sprites>, out: &mut Vec<Vtx>) {
        let (eye, rows) = view;
        let (right, down, ahead) = (rows.row(0), rows.row(1), rows.row(2));
        let quads = self.iter().filter(|f| matches!(f.kind(), 3 | 5..=11)).take(QUADS);
        for f in quads {
            let at = Vec3::new(f.pos[0] as f32, f.pos[1] as f32, f.pos[2] as f32);
            let z = ahead.dot(at - eye);
            if !(0.0..65536.0).contains(&z) {
                continue;
            }
            let texture = match f.skin {
                Skin::Sprite(n) => sprites.and_then(|s| s.entry(n)),
                Skin::Sequence(slot) => self.sequences.texture(slot),
                Skin::None => None,
            };
            let Some(t) = texture else { continue };
            let (s, c) = (self.trig.sin(f.roll) as f32 / 4096.0, self.trig.cos(f.roll) as f32 / 4096.0);
            let (w, h) = (f.size[0] as f32, f.size[1] as f32);
            let u = (right * c + down * s) * w;
            let v = (down * c - right * s) * h;
            hud::face(t, at, (u, v), f.colour, f.face, out);
        }
    }

    /// Reptar's flame from `at` along `dir` (0x8005cd74).
    pub fn flame(&mut self, at: [i32; 3], dir: [i32; 3], sprites: Option<&Sprites>) -> bool {
        self.launch(&Launch { at, dir, ..FLAME }, sprites)
    }

    /// An explosion at `at` (0x80053d28): kind 7 (sequence 0x1016), radius
    /// 3000, 5000 ms, standing still; its direction is drawn (0x80088bac,
    /// 120 about +z) though its speed is 0.
    pub fn explosion(&mut self, at: [i32; 3], sprites: Option<&Sprites>) -> bool {
        let dir = self.cone([0, 0, 4096], &IDENTITY, 0, 120);
        let l = Launch { flags: 7, at, radius: 3000, life: 5000, dir, colour: [255; 3], ..Launch::default() };
        self.launch(&l, sprites)
    }

    /// A smaller burst (0x80053e10): kind 9 (sequence 0x1018), radius 1500,
    /// 3000 ms.
    pub fn burst(&mut self, at: [i32; 3], sprites: Option<&Sprites>) -> bool {
        let dir = self.cone([0, 0, 4096], &IDENTITY, 0, 120);
        let l = Launch { flags: 9, at, radius: 1500, life: 3000, dir, colour: [255; 3], ..Launch::default() };
        self.launch(&l, sprites)
    }

    /// A broken building's debris (0x8005202c, its touch): ten bits of
    /// rubble (sprites 392-405) thrown up within half a turn at speed
    /// rand mod 200 (100 at most), falling at 13, bouncing off the world
    /// by 2048 / 4096, 1000 ms each. `damage` is the object's +0x6d.
    pub fn debris(&mut self, at: [i32; 3], damage: i16, sprites: Option<&Sprites>) {
        for _ in 0..10 {
            let sprite = DEBRIS[((lcg(&mut self.seeds.debris) >> 16) & 0x7fff) as usize % 10];
            let speed = ((((lcg(&mut self.seeds.debris) >> 16) & 0x7fff) % 200) as i32).min(100);
            let dir = self.cone([0, -4096, 0], &IDENTITY, 0, 2048);
            let l = Launch {
                flags: 0x30705,
                at,
                radius: 200,
                life: 1000,
                dir,
                speed,
                spin: 3391,
                bounce: 2048,
                sprite: Some(sprite),
                size: [200, 200],
                face: 0xc080_0000,
                damage,
                ..Launch::default()
            };
            self.launch(&l, sprites);
        }
    }

    /// A pie's splat (0x8005282c, the pie's flight end): five bits of
    /// sprite 359 thrown within half a turn of the surface's `normal` at
    /// speed rand mod 150 (50 at most), falling at 13, bouncing off the
    /// world by 1000 / 4096, 2000 ms each; opaque (face 0xd0800000).
    /// A collectible's three sparks as it is taken (0x800514b0): kind 5
    /// (flags 0x305), half-size 200, face 0xd2800000, 500 ms, bounce 2048,
    /// speed rand mod 100 from its own seed 0xb0a5 (set afresh each time),
    /// all round about up (0x80088bac(up, 0, 4096)). The sprite by
    /// template: 835 TEXPOS + 3552 (296), 863 and 1017 359, 87 362, else
    /// TEXPOS + 3540 (295). `damage` is its +0x6d.
    pub fn sparkle(&mut self, template: u16, at: [i32; 3], damage: i16, sprites: Option<&Sprites>) {
        let sprite = match template {
            835 => 296,
            863 | 1017 => 359,
            87 => 362,
            _ => 295,
        };
        let mut seed = 0xb0a5u32;
        for _ in 0..3 {
            let speed = (((lcg(&mut seed) >> 16) & 0x7fff) % 100) as i32;
            let dir = self.cone([0, 4096, 0], &IDENTITY, 0, 4096);
            let l = Launch {
                flags: 0x305,
                at,
                life: 500,
                dir,
                speed,
                bounce: 2048,
                sprite: Some(sprite),
                size: [200, 200],
                face: 0xd280_0000,
                damage,
                ..Launch::default()
            };
            self.launch(&l, sprites);
        }
    }

    pub fn splat(&mut self, normal: [i32; 3], at: [i32; 3], sprites: Option<&Sprites>) {
        for _ in 0..5 {
            let speed = ((((lcg(&mut self.seeds.pie) >> 16) & 0x7fff) % 150) as i32).min(50);
            let dir = self.cone(normal, &IDENTITY, 0, 2048);
            let l = Launch {
                flags: 0x30705,
                at,
                radius: 200,
                life: 2000,
                dir,
                speed,
                spin: 3391,
                bounce: 1000,
                sprite: Some(359),
                size: [200, 200],
                face: 0xd080_0000,
                ..Launch::default()
            };
            self.launch(&l, sprites);
        }
    }
}

/// Gravity by flags & 0xf00 (0x80087c5c).
fn gravity(flags: u32) -> i32 {
    match flags & flag::GRAVITY {
        0x100 => -6,
        0x200 => -3,
        0x400 => 1,
        0x500 => 3,
        0x600 => 6,
        0x700 => 13,
        0x800 => 27,
        _ => 0,
    }
}

/// How fast the colour fades by flags & 0x7000, a dt (0x80087c5c).
fn fade_rate(flags: u32) -> i32 {
    match flags & flag::FADE {
        0x1000 => 19,
        0x2000 => 38,
        0x3000 => 57,
        0x4000 => 153,
        0x5000 => 9,
        _ => 0,
    }
}

/// One record's frame (0x80087c5c); true when it is done.
fn step_one(f: &mut Flight, col: &Col, angles: &Angles, ms: i32, dt: i32) -> bool {
    let mut done = false;
    if f.grow != 0 {
        let s = (f.grow * dt) >> 6;
        f.size = f.size.map(|w| (w as i32 + s) as i16);
        f.radius = f.size[0].max(f.size[1]) as i32;
    }
    // 0x8004fbb4: pos += ((a dt dt) >> 1 + v dt) >> 6; v += a dt.
    let before = f.pos;
    let g = gravity(f.flags);
    for (k, (p, v)) in f.pos.iter_mut().zip(f.v.iter_mut()).enumerate() {
        let ad = if k == 1 { g * dt } else { 0 };
        *p += (((ad * dt) >> 1) + *v * dt) >> 6;
        *v += ad;
    }
    let s = (f.spin * dt) >> 6;
    if s != 0 {
        f.roll = (f.roll + ((s * f.spin_axis[0] as i32) >> 12)) & 4095;
    }
    if f.flags & flag::COLLIDE != 0 && f.flags & flag::OBJECTS == 0 {
        let hit = collide_world(f, col, angles, before);
        let ends = (f.flags & flag::END_FLOOR != 0 && hit & 2 != 0)
            || (f.flags & flag::END_WALL != 0 && hit & 1 != 0)
            || (f.flags & flag::END_HIT != 0 && hit & 8 != 0);
        done |= ends;
    }
    let fade = (fade_rate(f.flags) * dt) >> 6;
    if fade != 0 {
        let (h, s, v) = rgb_to_hsv(f.colour);
        if v != 0 {
            if v < fade {
                f.colour = [0; 3];
            } else {
                f.colour = hsv_to_rgb(h, s, v - fade);
            }
        }
    }
    if f.life > 0 {
        f.life -= ms;
        if f.life <= 0 {
            done = true;
        }
    }
    done
}

/// The world part of 0x80087594 (with 0x80086668's segment, 0x8002cd7c):
/// the step from `before` to the new position, carried on by the radius
/// (10 at least) along the velocity. A hit (bits 1 and 8) bounces when
/// it meets the surface head on (0x8007666c: the angle over 1024): the
/// flight backs off the radius from the hit, and stops (bit 2) slower
/// than 95 + 382 with a floor within its radius under it (0x80087414),
/// else goes back along the reflection at (|v| - 95) * bounce >> 12 and
/// its spin keeps bounce / 4096. Returns the hit bits.
///
/// Not here: objects (flag 0x20000, 0x80086340), what the hit calls
/// (+0x80, +0x88's sound, +0x2a's floor effect).
fn collide_world(f: &mut Flight, col: &Col, angles: &Angles, before: [i32; 3]) -> u32 {
    if f.flags & flag::WORLD == 0 {
        return 0;
    }
    let reach = f.radius.max(10);
    let u = collide::unit(f.v);
    let to = [0, 1, 2].map(|k| f.pos[k] + ((u[k] * reach) >> 12));
    let mut hit = Segment::default();
    if collide::segment(col, before, to, &mut hit) == 0 {
        return 0;
    }
    let mut bits = 1 | 8;
    let (c, r) = reflect(angles, f.v, hit.normal);
    if c < 1025 {
        return bits & !1;
    }
    let back = [0, 1, 2].map(|k| hit.point[k] - ((u[k] * reach) >> 12));
    let speed = collide::length(f.v) - 95;
    let below = [back[0], back[1] + f.radius, back[2]];
    let floor = collide::segment(col, back, below, &mut Segment::default()) != 0;
    if floor && speed < 382 {
        f.v = [0; 3];
        bits |= 2;
    } else {
        let s = (speed * f.bounce) >> 12;
        f.v = r.map(|r| (r * s) >> 12);
    }
    f.spin = (((f.spin * f.bounce) >> 12) as i16) as i32;
    f.pos = back;
    bits
}

/// The reflection of `v` off a surface of normal `n` (0x8007666c): v made
/// unit, d = n . v; when d < 0, v - 2 d n. Returns acos d and it.
fn reflect(angles: &Angles, v: [i32; 3], n: [i32; 3]) -> (i32, [i32; 3]) {
    let u = collide::unit(v);
    let d = ((n[0] * u[0]) >> 12) + ((n[1] * u[1]) >> 12) + ((n[2] * u[2]) >> 12);
    let r = if d < 0 { [0, 1, 2].map(|k| u[k] - ((d * n[k]) >> 11)) } else { u };
    (angles.acos(d), r)
}

/// RGB to hue (6144 a turn), saturation and value (0x8001f238).
pub fn rgb_to_hsv([r, g, b]: [u8; 3]) -> (i32, i32, i32) {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max == 0 {
        return (0, 0, 0);
    }
    let d = max - min;
    if d == 0 {
        return (0, 0, max);
    }
    let (base, num) = if r == max {
        (0, g - b)
    } else if g == max {
        (2048, b - r)
    } else {
        (4096, r - g)
    };
    let mut h = base + (num << 10) / d;
    if h < 0 {
        h += 6144;
    }
    let s = ((d << 8) - 1) / max;
    (h, s, max)
}

/// Hue, saturation and value back to RGB (0x8001f360).
pub fn hsv_to_rgb(h: i32, s: i32, v: i32) -> [u8; 3] {
    if v == 0 {
        return [0; 3];
    }
    if s == 0 {
        return [v as u8; 3];
    }
    let (i, f) = (h >> 10, h & 1023);
    let p = (v * (255 - s) + 255) >> 8;
    let q = (v * (255 - ((s * f) >> 10)) + 127) >> 8;
    let t = (v * (255 - ((s * (1024 - f)) >> 10)) + 127) >> 8;
    let rgb = match i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => [0, 0, 0],
    };
    rgb.map(|c| c as u8)
}

/// Where Reptar's flame starts and goes, from the head's part (0x8005ce74
/// on bone slot 16): its matrix as the game keeps it (world to part, the
/// transpose of `rot`, 4096 = 1) and joint `at`. The start is the joint
/// plus (0, -1400, 512) (0x800c41f0) turned into the world; the direction
/// is the part's yaw and pitch (0x800210b0) made a unit vector again
/// (0x80076040).
pub fn flame_from(angles: &Angles, trig: &Trig, rot: Mat3, at: Vec3) -> ([i32; 3], [i32; 3]) {
    // The part's matrix to the world, rows, 4096 = 1.
    let m: Matrix = [0, 1, 2].map(|r| [0, 1, 2].map(|c| (rot.col(c)[r] * 4096.0).round() as i32));
    let off = apply(&m, [0, -1400, 512]);
    let start = [0, 1, 2].map(|k| at[k].round() as i32 + off[k]);
    // 0x800210b0 on the transposed (world) matrix.
    let pitch = -angles.asin(m[1][2]) as i16 as i32;
    let yaw = angles.ratan2(-m[2][0], m[2][2]);
    // 0x80076040.
    let (sy, cy, sp, cp) = (trig.sin(yaw), trig.cos(yaw), trig.sin(pitch), trig.cos(pitch));
    let v = [((sy * cp) << 4) >> 16, -sp as i16 as i32, ((cy * cp) << 4) >> 16];
    (start, collide::unit(v))
}
