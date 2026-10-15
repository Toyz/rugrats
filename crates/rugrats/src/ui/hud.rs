//! The HUD's pieces (SLUS_006.50:0x8008f8f4, drawn by 0x8008f5d8): a
//! texture from the database's TEXPOS.DAT (file 0x800b9b20[db + 1], 428
//! 12-byte entries, looked up by 0x80089fa4 / 0x800895ac) on a quad of
//! half-size (w, h) at depth z, turned by an angle, its x scaled for the
//! 512-wide screen (0x800c4a04 / 320), projected by the GTE and moved to
//! a screen position.
//!
//! The health meter (0x8008ba70, drawn by 0x8008bdf4): a frame (sprite
//! 258, 32 x 24) over a fill (260, 32 x 60) at depth 768, at (192, 76)
//! from the centre (0x800c45bc); the fill's top comes down as health
//! falls. It shows for 1500 dt after a change.

use rugrats_data::level::Texture;
use rugrats_render::Vtx;

use crate::disc::GameDisc;

const TEXPOS: u32 = 0x800b_9b20;
const ENTRIES: usize = 428;

pub struct Sprites {
    textures: Vec<Texture>,
}

impl Sprites {
    pub fn load(disc: &GameDisc, db: usize) -> Result<Sprites, String> {
        // 0x8008a328 is called with db + 1; entry 0 is the menu's.
        let k = if db < 32 { db + 1 } else { 0 };
        let file = disc.exe_u32(TEXPOS + 4 * k as u32)? as usize;
        let raw = disc.file(file)?;
        Ok(Sprites { textures: raw.chunks_exact(12).take(ENTRIES).map(Texture::parse).collect() })
    }

    /// Sprite `n` (0x80089fa4: n mod 428; an entry with no page is none).
    pub fn texture(&self, n: u32) -> Option<&Texture> {
        self.textures.get(n as usize % ENTRIES).filter(|t| t.tpage != 0)
    }

    /// Copies sprite `n`'s texels and CLUT from `from` into `to` (a screen
    /// over another VRAM keeps the level's icon).
    pub fn copy(&self, n: u32, from: &rugrats_render::Vram, to: &mut rugrats_render::Vram) {
        let Some(t) = self.texture(n) else { return };
        let (px, py) = ((t.tpage as usize & 15) * 64, ((t.tpage as usize >> 4) & 1) * 256);
        let depth = (t.tpage as usize >> 7) & 3;
        let (us, vs) = (t.uv.iter().map(|uv| uv[0] as usize), t.uv.iter().map(|uv| uv[1] as usize));
        let (u0, u1) = (us.clone().min().unwrap_or(0), us.max().unwrap_or(0));
        let (v0, v1) = (vs.clone().min().unwrap_or(0), vs.max().unwrap_or(0));
        // Texels a VRAM word: 4 (4-bit), 2 (8-bit), 1 (16-bit).
        let per = [4, 2, 1, 1][depth];
        let (x0, x1) = (px + u0 / per, px + u1 / per);
        let w = |x: usize, y: usize| from.words[y * 1024 + x];
        let rows: Vec<u16> =
            (py + v0..=py + v1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).map(|(x, y)| w(x, y)).collect();
        to.load(x0 as u16, (py + v0) as u16, (x1 - x0 + 1) as u16, (v1 - v0 + 1) as u16, &rows);
        let (cx, cy) = ((t.clut as usize & 63) * 16, (t.clut as usize >> 6) & 511);
        let n = [16, 256, 0, 0][depth];
        if n > 0 {
            let clut: Vec<u16> = (cx..cx + n).map(|x| w(x, cy)).collect();
            to.load(cx as u16, cy as u16, n as u16, 1, &clut);
        }
    }

    /// Sprite `n` as a quad in the world facing the camera: centre `at`,
    /// half-size `w` x `h` along the camera's `right` and `down`.
    pub fn billboard(
        &self,
        n: u32,
        at: rrt::glam::Vec3,
        w: f32,
        h: f32,
        right: rrt::glam::Vec3,
        down: rrt::glam::Vec3,
        out: &mut Vec<Vtx>,
    ) {
        let Some(t) = self.texture(n) else { return };
        let mode = t.clut as u32 | (t.tpage as u32) << 16;
        let window = Vtx::window_of(
            [t.uv[0][0], t.uv[1][0], t.uv[2][0], t.uv[3][0]],
            [t.uv[0][1], t.uv[1][1], t.uv[2][1], t.uv[3][1]],
        );
        let corners = [(-w, -h), (w, -h), (-w, h), (w, h)];
        let v = |k: usize| {
            let p = at + right * corners[k].0 + down * corners[k].1;
            Vtx {
                pos: p.to_array(),
                colour: 0x80_8080,
                uv: Vtx::uv_of(t.uv[k][0] as u32, t.uv[k][1] as u32),
                mode,
                window,
            }
        };
        // Wound for the world pass, which culls back faces.
        out.extend([v(0), v(2), v(1), v(1), v(2), v(3)]);
    }

    /// Sprite `n` at screen `at` (from the centre), half-size `w` x `h` at
    /// depth `z`, seen at projection `focal`; `top` lowers its top edge by
    /// that much (a meter's fill).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, n: u32, at: (i16, i16), w: f32, h: f32, z: f32, top: f32, focal: f32, out: &mut Vec<Vtx>) {
        let Some(t) = self.texture(n) else { return };
        let sx = focal / z * (512.0 / 320.0);
        let sy = focal / z;
        let (cx, cy) = (at.0 as f32 + 256.0, at.1 as f32 + 120.0);
        // Corners in 0x8008f808's order: (-w, -h), (w, -h), (-w, h), (w, h).
        let corners = [(-w, -h + top), (w, -h + top), (-w, h), (w, h)];
        let mode = t.clut as u32 | (t.tpage as u32) << 16;
        let window = Vtx::window_of(
            [t.uv[0][0], t.uv[1][0], t.uv[2][0], t.uv[3][0]],
            [t.uv[0][1], t.uv[1][1], t.uv[2][1], t.uv[3][1]],
        );
        let v = |k: usize| {
            let (x, y) = corners[k];
            // The fill's texture is cut as its top comes down.
            let (u, mut vv) = (t.uv[k][0] as u32, t.uv[k][1] as u32);
            if k < 2 && top > 0.0 {
                let full = (t.uv[2][1] as f32 - t.uv[0][1] as f32).abs();
                vv = (vv as f32 + full * top / (2.0 * h)).round() as u32;
            }
            Vtx { pos: [cx + x * sx, cy + y * sy, 0.0], colour: 0x80_8080, uv: Vtx::uv_of(u, vv), mode, window }
        };
        out.extend([v(0), v(1), v(2), v(1), v(3), v(2)]);
    }
}

/// The health meter's sprites and place (0x8008ba70).
pub const METER_FRAME: u32 = 258;
pub const METER_FILL: u32 = 260;
pub const METER_AT: (i16, i16) = (192, 76);
pub const METER_DEPTH: f32 = 768.0;
/// How long it shows after a change (dt).
pub const METER_TIME: i32 = 1500;

/// The meter for `health` of `max` (0x8008bdf4: the fill's top lowered by
/// 2 h (max - health) / max).
pub fn meter(s: &Sprites, health: i32, max: i32, focal: f32, out: &mut Vec<Vtx>) {
    let top = if max > 0 { 2.0 * 60.0 * (max - health).max(0) as f32 / max as f32 } else { 0.0 };
    s.draw(METER_FILL, METER_AT, 32.0, 60.0, METER_DEPTH, top, focal, out);
    // The bottle's top over its body. Where 0x8008fa9c puts each piece is
    // not read yet: the cap sits on the body here.
    let up = ((60.0 + 24.0) * focal / METER_DEPTH) as i16;
    s.draw(METER_FRAME, (METER_AT.0, METER_AT.1 - up), 32.0, 24.0, METER_DEPTH, 0.0, focal, out);
}

/// A gauge's back (0x8008c5e0's last sprite, 282 in level 2).
pub const GAUGE_BACK: u32 = 282;

/// A gauge (0x8008c5e0, drawn by 0x8008c394): its back, then its fill
/// sprite, half-size 64 x 64 at depth 1024, at `at`; the fill's top
/// lowered as `value` falls from `max`.
pub fn gauge(s: &Sprites, fill: u32, at: (i16, i16), value: i32, max: i32, focal: f32, out: &mut Vec<Vtx>) {
    let top = if max > 0 { 2.0 * 64.0 * (max - value).clamp(0, max) as f32 / max as f32 } else { 0.0 };
    s.draw(GAUGE_BACK, at, 64.0, 64.0, 1024.0, 0.0, focal, out);
    s.draw(fill, at, 64.0, 64.0, 1024.0, top, focal, out);
}
