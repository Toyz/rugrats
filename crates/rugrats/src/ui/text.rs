//! The game's text: three fonts in PHOUSE.FNT (file 106), which boot
//! uploads to VRAM (960, 256) (SLUS_006.50:0x8002ed80), drawn a glyph a
//! sprite (0x8002e67c), laid out and centred by 0x8002e144, and slid into
//! place by 0x8002e2b4.

use rugrats_render::Vtx;

use crate::disc::GameDisc;

/// The font image's file, where it goes in VRAM, and its size in VRAM
/// words.
pub const FONT_FILE: usize = 106;
pub const FONT_AT: (u16, u16) = (960, 256);
pub const FONT_SIZE: (u16, u16) = (64, 213);
/// The CLUT every font draws with ((960, 468), inside the image).
pub const CLUT: u16 = (468 << 6) | (960 >> 4);
/// The texture page: (960, 256), 4-bit, dithered (0x200).
pub const PAGE: u16 = 0x200 | (256 >> 4) | (960 >> 6);
/// The glyph table (0x800b78c8): per font 91 glyphs (' ' to 'z'), each u,
/// v, width, and how far below the line its top sits.
const GLYPHS: u32 = 0x800b_78c8;
const PER_FONT: usize = 91;

/// The fonts' line heights.
pub const LINE: [i16; 3] = [16, 24, 12];

/// A text item as the game keeps it (44 bytes).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Text {
    pub font: u8,
    /// Pixels between glyphs.
    pub spacing: i16,
    pub rgb: [u8; 3],
    pub x: i16,
    pub y: i16,
    /// Where a slide is heading (+0x0c, +0x0e) and where it settles
    /// (+0x10, +0x12).
    pub target: (i16, i16),
    pub rest: (i16, i16),
    pub string: String,
    /// +0x18: 2 slide in x, 4 slide in y, 8 bounce, 16 wobble, 32 wait.
    pub flags: u16,
    /// +0x1a: which half of a bounce or wobble.
    pub toggle: i16,
    /// Pixels a frame (+0x20 x, +0x22 y).
    pub speed: (i16, i16),
    /// A bounce's height or a wobble's count (+0x24 x, +0x26 y).
    pub count: (i16, i16),
    /// Frames to wait (+0x28).
    pub wait: i16,
}

pub const SLIDE_X: u16 = 2;
pub const SLIDE_Y: u16 = 4;
pub const BOUNCE: u16 = 8;
pub const WOBBLE: u16 = 16;
pub const WAIT: u16 = 32;

/// How far a wobble pulls back (0x800c3f6c).
pub const WOBBLE_BY: i16 = 3;

/// One glyph on screen, in the PlayStation's drawing coordinates (the
/// screen's centre at 0, 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub u: u8,
    pub v: u8,
    pub rgb: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct Fonts {
    glyphs: Vec<[u8; 4]>,
}

impl Fonts {
    pub fn load(disc: &GameDisc) -> Result<Fonts, String> {
        let raw = disc.exe_bytes(GLYPHS, 4 * 3 * PER_FONT)?;
        Ok(Fonts { glyphs: raw.as_chunks::<4>().0.iter().map(|g| [g[0], g[1], g[2], g[3]]).collect() })
    }

    fn glyph(&self, font: u8, index: usize) -> [u8; 4] {
        self.glyphs.get(font as usize * PER_FONT + index).copied().unwrap_or([0; 4])
    }

    /// Centres `t` on its position (0x8002e144): across with flag 1, down
    /// with flag 2. A line's width is its glyphs' widths, the gaps between
    /// all the text's glyphs shared out; a new line counts as glyph 10 of
    /// the next, as the original's loop does.
    pub fn centre(&self, t: &mut Text, across: bool, down: bool) {
        let (mut line, mut widest, mut glyphs, mut lines) = (0i32, 0i32, 0i32, 1i32);
        for c in t.string.bytes() {
            let index = if c == b'\n' {
                lines += 1;
                widest = widest.max(line);
                line = 0;
                10
            } else {
                c.wrapping_sub(32) as usize
            };
            if index < PER_FONT {
                line += self.glyph(t.font, index)[2] as i32;
                glyphs += 1;
            }
        }
        widest = widest.max(line);
        let half_width = (widest + (glyphs - 1) * t.spacing as i32) >> 1;
        let height = lines * LINE[t.font as usize % 3] as i32;
        if across {
            t.x = (t.x as i32 - half_width) as i16;
            t.target.0 = t.x;
        }
        if down {
            t.y = (t.y as i32 - (height >> 1)) as i16;
            t.target.1 = t.y;
        }
    }

    /// The glyphs of `t` (0x8002e67c): a space moves on half a line's
    /// height, a new line goes back to the item's x.
    pub fn sprites(&self, t: &Text) -> Vec<Sprite> {
        let h = LINE[t.font as usize % 3];
        let (mut x, mut y) = (t.x, t.y);
        let mut out = Vec::new();
        for c in t.string.bytes() {
            match c {
                b'\n' => {
                    x = t.x;
                    y += h;
                }
                b' ' => x += h >> 1,
                c => {
                    let [u, v, w, drop] = self.glyph(t.font, c.wrapping_sub(32) as usize);
                    out.push(Sprite { x, y: y + drop as i16, w: w as i16, h, u, v, rgb: t.rgb });
                    x += w as i16 + t.spacing;
                }
            }
        }
        out
    }
}

impl Text {
    /// One frame of its slide (0x8002e2b4).
    pub fn step(&mut self) {
        if self.flags & WAIT != 0 {
            if self.wait != 0 {
                self.wait -= 1;
            } else {
                self.flags &= !WAIT;
            }
            return;
        }
        // The y loop tests the flag as it was when the frame began; both
        // loops read the speed afresh, a wobble setting it to 2.
        let slide_y = self.flags & SLIDE_Y != 0;
        let mut k = 0;
        while k < self.speed.1 {
            if slide_y {
                if self.y < self.target.1 {
                    self.y += 1;
                } else if self.y > self.target.1 {
                    self.y -= 1;
                } else {
                    let (rest, at) = (self.rest.1, self.y);
                    settle(
                        &mut self.flags,
                        SLIDE_Y,
                        &mut self.toggle,
                        &mut self.target.1,
                        rest,
                        &mut self.speed.1,
                        &mut self.count.1,
                        at,
                    );
                }
            }
            k += 1;
        }
        let mut k = 0;
        while k < self.speed.0 {
            if self.flags & SLIDE_X != 0 {
                if self.x < self.target.0 {
                    self.x += 1;
                }
                if self.x > self.target.0 {
                    self.x -= 1;
                }
                if self.x == self.target.0 {
                    let (rest, at) = (self.rest.0, self.x);
                    settle(
                        &mut self.flags,
                        SLIDE_X,
                        &mut self.toggle,
                        &mut self.target.0,
                        rest,
                        &mut self.speed.0,
                        &mut self.count.0,
                        at,
                    );
                }
            }
            k += 1;
        }
    }

    /// The glyphs as triangles in screen pixels (the drawing offset 256,
    /// 120 added), textured from the font page.
    pub fn triangles(&self, fonts: &Fonts, out: &mut Vec<Vtx>) {
        for s in fonts.sprites(self) {
            sprite_triangles(&s, out);
        }
    }
}

/// Arrived where a slide was heading: done, or the next half of a bounce
/// (a third as high each time) or of a wobble (back by [`WOBBLE_BY`], then
/// to rest, `count` times, at 2 a frame).
#[allow(clippy::too_many_arguments)]
fn settle(
    flags: &mut u16,
    slide: u16,
    toggle: &mut i16,
    target: &mut i16,
    rest: i16,
    speed: &mut i16,
    count: &mut i16,
    at: i16,
) {
    if *flags & (BOUNCE | WOBBLE) == 0 {
        *flags &= !slide;
    } else if *flags & BOUNCE != 0 {
        if *toggle == 0 {
            *toggle = 1;
            *target += *count / 3;
            *count /= 3;
        } else {
            *toggle = 0;
            *target = rest;
        }
        if *target == rest && at == *target {
            *flags &= !BOUNCE;
        }
    } else {
        *speed = 2;
        if *toggle == 0 {
            *toggle = 1;
            *target -= WOBBLE_BY;
            *count -= 1;
        } else {
            *toggle = 0;
            *target = rest;
        }
        if *count == 0 {
            *flags &= !WOBBLE;
        }
    }
}

/// A textured sprite (GPU 0x64: modulated, opaque) as two triangles. The
/// GPU wraps u within the page, so a sprite running past u 255 (font 0's
/// "r" at 245, 12 wide) is drawn as two, the rest from u 0.
pub fn sprite_triangles(s: &Sprite, out: &mut Vec<Vtx>) {
    let end = s.u as i16 + s.w;
    if end > 256 {
        let first = 256 - s.u as i16;
        sprite_triangles(&Sprite { w: first, ..*s }, out);
        sprite_triangles(&Sprite { x: s.x + first, w: end - 256, u: 0, ..*s }, out);
        return;
    }
    let (x0, y0) = (s.x as f32 + 256.0, s.y as f32 + 120.0);
    let (x1, y1) = (x0 + s.w as f32, y0 + s.h as f32);
    let (u0, v0) = (s.u as u32, s.v as u32);
    let (u1, v1) = (u0 + s.w as u32, v0 + s.h as u32);
    let colour = s.rgb[0] as u32 | (s.rgb[1] as u32) << 8 | (s.rgb[2] as u32) << 16;
    let window = Vtx::window_of([u0 as u8, u1.min(255) as u8, 0, 0], [v0 as u8, v1.min(255) as u8, 0, 0]);
    // A sprite reaches its last texel (unlike a polygon), so the window
    // runs to u1 - 1, v1 - 1, which window_of gives.
    let window = (window & 0x0000_ffff) | (((u1 - 1).min(255)) << 16) | (((v1 - 1).min(255)) << 24);
    let vtx = |x: f32, y: f32, u: u32, v: u32| Vtx {
        pos: [x, y, 0.0],
        colour,
        uv: Vtx::uv_of(u, v),
        mode: CLUT as u32 | (PAGE as u32) << 16,
        window,
    };
    let (a, b, c, d) = (vtx(x0, y0, u0, v0), vtx(x1, y0, u1, v0), vtx(x0, y1, u0, v1), vtx(x1, y1, u1, v1));
    out.extend([a, b, c, b, d, c]);
}
