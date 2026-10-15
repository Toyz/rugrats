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

/// The title menu as it stands (0x8003d884).
pub struct Title {
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
        let strings = ITEM_STRINGS.map(|a| disc.exe_string_at(a).unwrap_or_default());
        let items = menu_items(&fonts, strings);
        let mut title = Title {
            stage: crate::stage::Stage::new(vram, Vec::new()),
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
        self.draw();
    }

    fn draw(&mut self) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        self.stage.overlay.clear();
        menu_triangles(&self.fonts, &self.items, self.chosen, &mut self.stage.overlay);
    }
}
