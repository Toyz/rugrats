//! The front end (SLUS_006.50:0x8003eb28): the title menu over its moving
//! backdrop, with the logo and the babies in 3D. It runs at 30 frames a
//! second (each frame two vertical blanks).

use rugrats_data::hierarchy::{self, Hierarchy};
use rugrats_data::model::{Model, Models};
use rugrats_render::{Vram, Vtx, models};

use crate::audio::sound::{EFFECTS_VOLUME, MUSIC_VOLUME};
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::input::{CROSS, Pads};
use crate::rand::Rand;
use crate::save::card::{Art, STATE_SIZE};
use crate::stage::Stage;
use crate::ui::audio_menu::{AudioMenu, Settings};
use crate::ui::card_menu::{self, CardMenu, Strings};
use crate::ui::golf_setup::{self, GolfSetup};
use crate::ui::hud::Sprites;
use crate::ui::screens::{HEADS, PRESS};
use crate::ui::text::{self, Fonts, Text};

/// The backdrop's patterns: six 128 x 128 4-bit tiles in CHARACTR.RAW,
/// three across and two down from VRAM (576, 256), each with its own CLUT
/// row at (688, 511 - index), every colour marked semi-transparent at
/// start-up (0x8001dbcc).
pub const CLUT_X: u16 = 688;
pub const CLUT_ROWS: std::ops::RangeInclusive<u16> = 506..=511;

/// Two layers fading one into the next (0x80034fdc, 0x8003528c, stepped by
/// 0x80034e80): A, opaque, at brightness `full - level`, and B, added on
/// top, at `level`. A fade starts every 30 frames; it moves A on to the
/// next pattern, takes `level` down 16 a frame, and at 0 moves B on to
/// match and puts `level` back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backdrop {
    /// Patterns as (column 0-2, row 0-1): 0x800c406c/70 and 0x800c4064/68.
    pub a: (u8, u8),
    pub b: (u8, u8),
    /// 0x800c408c, 0x800c408d.
    pub level: u8,
    pub full: u8,
    /// A fade under way (0x800c4074).
    pub fading: bool,
}

impl Default for Backdrop {
    fn default() -> Backdrop {
        // The executable's initial values.
        Backdrop { a: (2, 0), b: (2, 0), level: 0x80, full: 0x80, fading: false }
    }
}

fn next(p: &mut (u8, u8)) {
    p.0 += 1;
    if p.0 >= 3 {
        p.0 = 0;
        p.1 += 1;
    }
    if p.1 >= 2 {
        p.1 = 0;
    }
}

impl Backdrop {
    /// One frame of a fade (0x80034e80); false once it is over.
    pub fn step(&mut self) -> bool {
        if self.level == 0 {
            return true;
        }
        if self.level == self.full {
            next(&mut self.a);
        }
        self.level = self.level.wrapping_sub(16);
        if self.level != 0 {
            return true;
        }
        next(&mut self.b);
        self.level = self.full;
        false
    }

    /// Both layers, A then B, in screen pixels.
    pub fn triangles(&self, out: &mut Vec<Vtx>) {
        layer(self.a, self.full.wrapping_sub(self.level), false, out);
        layer(self.b, self.level, true, out);
    }
}

/// One layer: two 256 x 256 quads, x from -256 to 256 and y from -128 to
/// 128, texels 0-255 across each, through a texture window of the 128 x
/// 128 pattern (0x8009d138), as the original draws it.
fn layer(p: (u8, u8), grey: u8, added: bool, out: &mut Vec<Vtx>) {
    let (col, row) = (p.0 as u32, p.1 as u32);
    // Column 2 starts a new 64-word page; the others are in the first.
    let (page, u0) = if col == 2 { (26u32, 0u32) } else { (25, col * 128) };
    let page = if added { page + 32 } else { page };
    let clut = ((511 - (col + 3 * row)) << 6) | (CLUT_X as u32 >> 4);
    let semi = if added { Vtx::SEMI } else { 0 };
    let mode = clut | page << 16 | semi | Vtx::WRAP;
    let colour = grey as u32 * 0x01_0101;
    // A 128-texel window: the masks keep bit 7 from the offset.
    let window = 0x80 | 0x80 << 8 | u0 << 16 | (row * 128) << 24;
    for x0 in [0.0, 256.0] {
        let (y0, x1, y1) = (-8.0, x0 + 256.0, 248.0);
        let v = |x: f32, y: f32, u: u32, v: u32| Vtx { pos: [x, y, 0.0], colour, uv: Vtx::uv_of(u, v), mode, window };
        let (a, b, c, d) = (v(x0, y0, 0, 0), v(x1, y0, 255, 0), v(x0, y1, 0, 255), v(x1, y1, 255, 255));
        out.extend([a, b, c, b, d, c]);
    }
}

/// The title menu's items (0x8003d5a0): "Start Game", "Load Game",
/// "Audio" (string pointers 0x800bb6c0, 0x800bb51c, 0x800bb694), font 1,
/// centred, rising from below to y 25, 50, 75 at 12 a frame and wobbling
/// four times as they arrive.
pub fn menu_items(fonts: &Fonts, strings: [String; 3]) -> Vec<Text> {
    strings
        .into_iter()
        .enumerate()
        .map(|(k, s)| {
            let k = k as i16;
            let mut t = Text { font: 1, spacing: 1, string: s, ..Default::default() };
            fonts.centre(&mut t, true, false);
            t.target.1 = 25 * (k + 1);
            t.rest.1 = 25 * (k + 1);
            t.y = 200 + 50 * k;
            t.speed.1 = 12;
            t.count.1 = 4;
            t.rgb = [255, 255, 255];
            t.flags = text::SLIDE_Y | text::WOBBLE;
            t
        })
        .collect()
}

/// The Activities menu's items: font 1, sliding up into two columns.
pub fn activity_items(fonts: &Fonts, strings: &[String; 4]) -> Vec<Text> {
    strings
        .iter()
        .zip(ACTIVITY_AT)
        .enumerate()
        .map(|(k, (s, (x, y)))| {
            let _ = fonts;
            Text {
                font: 1,
                spacing: 1,
                string: s.clone(),
                x,
                y: 200 + 50 * (k as i16 % 2),
                target: (x, y),
                rest: (x, y),
                speed: (0, 12),
                count: (0, 4),
                rgb: [255, 255, 255],
                flags: text::SLIDE_Y | text::WOBBLE,
                ..Default::default()
            }
        })
        .collect()
}

/// A menu item drawn as the menu draws them (0x80035ab8): its shadow
/// first, black, two pixels down and right, then the item, yellow when
/// chosen.
pub fn menu_triangles(fonts: &Fonts, items: &[Text], chosen: usize, out: &mut Vec<Vtx>) {
    for (k, t) in items.iter().enumerate() {
        let mut shadow = t.clone();
        shadow.rgb = [1, 1, 1];
        shadow.x += 2;
        shadow.y += 2;
        shadow.triangles(fonts, out);
        let mut lit = t.clone();
        lit.rgb = if k == chosen { [255, 255, 5] } else { [255, 255, 255] };
        lit.triangles(fonts, out);
    }
}

/// Files the front end puts in VRAM: MENU.TEX at (0, 256), 576 x 256
/// (0x8003eb28, the rectangle at 0x800c40d8); CHARACTR.RAW, loaded at boot
/// to (576, 256), 384 x 256.
const MENU_TEX: usize = 108;
const CHARACTR_RAW: usize = 498;

/// The strings of the title menu's items.
const ITEM_STRINGS: [u32; 3] = [0x800b_b6c0, 0x800b_b51c, 0x800b_b694];
/// The Start Game menu's (0x8003cc9c): "Main Game", "Activities",
/// "Training".
const START_STRINGS: [u32; 3] = [0x800b_b6b4, 0x800b_b6b8, 0x800b_b6bc];

/// The game modes the Start Game menu returns.
pub const MAIN_GAME: u8 = 1;
pub const TRAINING: u8 = 16;

/// Which list the title shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Main,
    /// 0x8003cf80.
    StartGame,
    /// 0x8003b7bc.
    Activities,
    /// The card's records (0x800398bc(8, 1, 0, 0)).
    Load,
    /// The audio settings (0x80041970).
    Audio,
    /// Mini-golf's setup (0x80096134).
    GolfSetup,
}

/// The Activities menu's items (0x800bb6a4..b0), where they sit, and
/// what they play (0x80013030: Cookie Race 13, Gold Rush 17, Mini-Golf
/// -2 (its course menu 0x80096134; for now Hole One, level 25), Egg Hunt
/// 18).
const ACTIVITY_STRINGS: [u32; 4] = [0x800b_b6a4, 0x800b_b6a8, 0x800b_b6ac, 0x800b_b6b0];
const ACTIVITY_AT: [(i16, i16); 4] = [(-225, 50), (-225, 75), (80, 50), (80, 75)];
const ACTIVITY_LEVELS: [u8; 4] = [13, 17, 25, 18];

/// Database 0's models (file 242 + 0) and the sign among them.
const DB00_MOD: usize = 242;
const SIGN_MODEL: u32 = 1038;
/// The sign's wobble (0x800b7dd4): five (x, y) steps.
const WOBBLE: u32 = 0x800b_7dd4;
/// The front end's camera (0x80040194): field of view 512, at (0, 30,
/// -500), looking along z.
const FOV: i32 = 512;
const CAMERA: [f32; 3] = [0.0, 30.0, -500.0];

/// The Rugrats sign: model 1038 at the origin, its vertices nudged every
/// other frame (0x8003f200, from 0x8003f350 while it is still).
pub struct Sign {
    pub model: Model,
    steps: [(i16, i16); 5],
    /// 0x800c40e0, and the step (0x800c40e8).
    ticks: u32,
    phase: u32,
}

impl Sign {
    /// One frame: on even ticks, odd vertices move by step `phase % 5`
    /// and even ones back by step `4 - phase % 5`, the steps summing to
    /// nothing over five.
    pub fn step(&mut self) {
        let tick = self.ticks;
        self.ticks += 1;
        if tick & 1 != 0 {
            return;
        }
        let k = (self.phase % 5) as usize;
        for (i, v) in self.model.vertices.iter_mut().enumerate() {
            let (dx, dy) = if i & 1 == 1 {
                self.steps[k]
            } else {
                let (x, y) = self.steps[4 - k];
                (-x, -y)
            };
            v[0] = v[0].wrapping_add(dx);
            v[1] = v[1].wrapping_add(dy);
        }
        self.phase += 1;
    }
}

/// The trademark beside the sign (0x8003f0f0): a raw textured quad from
/// page 0x1e, CLUT 0x64fb, drawn while the sign is still.
pub(crate) fn trademark(out: &mut Vec<Vtx>) {
    let mode = 0x64fb | 0x1e << 16;
    let window = Vtx::window_of([192, 212, 192, 212], [148, 148, 160, 160]);
    let v = |x: f32, y: f32, u: u32, v: u32| Vtx {
        pos: [x + 256.0, y + 120.0, 0.0],
        colour: 0x80_8080,
        uv: Vtx::uv_of(u, v),
        mode,
        window,
    };
    let (a, b, c, d) =
        (v(210.0, -75.0, 192, 148), v(230.0, -75.0, 212, 148), v(210.0, -69.0, 192, 160), v(230.0, -69.0, 212, 160));
    out.extend([a, b, c, b, d, c]);
}

/// The buttons' model (0x8003f444(0)), the CANCEL and ACCEPT labels'
/// strings, and where the cubes sit.
const BUTTON_MODEL: u32 = 334;
const CANCEL: u32 = 0x800b_b504;
const ACCEPT: u32 = 0x800b_b508;
/// The second camera (0x80040010), which the buttons are drawn with:
/// (0, 0, -500), looking along z.
const BUTTON_CAMERA: [f32; 3] = [0.0, 0.0, -500.0];

/// A button cube beside CANCEL or ACCEPT (a menu block, 0x8003f444): its
/// angles ease toward their targets (0x8003f7f0), and it is drawn from
/// them (object flag 0x20: 0x80020ee0, transposed by the object
/// transform).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    pub pos: [i32; 3],
    /// +0x24, the targets +0x2c, the step +0xce.
    pub angles: [i16; 3],
    pub targets: [i16; 3],
    pub speed: i16,
}

impl Button {
    /// One frame of 0x8003f7f0's turning.
    pub fn step(&mut self) {
        for (a, t) in self.angles.iter_mut().zip(self.targets.iter_mut()) {
            if *t < *a {
                *a = (*a).wrapping_sub(self.speed).max(*t);
            } else if *a < *t {
                *a = (*a).wrapping_add(self.speed).min(*t);
            } else if a.unsigned_abs() == 4096 && t.unsigned_abs() == 4096 {
                // A whole turn reads as none.
                *a = 0;
                *t = 0;
            }
        }
    }
}

/// The card page's cubes, new: CANCEL's at (-600, -400, 1298) turning to
/// a half turn, ACCEPT's at (600, -400, 1298) and PRESS X's at (-60, 414,
/// 1298) to a quarter, 128 a frame (0x80035550, 0x80035674, 0x80035798).
pub fn card_cubes() -> [Button; 3] {
    let cube = |pos: [i32; 3], target: i16| Button { pos, angles: [0; 3], targets: [target, 0, 0], speed: 128 };
    [cube([-600, -400, 1298], 2048), cube([600, -400, 1298], 1024), cube([-60, 414, 1298], 1024)]
}

/// Database 0's hierarchies (file 370 + 0): the babies are 1-5.
const DB00_HIE: usize = 370;

/// The corners a baby peeks from (0x80043098): where it starts, where it
/// slides to, and its turn about z.
const CORNERS: [([i32; 2], [i32; 2], i16); 4] = [
    ([-3000, -2500], [-1800, -1300], -1536),
    ([3000, -2500], [1800, -1300], 1536),
    ([3000, 2500], [1800, 1300], 512),
    ([-3000, 2500], [-1800, 1300], -512),
];

/// A peeking baby (0x800c4b20, 0x800c4b24: made by 0x80035d44, run by
/// 0x80043098, moved by 0x80042f34).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baby {
    /// +0x5b2.
    pub state: i16,
    /// +0x5b4: the corner (0-3; 4-7 sit out), or a count while waiting.
    pub corner: i16,
    /// +0x5b8: which of the five babies.
    pub kind: i16,
    /// Which hierarchy copy (an index into 0x800e2440): hierarchy n + 1.
    pub hierarchy: usize,
    /// +0x18, its target +0x19c, the steps toward it +0x5aa.
    pub pos: [i32; 3],
    pub target: [i32; 3],
    pub speed: [i16; 3],
    /// +0x24, the targets +0x2c, the step +0x5b0.
    pub angles: [i16; 3],
    pub turn_to: [i16; 3],
    pub turn: i16,
}

impl Default for Baby {
    /// As 0x80035d44 makes one.
    fn default() -> Baby {
        Baby {
            state: 0,
            corner: 0,
            kind: 0,
            hierarchy: 0,
            pos: [3000, 2500, 4000],
            target: [3000, 2500, 4000],
            speed: [20, 20, 500],
            angles: [0; 3],
            turn_to: [2048, 0, 0],
            turn: 1024,
        }
    }
}

/// The corners and babies in use, shared by the two (0x800c4090,
/// 0x800c4094).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InUse {
    pub corners: u32,
    pub kinds: u32,
}

impl Baby {
    /// One frame of 0x80043098.
    pub fn think(&mut self, used: &mut InUse, rng: &mut Rand) {
        match self.state {
            0 => {
                self.corner = (rng.next() & 7) as i16;
                let mut kind = 0;
                if self.corner < 5 {
                    while used.corners & (1 << self.corner) != 0 {
                        self.corner = (rng.next() % 5) as i16;
                    }
                    used.corners |= 1 << self.corner;
                    kind = rng.next() % 5;
                    while used.kinds & (1 << kind) != 0 {
                        kind = rng.next() % 5;
                    }
                    self.hierarchy = kind as usize;
                    used.kinds |= 1 << kind;
                }
                self.kind = kind as i16;
                // Kinds 1-5 also set an object's animation by registry id
                // (0x80075304: 1492, 1776, 600, 608): none here.
                self.speed[0] = 80;
                self.speed[1] = 80;
                self.state = 1;
                self.turn = 1024;
                self.angles = [2176, 64, 512];
                self.turn_to = [2176, 64, 512];
                match CORNERS.get(self.corner as usize) {
                    Some(&(from, to, z)) => {
                        self.turn_to[0] = 2048;
                        self.turn_to[2] = z;
                        self.pos[0] = from[0];
                        self.pos[1] = from[1];
                        self.target[0] = to[0];
                        self.target[1] = to[1];
                    }
                    None => self.state = 7,
                }
            }
            1 => {
                self.speed[0] -= 2;
                self.speed[1] -= 2;
                if self.pos[0] == self.target[0] && self.pos[1] == self.target[1] {
                    self.state = 2;
                }
            }
            2 => {
                // A giggle (0x80070064), drawing on rand twice.
                rng.next();
                rng.next();
                self.speed[0] = 0;
                self.speed[1] = 0;
                self.state = 3;
                if let Some(&(from, to, _)) = CORNERS.get(self.corner as usize) {
                    self.pos[0] = to[0];
                    self.pos[1] = to[1];
                    self.target[0] = from[0];
                    self.target[1] = from[1];
                }
            }
            3 => {
                self.speed[0] += 2;
                self.speed[1] += 2;
                if self.pos[0] == self.target[0] && self.pos[1] == self.target[1] {
                    self.state = 0;
                    used.kinds &= !(1 << self.kind);
                    used.corners &= !(1 << self.corner);
                }
            }
            4 => {
                self.turn += 1;
                if self.turn_to[1] == self.angles[1] {
                    self.turn_to[1] = -2304;
                    self.state = 5;
                }
            }
            5 => {
                self.turn -= 2;
                if self.turn <= 0 {
                    self.turn = 1;
                }
                if self.turn_to[1] == self.angles[1] {
                    self.turn_to[1] = 0;
                    self.turn = 0;
                    self.state = 6;
                }
            }
            6 => {
                self.turn += 1;
                if self.turn_to[1] == self.angles[1] {
                    self.turn = 32;
                    self.state = 0;
                    used.kinds &= !(1 << self.kind);
                    used.corners &= !(1 << self.corner);
                }
            }
            7 => {
                // Sitting out: the count runs to 31, then a new pick. The
                // corner and baby it marked stay marked.
                let waited = self.corner >= 31;
                self.corner += 1;
                if waited {
                    self.state = 0;
                }
            }
            8 => {
                match self.corner {
                    0..=3 => {
                        let (from, _, z) = CORNERS[self.corner as usize];
                        self.turn_to[0] = 2048;
                        self.turn_to[2] = z;
                        self.target[0] = from[0];
                        self.target[1] = from[1];
                    }
                    4 => {
                        self.target[1] = -1700;
                        self.pos[1] = -1700;
                        self.turn_to = [0; 3];
                        self.target[0] = 0;
                        self.pos[0] = 0;
                    }
                    _ => return,
                }
                self.state = 9;
            }
            9 => {
                self.speed[0] += 4;
                self.turn += 4;
                self.speed[1] += 4;
            }
            _ => {}
        }
    }

    /// One frame of 0x80042f34: the angles, then the position, each
    /// stepping toward its target without passing it.
    pub fn step(&mut self) {
        for (a, t) in self.angles.iter_mut().zip(self.turn_to.iter_mut()) {
            if *t < *a {
                *a = (*a).wrapping_sub(self.turn).max(*t);
            } else if *a < *t {
                *a = (*a).wrapping_add(self.turn).min(*t);
            } else if a.unsigned_abs() == 4096 && t.unsigned_abs() == 4096 {
                *a = 0;
                *t = 0;
            }
        }
        for ((p, &t), &s) in self.pos.iter_mut().zip(&self.target).zip(&self.speed) {
            let s = s as i32;
            if t < *p {
                *p = if *p - s < t { t } else { *p - s };
            } else if *p < t {
                *p = if t < *p + s { t } else { *p + s };
            }
        }
    }
}

/// What the title menu ends with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Triangle, or 30 seconds untouched: back to main's chooser (-1).
    Idle,
    /// A game mode chosen (stored to 0x800c5678): 1 the main game, 16
    /// training.
    Play(u8),
    /// Load Game (0x800398bc(8, 1, 0, 0)): the state a record held.
    Load(Box<[u8; STATE_SIZE]>),
    /// Mini-golf set up: each player's face slot.
    Golf(Vec<usize>),
}

/// What a chosen item leads to, once its items have slid away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The Start Game menu (0x8003cf80).
    StartGame,
    /// The memory card menu (0x800398bc(8, 1, 0, 0)).
    LoadGame,
    /// The audio settings (0x80041970).
    Audio,
}

/// The idle countdown (0x8003d884: 300 x its argument, 30), and what a
/// frame takes off it (5 a vertical blank, 0x800c3f1c).
const COUNTDOWN: i32 = 300 * 30;
const FRAME_TIME: i32 = 10;

/// Sounds the menu plays (0x8006f26c), by the game's numbers.
pub mod sound {
    pub const LEAVE: u8 = 1;
    pub const PICK: u8 = 2;
    pub const DOWN: u8 = 3;
    pub const UP: u8 = 4;
    pub const SHOULDERS: u8 = 5;
}

/// The title menu as it stands (0x8003d884).
pub struct Title {
    /// The two peeking babies and what they have taken.
    pub babies: [Baby; 2],
    pub in_use: InUse,
    hierarchies: Vec<Hierarchy>,
    models: Models,
    /// The items sliding away before an action.
    pub leaving: Option<Action>,
    strings: [String; 3],
    start_strings: [String; 3],
    activity_strings: [String; 4],
    pub menu: Menu,
    card_art: Art,
    /// What the Start Game menu's items sliding away end with.
    start_leaving: Option<Option<u8>>,
    /// Frames until the menu gives up (0x8003d884's s2).
    pub countdown: i32,
    /// Sounds asked for this frame, for the sound side.
    pub sounds: Vec<u8>,
    /// CANCEL's and ACCEPT's cubes (0x800c4b14, 0x800c4b18), their labels.
    pub buttons: [Button; 2],
    labels: [Text; 2],
    button_model: Model,
    trig: Trig,
    pub sign: Sign,
    pub stage: Stage,
    fonts: Fonts,
    pub backdrop: Backdrop,
    pub items: Vec<Text>,
    pub chosen: usize,
    /// Frames since the front end began (0x800c4080).
    pub frames: u32,
    started: u64,
    /// The audio settings as main has them, the menu editing them, and
    /// the song volume to set this frame.
    pub settings: Settings,
    audio: Option<AudioMenu>,
    pub song_volume: Option<i32>,
    audio_strings: [String; 5],
    /// The card pages (Load Game), their strings, PRESS X's text, and the
    /// page's own cubes: CANCEL's, ACCEPT's and PRESS X's (0x800398bc
    /// makes them anew, 0x800c4b14, 0x800c4b18, 0x800c4b40).
    card: Option<CardMenu>,
    card_cubes: [Button; 3],
    card_strings: Strings,
    press: String,
    /// Mini-golf's setup, the count kept between visits (0x800c46cc),
    /// the menu's sprites (TEXPOS entry 0), the heads, the projection
    /// and the setup's strings.
    golf: Option<GolfSetup>,
    golf_count: usize,
    sprites: Option<Sprites>,
    heads: Vec<Option<Model>>,
    focal: f32,
    golf_strings: Vec<String>,
}

fn words(bytes: &[u8]) -> Vec<u16> {
    bytes.as_chunks::<2>().0.iter().map(|h| u16::from_le_bytes([h[0], h[1]])).collect()
}

/// VRAM as the front end has it: MENU.TEX at (0, 256), the characters'
/// image beside it with the backdrop's CLUTs marked semi-transparent, and
/// the fonts.
pub fn menu_vram(disc: &GameDisc) -> Result<Vram, String> {
    let mut vram = Vram::new();
    vram.load(0, 256, 576, 256, &words(&disc.file(MENU_TEX)?));
    let mut chars = words(&disc.file(CHARACTR_RAW)?);
    for row in CLUT_ROWS {
        let at = (row as usize - 256) * 384 + (CLUT_X as usize - 576);
        for w in chars.iter_mut().skip(at).take(16) {
            *w |= 0x8000;
        }
    }
    vram.load(576, 256, 384, 256, &chars);
    let (fx, fy) = text::FONT_AT;
    let (fw, fh) = text::FONT_SIZE;
    vram.load(fx, fy, fw, fh, &words(&disc.file(text::FONT_FILE)?));
    Ok(vram)
}

/// The game's string table (0x800bb4e8): string `n`.
pub fn string(disc: &GameDisc, n: u32) -> String {
    disc.exe_string_at(0x800b_b4e8 + 4 * n).unwrap_or_default()
}

/// A hint card (0x80044434): a HINT file's lines over the front end's
/// backdrop, until Cross.
pub struct Card {
    pub stage: Stage,
    fonts: Fonts,
    backdrop: Backdrop,
    texts: Vec<Text>,
    frames: u32,
}

/// The cards after cinematics (CineFinish, 0x800783d8's switch on cine -
/// 1): the HINT file.
pub fn card_after(cine: usize) -> Option<usize> {
    Some(match cine {
        1 => 674,
        2 => 679,
        3 => 675,
        4 => 676,
        6 => 706,
        7 => 672,
        8 | 9 => 681,
        12 => 707,
        13 => 682,
        17 => 699,
        18 => 710,
        25 => 697,
        28 => 685,
        34 => 698,
        37 => 687,
        38 => 693,
        44 => 677,
        45 => 694,
        46 => 695,
        49 => 696,
        51 => 709,
        52 => 700,
        55 => 701,
        58 => 702,
        63 => 678,
        65 => 709,
        66 => 703,
        67 => 705,
        73 => 684,
        79 => 708,
        86 => 683,
        _ => return None,
    })
}

impl Card {
    /// HINT file `file`.
    pub fn open(disc: &GameDisc, file: usize) -> Result<Card, String> {
        let fonts = Fonts::load(disc)?;
        let raw = disc.file(file)?;
        let body = String::from_utf8_lossy(&raw).into_owned();
        let mut texts = Vec::new();
        let line = |string: String, y: i16, rgb: [u8; 3], fonts: &Fonts| {
            let mut t = Text { font: 0, spacing: 1, rgb, x: 0, y, string, ..Default::default() };
            fonts.centre(&mut t, true, false);
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            [shadow, t]
        };
        for (k, l) in body.split("\r\n").take_while(|l| !l.starts_with('~')).enumerate() {
            texts.extend(line(l.to_string(), -90 + 18 * k as i16, [0x7f, 0x7f, 0x7f], &fonts));
        }
        // "PRESS     TO CONTINUE", the gap the Cross button's.
        let press = string(disc, 67).replacen("     ", "  %  ", 1);
        texts.extend(line(press, 80, [255, 255, 255], &fonts));
        let stage = Stage::new(menu_vram(disc)?, Vec::new());
        let mut card = Card { stage, fonts, backdrop: Backdrop::default(), texts, frames: 0 };
        card.draw();
        Ok(card)
    }

    /// One frame (every other tick); true when Cross closes it.
    pub fn frame(&mut self, pads: &Pads) -> bool {
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        self.frames += 1;
        self.draw();
        pads.pressed(0, CROSS)
    }

    fn draw(&mut self) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.overlay.clear();
        for t in &self.texts {
            t.triangles(&self.fonts, &mut self.stage.overlay);
        }
    }
}

impl Title {
    /// The front end's setup, which also draws on rand once (an object it
    /// makes, through 0x80048a44, the call at 0x80048b78).
    pub fn open(disc: &GameDisc, now: u64, rng: &mut Rand) -> Result<Title, String> {
        rng.next();
        let vram = menu_vram(disc)?;
        let fonts = Fonts::load(disc)?;
        let models = Models::parse(&disc.file(DB00_MOD)?).map_err(|e| e.to_string())?;
        let model = models.get(SIGN_MODEL).ok_or("no sign model in DB00")?.clone();
        let raw = disc.exe_bytes(WOBBLE, 20)?;
        let h = |k: usize| i16::from_le_bytes([raw[k], raw[k + 1]]);
        let steps = [0, 1, 2, 3, 4].map(|k| (h(4 * k), h(4 * k + 2)));
        let sign = Sign { model, steps, ticks: 0, phase: 0 };
        let trig = Trig::load(disc)?;
        let projection = trig.geom_screen(FOV).ok_or("no projection")? as f32;
        let strings = ITEM_STRINGS.map(|a| disc.exe_string_at(a).unwrap_or_default());
        let start_strings = START_STRINGS.map(|a| disc.exe_string_at(a).unwrap_or_default());
        let activity_strings = ACTIVITY_STRINGS.map(|a| disc.exe_string_at(a).unwrap_or_default());
        let items = menu_items(&fonts, strings.clone());
        let mut stage = Stage::new(vram, Vec::new());
        stage.camera = crate::stage::view(CAMERA.into(), rrt::glam::Mat3::IDENTITY, projection);
        let button_model = models.get(BUTTON_MODEL).ok_or("no button model in DB00")?.clone();
        // The five babies' skeletons: hierarchy copies 1-5 (0x800e2440).
        let all = hierarchy::parse_file(&disc.file(DB00_HIE)?).map_err(|e| e.to_string())?;
        let hierarchies = (1..=5)
            .map(|k| all.get(k).cloned().flatten().ok_or(format!("no hierarchy {k} in DB00")))
            .collect::<Result<Vec<_>, _>>()?;
        // The labels (0x80035550, 0x80035674): font 2, as they are, over
        // shadows two pixels down and right.
        let label = |addr: u32, x: i16| Text {
            font: 2,
            spacing: 1,
            rgb: [255, 255, 255],
            x,
            y: -93,
            string: disc.exe_string_at(addr).unwrap_or_default(),
            ..Default::default()
        };
        let labels = [label(CANCEL, -192), label(ACCEPT, 115)];
        // Their cubes at (-600 or 600, -400, 1298), turning from 0 to a
        // half turn (CANCEL) or a quarter (ACCEPT) at 128 a frame.
        let button =
            |x: i32, target: i16| Button { pos: [x, -400, 1298], angles: [0; 3], targets: [target, 0, 0], speed: 128 };
        let buttons = [button(-600, 2048), button(600, 1024)];
        let mut title = Title {
            babies: [Baby::default(), Baby::default()],
            in_use: InUse::default(),
            hierarchies,
            models,
            leaving: None,
            strings,
            start_strings,
            activity_strings,
            menu: Menu::Main,
            card_art: disc.card.clone(),
            start_leaving: None,
            countdown: COUNTDOWN,
            sounds: Vec::new(),
            buttons,
            labels,
            button_model,
            trig,
            sign,
            stage,
            fonts,
            backdrop: Backdrop::default(),
            items,
            chosen: 0,
            frames: 0,
            started: now,
            settings: Settings { music: MUSIC_VOLUME, effects: EFFECTS_VOLUME, mono: false },
            audio: None,
            song_volume: None,
            audio_strings: AudioMenu::strings(disc),
            card: None,
            card_cubes: card_cubes(),
            card_strings: Strings::load(disc),
            press: string(disc, PRESS),
            golf: None,
            golf_count: 1,
            sprites: Sprites::load(disc, 99).ok(),
            heads: Vec::new(),
            focal: projection,
            golf_strings: [87u32, 86, 199, 200, 120, 121, 122, 201].iter().map(|&n| string(disc, n)).collect(),
        };
        title.heads =
            HEADS.iter().map(|&id| title.models.has(id).then(|| title.models.get(id).cloned()).flatten()).collect();
        title.draw(true);
        Ok(title)
    }

    /// The Activities menu at once (back from an activity: 0x8003b7bc).
    pub fn activities(&mut self) {
        self.menu = Menu::Activities;
        self.items = activity_items(&self.fonts, &self.activity_strings);
        self.chosen = 0;
    }

    /// Whether tick `now` starts one of the menu's frames (every other).
    pub fn frame_due(&self, now: u64) -> bool {
        (now - self.started).is_multiple_of(2)
    }

    /// One frame of the menu (0x8003d884's loop), the pads as sampled at
    /// the end of the last.
    pub fn frame(&mut self, pads: &Pads, rng: &mut Rand) -> Option<Outcome> {
        use crate::input::*;
        self.sounds.clear();
        if let Some(action) = self.leaving {
            // The items slide away (0x80035a54), drawn by the menu's draw.
            self.menu_frame(rng);
            if self.items.iter().any(|t| t.flags & text::SLIDE_Y != 0) {
                return None;
            }
            self.leaving = None;
            if action == Action::StartGame {
                // 0x8003cf80 sets up its own three items (0x8003cc9c).
                self.menu = Menu::StartGame;
                self.items = menu_items(&self.fonts, self.start_strings.clone());
                self.chosen = 0;
                self.draw(true);
                return None;
            }
            if action == Action::LoadGame {
                self.menu = Menu::Load;
                self.card = Some(CardMenu::open(false, true, self.card_art.clone(), [0; STATE_SIZE], [0; 3]));
                self.card_cubes = card_cubes();
                self.chosen = 0;
                self.items.clear();
                self.draw_load();
                return None;
            }
            // Audio (0x80041970).
            self.menu = Menu::Audio;
            self.audio = Some(AudioMenu::open(&self.audio_strings, &self.fonts, self.settings));
            self.items.clear();
            self.draw(true);
            return None;
        }
        if self.menu == Menu::Audio {
            return self.audio_frame(pads);
        }
        if self.menu == Menu::GolfSetup {
            return self.golf_frame(pads);
        }
        if self.menu == Menu::Activities {
            return self.activities_frame(pads, rng);
        }
        if self.menu == Menu::Load {
            return self.load_frame(pads, rng);
        }
        if self.menu == Menu::StartGame {
            return self.start_frame(pads, rng);
        }
        // A fade every 30 frames.
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        self.frames += 1;
        let sliding = self.items.iter().any(|t| t.flags & text::SLIDE_Y != 0);
        let pressed = |b: usize| pads.pressed(0, b);
        if pressed(START) && !sliding {
            // The babies go too (0x5b2 = 8).
            for b in &mut self.babies {
                b.state = 8;
            }
            self.in_use = InUse::default();
            self.leave(Action::StartGame, rng);
            return None;
        }
        if pressed(DOWN) || pressed(SELECT) {
            self.sounds.push(sound::DOWN);
            self.chosen = (self.chosen + 1) % 3;
            self.countdown = COUNTDOWN;
        }
        if pressed(UP) {
            self.sounds.push(sound::UP);
            self.chosen = (self.chosen + 2) % 3;
            self.countdown = COUNTDOWN;
        }
        if pressed(CROSS) && !sliding {
            let shoulders = [L1, R1, L2, R2].iter().all(|&b| pads.held(0, b));
            self.sounds.push(if shoulders { sound::SHOULDERS } else { sound::PICK });
            if self.chosen > 0 {
                self.sounds.push(sound::PICK);
            }
            self.leave([Action::StartGame, Action::LoadGame, Action::Audio][self.chosen], rng);
            return None;
        }
        self.menu_frame(rng);
        self.countdown -= FRAME_TIME;
        if pressed(TRIANGLE) || self.countdown < 0 {
            self.sounds.push(sound::LEAVE);
            return Some(Outcome::Idle);
        }
        None
    }

    /// One frame of the Start Game menu (0x8003cf80's loop). Its buttons:
    /// Down or Select and Up move; Start plays the main game at once;
    /// Triangle goes back; Cross picks (with the four shoulders held, the
    /// main game goes through the level chooser 0x8003bea0).
    fn start_frame(&mut self, pads: &Pads, rng: &mut Rand) -> Option<Outcome> {
        use crate::input::*;
        if let Some(end) = self.start_leaving {
            self.menu_frame(rng);
            if self.items.iter().any(|t| t.flags & text::SLIDE_Y != 0) {
                return None;
            }
            self.start_leaving = None;
            return match end {
                Some(mode) => Some(Outcome::Play(mode)),
                None => {
                    // Back to the title's menu (0x8003d5a0).
                    self.menu = Menu::Main;
                    self.items = menu_items(&self.fonts, self.strings.clone());
                    self.chosen = 0;
                    self.countdown = COUNTDOWN;
                    self.draw(true);
                    None
                }
            };
        }
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        self.frames += 1;
        let pressed = |b: usize| pads.pressed(0, b);
        let sliding = self.items.iter().any(|t| t.flags & text::SLIDE_Y != 0);
        if pressed(DOWN) || pressed(SELECT) {
            self.sounds.push(sound::DOWN);
            self.chosen = (self.chosen + 1) % 3;
        }
        if pressed(UP) {
            self.sounds.push(sound::UP);
            self.chosen = (self.chosen + 2) % 3;
        }
        let end = if pressed(START) {
            Some(Some(MAIN_GAME))
        } else if pressed(TRIANGLE) && !sliding {
            self.sounds.push(sound::LEAVE);
            Some(None)
        } else if pressed(CROSS) && !sliding {
            let shoulders = [L1, R1, L2, R2].iter().all(|&b| pads.held(0, b));
            self.sounds.push(if shoulders { sound::SHOULDERS } else { sound::PICK });
            match self.chosen {
                0 => Some(Some(MAIN_GAME)),
                2 => Some(Some(TRAINING)),
                _ => {
                    // 0x8003b7bc: its own four items, sliding in.
                    self.menu = Menu::Activities;
                    self.items = activity_items(&self.fonts, &self.activity_strings);
                    self.chosen = 0;
                    None
                }
            }
        } else {
            None
        };
        if let Some(end) = end {
            self.start_leaving = Some(end);
            if end.is_some() {
                for b in &mut self.babies {
                    b.state = 8;
                }
            }
            self.slide_away();
        }
        self.menu_frame(rng);
        None
    }

    /// One frame of the Activities menu (0x8003b7bc): Down or Select and
    /// Up step through the four, Left and Right change column; Cross
    /// plays one, Triangle goes back to Start Game.
    fn activities_frame(&mut self, pads: &Pads, rng: &mut Rand) -> Option<Outcome> {
        use crate::input::*;
        if let Some(end) = self.start_leaving {
            self.menu_frame(rng);
            if self.items.iter().any(|t| t.flags & text::SLIDE_Y != 0) {
                return None;
            }
            self.start_leaving = None;
            return match end {
                // Mini-golf goes to its setup first (0x80096134).
                Some(25) => {
                    self.menu = Menu::GolfSetup;
                    self.golf = Some(GolfSetup::with_strings(self.golf_strings.clone(), self.golf_count));
                    self.items.clear();
                    self.draw_golf();
                    None
                }
                Some(level) => Some(Outcome::Play(level)),
                None => {
                    self.menu = Menu::StartGame;
                    self.items = menu_items(&self.fonts, self.start_strings.clone());
                    self.chosen = 1;
                    self.draw(true);
                    None
                }
            };
        }
        let pressed = |b: usize| pads.pressed(0, b);
        if pressed(DOWN) || pressed(SELECT) {
            self.sounds.push(sound::DOWN);
            self.chosen = (self.chosen + 1) % 4;
        }
        if pressed(UP) {
            self.sounds.push(sound::UP);
            self.chosen = (self.chosen + 3) % 4;
        }
        if pressed(LEFT) || pressed(RIGHT) {
            self.sounds.push(sound::DOWN);
            self.chosen = (self.chosen + 2) % 4;
        }
        let end = if pressed(TRIANGLE) {
            self.sounds.push(sound::LEAVE);
            Some(None)
        } else if pressed(CROSS) {
            self.sounds.push(sound::PICK);
            Some(Some(ACTIVITY_LEVELS[self.chosen]))
        } else {
            None
        };
        if let Some(end) = end {
            self.start_leaving = Some(end);
            self.slide_away();
        }
        self.menu_frame(rng);
        None
    }

    /// One frame of Load Game (0x800398bc(8, 1, 0, 0)): the card pages;
    /// their Cross and Triangle returns end it with 0, back to the menu.
    fn load_frame(&mut self, pads: &Pads, rng: &mut Rand) -> Option<Outcome> {
        let _ = rng;
        let Some(card) = &mut self.card else { return None };
        let out = card.frame(pads);
        self.sounds.append(&mut card.sounds);
        match out {
            Some(card_menu::Outcome::Loaded(state)) => return Some(Outcome::Load(state)),
            Some(_) => {
                self.card = None;
                self.menu = Menu::Main;
                self.items = menu_items(&self.fonts, self.strings.clone());
                self.chosen = 1;
                self.countdown = COUNTDOWN;
                self.draw(true);
                return None;
            }
            None => {}
        }
        // The cubes shown turn (0x8003f934 steps them as it draws them).
        let footer = card.footer();
        for (k, c) in self.card_cubes.iter_mut().enumerate() {
            if (k == 2) == footer {
                c.step();
            }
        }
        self.frames += 1;
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        self.draw_load();
        None
    }

    fn draw_load(&mut self) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.moving.clear();
        self.stage.overlay.clear();
        if let Some(card) = &self.card {
            let labels = [self.labels[0].string.clone(), self.labels[1].string.clone()];
            card.triangles(&self.card_strings, &self.fonts, &self.press, &labels, &mut self.stage.overlay);
            let footer = card.footer();
            let shown: Vec<Button> =
                self.card_cubes.iter().enumerate().filter(|(k, _)| (*k == 2) == footer).map(|(_, c)| c.clone()).collect();
            for c in &shown {
                self.draw_cube(c);
            }
        }
    }

    /// Slide the items away (0x80035a54(items, 3, 150)): down to 150 at 8
    /// a frame, the second and third waiting 5 and 10 frames first.
    /// A frame of the Audio menu; at its end the title's items slide in
    /// again with Audio chosen (0x8003d5a0).
    fn audio_frame(&mut self, pads: &Pads) -> Option<Outcome> {
        self.song_volume = None;
        let Some(menu) = &mut self.audio else { return None };
        // The backdrop's fade steps twice a frame here (0x80042294,
        // 0x800427ec).
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        for _ in 0..2 {
            if self.backdrop.fading {
                self.backdrop.fading = self.backdrop.step();
            }
        }
        self.frames += 1;
        let done = menu.frame(pads);
        self.sounds.append(&mut menu.sounds);
        self.song_volume = menu.song_volume;
        self.settings = menu.settings;
        for b in &mut self.buttons {
            b.step();
        }
        if done.is_some() {
            self.audio = None;
            self.menu = Menu::Main;
            self.items = menu_items(&self.fonts, self.strings.clone());
            self.chosen = 2;
            self.countdown = COUNTDOWN;
            self.draw(true);
            return None;
        }
        self.draw_audio();
        None
    }

    /// A frame of mini-golf's setup: Triangle at the count goes back to
    /// the Activities menu; the last pick plays.
    fn golf_frame(&mut self, pads: &Pads) -> Option<Outcome> {
        let Some(setup) = &mut self.golf else { return None };
        let out = setup.frame(pads);
        self.sounds.append(&mut setup.sounds);
        self.golf_count = setup.count();
        self.frames += 1;
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        for b in &mut self.buttons {
            b.step();
        }
        match out {
            Some(golf_setup::Outcome::Back) => {
                self.golf = None;
                self.activities();
                self.draw(true);
                None
            }
            Some(golf_setup::Outcome::Play(slots)) => {
                self.golf = None;
                Some(Outcome::Golf(slots))
            }
            None => {
                self.draw_golf();
                None
            }
        }
    }

    fn draw_golf(&mut self) {
        let Some(setup) = &self.golf else { return };
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.moving.clear();
        self.stage.overlay.clear();
        setup.draw(
            &self.fonts,
            &self.heads,
            self.sprites.as_ref(),
            rrt::glam::Vec3::from(CAMERA),
            self.focal,
            &mut self.stage.under,
            &mut self.stage.moving,
            &mut self.stage.overlay,
        );
        self.draw_labels(2);
    }

    /// CANCEL and ACCEPT (`shown` of them) with their cubes.
    fn draw_labels(&mut self, shown: usize) {
        let shift = rrt::glam::Vec3::from(CAMERA) - rrt::glam::Vec3::from(BUTTON_CAMERA);
        for t in self.labels.iter().take(shown) {
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(&self.fonts, &mut self.stage.overlay);
            t.triangles(&self.fonts, &mut self.stage.overlay);
        }
        for b in self.buttons.iter().take(shown) {
            let m = self.trig.euler(b.angles);
            let rows = rrt::glam::Mat3::from_cols_array(
                &m.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
            )
            .transpose();
            let pos = rrt::glam::Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
            models::model_triangles(&self.button_model, rows.transpose(), pos, &mut self.stage.moving);
        }
    }

    fn draw_audio(&mut self) {
        let Some(menu) = &self.audio else { return };
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.moving.clear();
        self.stage.overlay.clear();
        let shift = rrt::glam::Vec3::from(CAMERA) - rrt::glam::Vec3::from(BUTTON_CAMERA);
        menu.draw(&self.models, &self.trig, shift, &self.fonts, &mut self.stage.moving, &mut self.stage.overlay);
        // CANCEL always, ACCEPT once the pads are read.
        let shown = if menu.accept_shown() { 2 } else { 1 };
        for t in self.labels.iter().take(shown) {
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(&self.fonts, &mut self.stage.overlay);
            t.triangles(&self.fonts, &mut self.stage.overlay);
        }
        for b in self.buttons.iter().take(shown) {
            let m = self.trig.euler(b.angles);
            let rows = rrt::glam::Mat3::from_cols_array(
                &m.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
            )
            .transpose();
            let pos = rrt::glam::Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
            models::model_triangles(&self.button_model, rows.transpose(), pos, &mut self.stage.moving);
        }
    }

    fn leave(&mut self, action: Action, rng: &mut Rand) {
        self.slide_away();
        self.leaving = Some(action);
        self.menu_frame(rng);
    }

    fn slide_away(&mut self) {
        for (k, t) in self.items.iter_mut().enumerate() {
            t.target.1 = 150;
            t.rest.1 = 150;
            t.speed.1 = 8;
            t.count.1 = t.y - 150;
            t.flags |= text::SLIDE_Y;
            if k > 0 {
                t.wait = 5 * k as i16;
                t.flags |= text::WAIT;
            }
        }
    }

    /// The menu's draw (0x80035ab8) and what runs with it each frame.
    fn menu_frame(&mut self, rng: &mut Rand) {
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        for t in &mut self.items {
            if t.flags & text::SLIDE_Y != 0 {
                t.step();
            }
        }
        self.sign.step();
        for b in &mut self.babies {
            b.think(&mut self.in_use, rng);
            b.step();
        }
        for b in &mut self.buttons {
            b.step();
        }
        // CANCEL and ACCEPT are drawn by the menu's loop, not while the
        // items slide away.
        self.draw(self.leaving.is_none());
    }

    fn draw(&mut self, labels: bool) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.moving.clear();
        models::model_triangles(
            &self.sign.model,
            rrt::glam::Mat3::IDENTITY,
            rrt::glam::Vec3::ZERO,
            &mut self.stage.moving,
        );
        // The babies, seen by the buttons' camera (draw_character,
        // 0x8003205c): at rest, turned by their angles (0x80020d10, the
        // transpose of 0x80020ee0's matrix).
        let shift = rrt::glam::Vec3::from(CAMERA) - rrt::glam::Vec3::from(BUTTON_CAMERA);
        for b in &self.babies {
            let Some(h) = self.hierarchies.get(b.hierarchy) else { continue };
            let e = self.trig.euler(b.angles);
            let rows = rrt::glam::Mat3::from_cols_array(
                &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
            )
            .transpose();
            let pos = rrt::glam::Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
            models::posed_triangles(h, &self.models, &[], rows.transpose(), pos, &mut self.stage.moving);
        }
        self.stage.overlay.clear();
        menu_triangles(&self.fonts, &self.items, self.chosen, &mut self.stage.overlay);
        trademark(&mut self.stage.overlay);
        if !labels {
            return;
        }
        for t in &self.labels {
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(&self.fonts, &mut self.stage.overlay);
            t.triangles(&self.fonts, &mut self.stage.overlay);
        }
        for b in self.buttons.clone() {
            self.draw_cube(&b);
        }
    }

    /// A button cube (0x8003f934): seen by its own camera, which differs
    /// from the main one only in where it stands; moved by the
    /// difference, it lands where the main camera shows it the same.
    fn draw_cube(&mut self, b: &Button) {
        let shift = rrt::glam::Vec3::from(CAMERA) - rrt::glam::Vec3::from(BUTTON_CAMERA);
        let m = self.trig.euler(b.angles);
        let rows = rrt::glam::Mat3::from_cols_array(
            &m.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
        .transpose();
        let mut tris = Vec::new();
        let pos = rrt::glam::Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
        models::model_triangles(&self.button_model, rows.transpose(), pos, &mut tris);
        self.stage.moving.extend(tris);
    }
}
