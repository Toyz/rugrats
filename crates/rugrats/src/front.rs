//! The front end (SLUS_006.50:0x8003eb28): the title menu over its moving
//! backdrop, with the logo and the babies in 3D. It runs at 30 frames a
//! second (each frame two vertical blanks).

use rugrats_render::Vtx;

use crate::text::{self, Fonts, Text};

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
        let v = |x: f32, y: f32, u: u32, v: u32| Vtx { pos: [x, y, 0.0], colour, uv: u | v << 8, mode, window };
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
    pub model: rugrats_data::model::Model,
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
            let (dx, dy) = if i & 1 == 1 { self.steps[k] } else { let (x, y) = self.steps[4 - k]; (-x, -y) };
            v[0] = v[0].wrapping_add(dx);
            v[1] = v[1].wrapping_add(dy);
        }
        self.phase += 1;
    }
}

/// The trademark beside the sign (0x8003f0f0): a raw textured quad from
/// page 0x1e, CLUT 0x64fb, drawn while the sign is still.
fn trademark(out: &mut Vec<Vtx>) {
    let mode = 0x64fb | 0x1e << 16;
    let window = Vtx::window_of([192, 212, 192, 212], [148, 148, 160, 160]);
    let v = |x: f32, y: f32, u: u32, v: u32| Vtx { pos: [x + 256.0, y + 120.0, 0.0], colour: 0x80_8080, uv: u | v << 8, mode, window };
    let (a, b, c, d) = (v(210.0, -75.0, 192, 148), v(230.0, -75.0, 212, 148), v(210.0, -69.0, 192, 160), v(230.0, -69.0, 212, 160));
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

/// The title menu as it stands (0x8003d884).
pub struct Title {
    /// CANCEL's and ACCEPT's cubes (0x800c4b14, 0x800c4b18), their labels.
    pub buttons: [Button; 2],
    labels: [Text; 2],
    button_model: rugrats_data::model::Model,
    trig: crate::gte::Trig,
    pub sign: Sign,
    pub stage: crate::stage::Stage,
    fonts: Fonts,
    pub backdrop: Backdrop,
    pub items: Vec<Text>,
    pub chosen: usize,
    /// Frames since the front end began (0x800c4080).
    pub frames: u32,
    started: u64,
}

fn words(bytes: &[u8]) -> Vec<u16> {
    bytes.chunks_exact(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect()
}

impl Title {
    pub fn open(disc: &crate::disc::GameDisc, now: u64) -> Result<Title, String> {
        let mut vram = rugrats_render::Vram::new();
        vram.load(0, 256, 576, 256, &words(&disc.file(MENU_TEX)?));
        let mut chars = words(&disc.file(CHARACTR_RAW)?);
        // The backdrop's CLUTs, every colour marked semi-transparent.
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
        let fonts = Fonts::load(disc)?;
        let models = rugrats_data::model::Models::parse(&disc.file(DB00_MOD)?).map_err(|e| e.to_string())?;
        let model = models.get(SIGN_MODEL).ok_or("no sign model in DB00")?.clone();
        let raw = disc.exe_bytes(WOBBLE, 20)?;
        let h = |k: usize| i16::from_le_bytes([raw[k], raw[k + 1]]);
        let steps = [0, 1, 2, 3, 4].map(|k| (h(4 * k), h(4 * k + 2)));
        let sign = Sign { model, steps, ticks: 0, phase: 0 };
        let trig = crate::gte::Trig::load(disc)?;
        let projection = trig.geom_screen(FOV).ok_or("no projection")? as f32;
        let strings = ITEM_STRINGS.map(|a| disc.exe_string_at(a).unwrap_or_default());
        let items = menu_items(&fonts, strings);
        let mut stage = crate::stage::Stage::new(vram, Vec::new());
        stage.camera = crate::stage::view(CAMERA.into(), rrt::glam::Mat3::IDENTITY, projection);
        let button_model = models.get(BUTTON_MODEL).ok_or("no button model in DB00")?.clone();
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
        let button = |x: i32, target: i16| Button { pos: [x, -400, 1298], angles: [0; 3], targets: [target, 0, 0], speed: 128 };
        let buttons = [button(-600, 2048), button(600, 1024)];
        let mut title = Title {
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
        };
        title.draw();
        Ok(title)
    }

    /// Advances to tick `now`: a frame every other tick.
    pub fn tick(&mut self, now: u64) {
        if (now - self.started) % 2 != 0 {
            return;
        }
        // A fade every 30 frames (0x8003da00), stepped by the menu's draw.
        if self.frames % 30 == 0 {
            self.backdrop.fading = true;
        }
        self.frames += 1;
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        for t in &mut self.items {
            if t.flags & text::SLIDE_Y != 0 {
                t.step();
            }
        }
        self.sign.step();
        for b in &mut self.buttons {
            b.step();
        }
        self.draw();
    }

    fn draw(&mut self) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.moving.clear();
        rugrats_render::models::model_triangles(&self.sign.model, rrt::glam::Mat3::IDENTITY, rrt::glam::Vec3::ZERO, &mut self.stage.moving);
        self.stage.overlay.clear();
        menu_triangles(&self.fonts, &self.items, self.chosen, &mut self.stage.overlay);
        trademark(&mut self.stage.overlay);
        for t in &self.labels {
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(&self.fonts, &mut self.stage.overlay);
            t.triangles(&self.fonts, &mut self.stage.overlay);
        }
        // The cubes are seen by their own camera, which differs from the
        // main one only in where it stands: moved by the difference, they
        // land where the main camera shows them the same.
        let shift = rrt::glam::Vec3::from(CAMERA) - rrt::glam::Vec3::from(BUTTON_CAMERA);
        for b in &self.buttons {
            let m = self.trig.euler(b.angles);
            let rows = rrt::glam::Mat3::from_cols_array(&m.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap()).transpose();
            let mut tris = Vec::new();
            let pos = rrt::glam::Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
            rugrats_render::models::model_triangles(&self.button_model, rows.transpose(), pos, &mut tris);
            self.stage.moving.extend(tris);
        }
    }
}
