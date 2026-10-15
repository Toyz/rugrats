//! Mini-golf's setup (0x80096134, drawn by 0x80095c2c steps 0-1;
//! docs/levels/golf.md "The setup loop"): how many players, then each
//! player picks a face from the six heads.

use rrt::glam::{Mat3, Vec3};
use rugrats_render::Vtx;

use crate::input::{self, Pads};
use crate::ui::text::{Fonts, Text};

/// The front end's sounds (0x8006f26c): leave, accept, the two moves.
const LEAVE: u8 = 1;
const ACCEPT: u8 = 2;
const MOVE_A: u8 = 3;
const MOVE_B: u8 = 4;
/// Colours: the title red, the question yellow, the count lit and not.
const RED: [u8; 3] = [0xe0, 0, 0];
const YELLOW: [u8; 3] = [255, 255, 0];
const LIT: [u8; 3] = [0xf0, 0xf0, 0xf0];
const DIM: [u8; 3] = [0x40, 0x40, 0x40];
/// The cursor (TEXPOS 373, 31 x 31 below the face) and a pick's star
/// (427, from (x - 68, y - 56) to (x + 69, y + 18)).
const CURSOR: u32 = 373;
const STAR: u32 = 427;
/// The faces' depth on this screen (0x80094e90).
const DEPTH: f32 = 5500.0;
const HEAD_SCALE: f32 = 0.54;

/// What the setup ends with.
pub enum Outcome {
    /// Triangle at the count: back to the Activities menu.
    Back,
    /// Every player picked: their face slots.
    Play(Vec<usize>),
}

pub struct GolfSetup {
    step: u8,
    count: usize,
    picks: Vec<usize>,
    taken: u32,
    col: usize,
    row: usize,
    frames: u32,
    strings: Vec<String>,
    pub sounds: Vec<u8>,
}

fn face_at(slot: usize) -> (i16, i16) {
    (-120 + 130 * (slot % 3) as i16, 25 + 60 * (slot / 3) as i16)
}

impl GolfSetup {
    /// Its strings: "Mini-Golf", "How Many Players?", "One player", "%s
    /// players", "Two", "Three", "Four", "Player %d, pick your
    /// character!" (87, 86, 199, 200, 120-122, 201).
    pub fn with_strings(strings: Vec<String>, count: usize) -> GolfSetup {
        GolfSetup {
            step: 0,
            count: count.clamp(1, 4),
            picks: Vec::new(),
            taken: 0,
            col: 0,
            row: 0,
            frames: 0,
            strings,
            sounds: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn frame(&mut self, pads: &Pads) -> Option<Outcome> {
        self.sounds.clear();
        self.frames += 1;
        let pressed = |b| pads.pressed(0, b);
        if self.step == 0 {
            if pressed(input::TRIANGLE) {
                self.sounds.push(LEAVE);
                return Some(Outcome::Back);
            }
            if pressed(input::LEFT) {
                self.count = (self.count - 1).max(1);
                self.sounds.push(MOVE_A);
            } else if pressed(input::RIGHT) {
                self.count = (self.count + 1).min(4);
                self.sounds.push(MOVE_B);
            }
            if pressed(input::CROSS) {
                self.sounds.push(ACCEPT);
                self.col = 0;
                self.step = 1;
                self.skip_taken();
            }
            return None;
        }
        if pressed(input::TRIANGLE) {
            // Every pick undone, back to the count; no sound.
            self.picks.clear();
            self.taken = 0;
            self.step = 0;
            return None;
        }
        if pressed(input::LEFT) {
            self.col = self.col.saturating_sub(1);
            self.sounds.push(MOVE_A);
        } else if pressed(input::RIGHT) {
            self.col = (self.col + 1).min(2);
            self.sounds.push(MOVE_B);
        } else if pressed(input::UP) {
            self.row = self.row.saturating_sub(1);
            self.sounds.push(MOVE_A);
        } else if pressed(input::DOWN) {
            self.row = (self.row + 1).min(1);
            self.sounds.push(MOVE_B);
        }
        if pressed(input::CROSS) {
            let slot = self.col + 3 * self.row;
            self.picks.push(slot);
            self.taken |= 1 << slot;
            self.sounds.push(ACCEPT);
            if self.picks.len() == self.count {
                return Some(Outcome::Play(self.picks.clone()));
            }
        }
        self.skip_taken();
        None
    }

    /// 0x800960a0: the cursor moves on past taken faces, wrapping to
    /// (0, 0).
    fn skip_taken(&mut self) {
        let mut tries = 0;
        while self.taken >> (self.col + 3 * self.row) & 1 != 0 && tries < 6 {
            self.col += 1;
            if self.col == 3 {
                self.col = 0;
                self.row += 1;
                if self.row == 2 {
                    self.row = 0;
                    break;
                }
            }
            tries += 1;
        }
    }

    /// The screen's text, faces, cursor and stars. `camera` is the
    /// title's eye and `focal` its projection.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        fonts: &Fonts,
        heads: &[Option<rugrats_data::model::Model>],
        sprites: Option<&crate::ui::hud::Sprites>,
        camera: Vec3,
        focal: f32,
        under: &mut Vec<Vtx>,
        moving: &mut Vec<Vtx>,
        overlay: &mut Vec<Vtx>,
    ) {
        let put = |s: &str, x: i16, y: i16, font: u8, rgb: [u8; 3], centre: bool, shadow: bool, out: &mut Vec<Vtx>| {
            let mut t = Text { font, spacing: 1, rgb, x, y, string: s.to_string(), ..Default::default() };
            if centre {
                fonts.centre(&mut t, true, false);
            }
            if shadow {
                let mut sh = t.clone();
                sh.rgb = [1, 1, 1];
                sh.x += 2;
                sh.y += 2;
                sh.triangles(fonts, out);
            }
            t.triangles(fonts, out);
        };
        put(&self.strings[0], 0, -100, 1, RED, true, true, overlay);
        if self.step == 0 {
            put(&self.strings[1], 0, -65, 1, YELLOW, true, true, overlay);
            for n in 0..4 {
                let rgb = if n + 1 == self.count { LIT } else { DIM };
                put(&(n + 1).to_string(), -160 + 100 * n as i16, -35, 1, rgb, false, false, overlay);
            }
        } else {
            let head = if self.count == 1 {
                self.strings[2].clone()
            } else {
                self.strings[3].replace("%s", &self.strings[4 + self.count - 2])
            };
            put(head.trim_end(), 0, -65, 1, LIT, true, true, overlay);
            let p = self.picks.len();
            let line = self.strings[7].replace("%d", &(p + 1).to_string());
            put(line.trim_end(), 0, -35, 0, crate::levels::golf::PLAYER_COLOURS[p % 4], true, true, overlay);
        }
        // The six heads at (-120 + 130 c, 25 + 60 r), depth 5500.
        for (slot, head) in heads.iter().enumerate() {
            let Some(m) = head else { continue };
            let (x, y) = face_at(slot);
            let at = camera + Vec3::new(x as f32 * DEPTH / (focal * 1.6), y as f32 * DEPTH / focal, DEPTH);
            rugrats_render::models::model_triangles(
                m,
                Mat3::from_rotation_x(std::f32::consts::FRAC_PI_2) * HEAD_SCALE,
                at,
                moving,
            );
        }
        // The picks: a star behind the head and the player's number.
        for (p, &slot) in self.picks.iter().enumerate() {
            let (x, y) = face_at(slot);
            if let Some(sp) = sprites {
                // Behind the heads (OT 1): with the backdrop.
                sp.draw(STAR, (x, y - 19), 68.5 / 1.6, 37.0, 1024.0, 0.0, 1024.0, under);
            }
            let mut t = Text {
                font: 1,
                spacing: 1,
                rgb: crate::levels::golf::PLAYER_COLOURS[p % 4],
                x,
                y: y - 20,
                string: (p + 1).to_string(),
                ..Default::default()
            };
            fonts.centre(&mut t, true, true);
            t.triangles(fonts, overlay);
        }
        // The cursor under the face it is on.
        if self.step == 1 {
            if let Some(sp) = sprites {
                let (x, y) = face_at(self.col + 3 * self.row);
                sp.draw(CURSOR, (x + 15, y + 5), 15.5 / 1.6, 15.5, 1024.0, 0.0, 1024.0, overlay);
            }
        }
    }
}
