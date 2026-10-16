//! The opening's ending, its one timed call (0x8008ea70, frames 1184 to
//! 1413 of cine 0): Tommy's milk flies at the lens, then the Rugrats sign
//! with "Search For Reptar" and "Loading" (docs/engine/cinematics.md).
//!
//! Two cameras of their own, both unturned, draw over the scene:
//!
//! - Camera A (0x800e54d8) starts at (2250, -2450, -5000) and moves by
//!   (-75, 81, 160) a frame to frame 1213, toward the milk (model 948,
//!   0x8008f078) at the origin, which turns by 273 a frame. The milk first
//!   shows small at the bottle's tip and then fills the screen.
//! - Once A's x is 0 or less, camera B (0x800e5588, at (0, 30, z)) brings
//!   the sign (model 1038) in: z from -17048 by 600 a frame to -500, then
//!   A's z is -300 and B's field of view 512 sets H 386 for both. From
//!   then on the two lines are drawn.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::model::Model;
use rugrats_render::Vtx;
use rugrats_render::models::model_triangles;

use crate::gte::Trig;
use crate::ui::text::{Fonts, Text};

/// The cinematic this belongs to, and its window.
pub const CINE: usize = 0;
const FIRST: u16 = 1184;
const LAST: u16 = 1413;

const MILK: u32 = 948;
const SIGN: u32 = 1038;
/// The cinematic's projection, and the one camera B's field of view of
/// 512 sets for everything after (SetGeomScreen, 0x8001dfb8).
const H_CINE: f32 = 178.0;
const H_SIGN: f32 = 386.0;
/// The sign's text: colour, and the lines' y (shadows 2 down and right).
const GREEN: [u8; 3] = [100, 200, 0];
const SHADOW: [u8; 3] = [1, 1, 1];

pub struct Ending {
    /// Camera A's position; the milk still moving (its +0xd0).
    a: [i32; 3],
    flying: bool,
    milk_turn: i32,
    /// Camera B's z, and the projection both draw with.
    b_z: i32,
    h: f32,
    text: bool,
}

impl Ending {
    /// The opening's init (0x8008e8c0(1), 0x8008e99c(1)).
    pub fn new() -> Ending {
        Ending { a: [2250, -2450, -5000], flying: true, milk_turn: 0, b_z: -17048, h: H_CINE, text: false }
    }

    /// The timed call for stream frame `n`.
    pub fn step(&mut self, n: u16) {
        if !(FIRST..=LAST).contains(&n) {
            return;
        }
        if self.flying {
            let k = n.min(1213) as i32 - 1183;
            self.a = [2250 - 75 * k, -2450 + 81 * k, -5000 + 160 * k];
            self.milk_turn = 273 * k;
        }
        if self.a[0] > 0 {
            return;
        }
        self.flying = false;
        if self.b_z < -500 {
            self.b_z += 600;
            if self.b_z >= -499 {
                self.b_z = -500;
                self.a[2] = -300;
                self.h = H_SIGN;
            }
        } else {
            self.text = true;
        }
    }

    /// Whether anything shows yet.
    pub fn started(n: Option<u16>) -> bool {
        n.is_some_and(|n| n >= FIRST)
    }

    /// The milk, the sign once B moves, and the text, into the overlay.
    pub fn draw(&self, models: &rugrats_data::model::Models, trig: &Trig, fonts: &Fonts, out: &mut Vec<Vtx>) {
        if let Some(m) = models.get(MILK) {
            let rows = turned(trig, [0, 0, self.milk_turn as i16]);
            project(m, rows, Vec3::ZERO, self.a, self.h, out);
        }
        if !self.flying
            && let Some(m) = models.get(SIGN)
        {
            project(m, Mat3::IDENTITY, Vec3::ZERO, [0, 30, self.b_z], self.h, out);
        }
        if self.text {
            // The trademark by the sign (0x8003f0f0).
            crate::ui::front::trademark(out);
            for (font, y, s) in [(1, 60, "Search For Reptar"), (0, 85, "Loading")] {
                for (rgb, d) in [(SHADOW, 2), (GREEN, 0)] {
                    let mut t = Text { font, spacing: 1, rgb, x: d, y: y + d, string: s.into(), ..Default::default() };
                    fonts.centre(&mut t, true, false);
                    t.triangles(fonts, out);
                }
            }
        }
    }
}

impl Default for Ending {
    fn default() -> Ending {
        Ending::new()
    }
}

/// An object's matrix from its angles (+0x24), as the button cubes turn.
fn turned(trig: &Trig, angles: [i16; 3]) -> Mat3 {
    let e = trig.euler(angles);
    Mat3::from_cols_array(&e.concat().try_into().unwrap_or([4096; 9]).map(|v: i32| v as f32 / 4096.0)).transpose()
}

/// Model `m` turned by `rows` at `pos`, seen from an unturned camera at
/// `eye` with projection `h` (x by 512 / 320), as screen triangles drawn
/// far first (the ordering table) for the overlay.
fn project(m: &Model, rows: Mat3, pos: Vec3, eye: [i32; 3], h: f32, out: &mut Vec<Vtx>) {
    let mut world = Vec::new();
    model_triangles(m, rows, pos, &mut world);
    let eye = Vec3::new(eye[0] as f32, eye[1] as f32, eye[2] as f32);
    let mut tris: Vec<(f32, [Vtx; 3])> = world
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|t| {
            let mut z = 0.0;
            let mut tri = [t[0], t[1], t[2]];
            for v in &mut tri {
                let p = Vec3::from_array(v.pos) - eye;
                if p.z < 16.0 {
                    return None;
                }
                z += p.z;
                v.pos = [256.0 + 512.0 / 320.0 * h * p.x / p.z, 120.0 + h * p.y / p.z, 0.0];
            }
            Some((z, tri))
        })
        .collect();
    tris.sort_by(|a, b| b.0.total_cmp(&a.0));
    out.extend(tris.into_iter().flat_map(|(_, t)| t));
}
