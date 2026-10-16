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
//!
//! The animated sprites ([`Sequences`], 0x800899c8 and its wrappers): a
//! running sequence keeps its own copy of the TEXPOS entry it shows and
//! changes it as its frames come round.

use rrt::glam::Vec3;
use rugrats_data::level::Texture;
use rugrats_render::{Vram, Vtx};

use crate::disc::GameDisc;

const TEXPOS: u32 = 0x800b_9b20;
const ENTRIES: usize = 428;

pub struct Sprites {
    /// Every entry in the file (428 in each database's).
    textures: Vec<Texture>,
}

/// How a world sprite is coloured: its texels scaled by `grey` (128 as
/// they are), and blended by its page when `semi` (packet bit 25, the
/// face flags' 0x02000000).
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub grey: u8,
    pub semi: bool,
}

impl Default for Look {
    fn default() -> Look {
        Look { grey: 128, semi: false }
    }
}

impl Sprites {
    pub fn load(disc: &GameDisc, db: usize) -> Result<Sprites, String> {
        // 0x8008a328 is called with db + 1; entry 0 is the menu's.
        let k = if db < 32 { db + 1 } else { 0 };
        let file = disc.exe_u32(TEXPOS + 4 * k as u32)? as usize;
        let raw = disc.file(file)?;
        Ok(Sprites { textures: raw.as_chunks::<12>().0.iter().map(|t| Texture::parse(t)).collect() })
    }

    /// Sprite `n` (0x80089fa4: n mod 428; an entry with no page is none).
    pub fn texture(&self, n: u32) -> Option<&Texture> {
        self.textures.get(n as usize % ENTRIES).filter(|t| t.tpage != 0)
    }

    /// Entry `n` as 0x800895ac takes it: no wrap (a sequence's frames, a
    /// flight's sprite). An entry with no page is the menu's (0x800b9ba4),
    /// which is not kept here: none.
    pub fn entry(&self, n: u32) -> Option<&Texture> {
        self.textures.get(n as usize).filter(|t| t.tpage != 0)
    }

    /// Copies sprite `n`'s texels and CLUT from `from` into `to` (a screen
    /// over another VRAM keeps the level's icon).
    pub fn copy(&self, n: u32, from: &Vram, to: &mut Vram) {
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
        self.quad(n, at, (right * w, down * h), Look::default(), out);
    }

    /// Sprite `n` as a quad in the world: centre `at`, its texture's u
    /// running along `u` and v along `v` (half extents), as 0x8008a954
    /// and 0x8008e508 draw the sprite quad (+-4096 scaled, turned, D_800b6da8).
    /// Front facing when u x v points away from the eye.
    pub fn quad(
        &self,
        n: u32,
        at: rrt::glam::Vec3,
        axes: (rrt::glam::Vec3, rrt::glam::Vec3),
        look: Look,
        out: &mut Vec<Vtx>,
    ) {
        let Some(t) = self.texture(n) else { return };
        let flags = if look.semi { FACE_SEMI } else { 0 };
        face(t, at, axes, [look.grey; 3], flags, out);
    }

    /// Sprite `n` at its own size in pixels, centred on screen point `at`
    /// (from the top left), its texels times `rgb` / 128 (0x80031db8's
    /// POLY_FT4: u from u0 - w / 2 to u0 + w / 2 - 1).
    pub fn at_pixels(&self, n: u32, at: (f32, f32), rgb: [u8; 3], out: &mut Vec<Vtx>) {
        let Some(t) = self.texture(n) else { return };
        let (w, h) = ((t.uv[1][0] as f32 - t.uv[0][0] as f32).abs(), (t.uv[2][1] as f32 - t.uv[0][1] as f32).abs());
        let (x0, y0) = ((at.0 - (w / 2.0).trunc()).trunc(), (at.1 - (h / 2.0).trunc()).trunc());
        let corners = [(x0, y0), (x0 + w, y0), (x0, y0 + h), (x0 + w, y0 + h)];
        let mode = t.clut as u32 | (t.tpage as u32) << 16;
        let window = Vtx::window_of(
            [t.uv[0][0], t.uv[1][0], t.uv[2][0], t.uv[3][0]],
            [t.uv[0][1], t.uv[1][1], t.uv[2][1], t.uv[3][1]],
        );
        let colour = u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0]);
        let v = |k: usize| Vtx {
            pos: [corners[k].0, corners[k].1, 0.0],
            colour,
            uv: Vtx::uv_of(t.uv[k][0] as u32, t.uv[k][1] as u32),
            mode,
            window,
        };
        out.extend([v(0), v(1), v(2), v(1), v(3), v(2)]);
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

/// A face flag (packet bit 25): the polygon is semi-transparent.
pub const FACE_SEMI: u32 = 0x0200_0000;
/// The face flags' bits that are added to the page's blend mode
/// (0x800233f4: the packet's tpage | flags & 0x600000).
pub const FACE_BLEND: u32 = 0x0060_0000;

/// Texture `t` on a quad in the world, as 0x8008a954 sends a
/// camera-facing one through 0x8002390c: centre `at`, the texture's u
/// along `u` and v along `v` (half extents), its texels times `rgb` / 128;
/// face flags [`FACE_SEMI`] make it semi-transparent and [`FACE_BLEND`]
/// goes into the page's blend. Front facing when u x v points away from
/// the eye.
pub fn face(t: &Texture, at: Vec3, (u, v): (Vec3, Vec3), rgb: [u8; 3], flags: u32, out: &mut Vec<Vtx>) {
    let semi = if flags & FACE_SEMI != 0 { Vtx::SEMI } else { 0 };
    let mode = t.clut as u32 | (t.tpage as u32) << 16 | (flags & FACE_BLEND) | semi;
    let window = Vtx::window_of(
        [t.uv[0][0], t.uv[1][0], t.uv[2][0], t.uv[3][0]],
        [t.uv[0][1], t.uv[1][1], t.uv[2][1], t.uv[3][1]],
    );
    // Corners in 0x8008a954's order: (-w, -h), (w, -h), (-w, h), (w, h).
    let corners = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
    let colour = u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0]);
    let vtx = |k: usize| {
        let p = at + u * corners[k].0 + v * corners[k].1;
        Vtx { pos: p.to_array(), colour, uv: Vtx::uv_of(t.uv[k][0] as u32, t.uv[k][1] as u32), mode, window }
    };
    // Wound for the world pass, which culls back faces.
    out.extend([vtx(0), vtx(2), vtx(1), vtx(1), vtx(2), vtx(3)]);
}

/// The sequences' file (DB00/TEXACTRL.DAT, loaded once by 0x8008946c and
/// made absolute by 0x800893a8): a size, then a count and a table of
/// offsets for each of two kinds (0x1000 and 0x2000 ids; the second is
/// empty in the game's file).
const TEXACTRL: usize = 551;
/// The running sequences' pool (0x8008a3d8(32), 0x8008931c: 44 bytes
/// each, their textures listed at 0x800e6690).
pub const SEQUENCE_SLOTS: usize = 32;

/// A sequence (a 0x1000 id): each frame `time` ms (+0x00), looping or not
/// (+0x02), its last frame (+0x03) and the TEXPOS entries it shows (+0x04).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence {
    pub time: i32,
    pub looping: bool,
    pub frames: Vec<u16>,
}

/// A running sequence (a pool record): its id (+0x08), how many hold it
/// (+0x18), whether it is on the stepped list (0x800c4d18, flag 0x4000),
/// the time left in its frame (+0x20), the frame (+0x24) and the entry it
/// shows (+0x0c).
#[derive(Clone, Debug)]
struct Running {
    id: u16,
    sequence: usize,
    refs: i32,
    active: bool,
    /// When it last joined a list: the lists are newest first.
    joined: u32,
    timer: i32,
    frame: usize,
    texture: Option<Texture>,
}

/// The animated sprites: the sequences and the pool that runs them.
#[derive(Clone, Debug, Default)]
pub struct Sequences {
    sequences: Vec<Sequence>,
    slots: Vec<Option<Running>>,
    clock: u32,
}

impl Sequences {
    pub fn load(disc: &GameDisc) -> Result<Sequences, String> {
        Ok(Sequences::parse(&disc.file(TEXACTRL)?))
    }

    /// TEXACTRL.DAT's 0x1000 sequences.
    pub fn parse(raw: &[u8]) -> Sequences {
        let word = |at: usize| raw.get(at..at + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])) as usize;
        let half = |at: usize| raw.get(at..at + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
        let (count, table) = (word(4), word(8));
        let sequences = (0..count)
            .map(|k| {
                let at = word(table + 4 * k);
                let last = raw.get(at + 3).copied().unwrap_or(0) as usize;
                let frames = word(at + 4);
                Sequence {
                    time: half(at) as i32,
                    looping: raw.get(at + 2).is_some_and(|&b| b != 0),
                    frames: (0..=last).map(|f| half(frames + 2 * f)).collect(),
                }
            })
            .collect();
        Sequences { sequences, slots: vec![None; SEQUENCE_SLOTS], clock: 0 }
    }

    /// The sequence an id names (0x800897a8: 0x1000 | index).
    pub fn sequence(&self, id: u16) -> Option<&Sequence> {
        (id & 0x3000 == 0x1000).then(|| self.sequences.get((id & 0xfff) as usize)).flatten()
    }

    fn tick(&mut self) -> u32 {
        self.clock += 1;
        self.clock
    }

    /// Starts sequence `id` on a free record (0x8008a22c, 0x800899c8 with
    /// a start of 0): held once, not yet stepped, on its first frame. None
    /// without the sequence or a free record.
    pub fn start(&mut self, id: u16, sprites: Option<&Sprites>) -> Option<usize> {
        let k = self.sequence(id).map(|_| (id & 0xfff) as usize)?;
        let slot = self.slots.iter().position(Option::is_none)?;
        let first = self.sequences[k].frames.first().copied();
        let texture = first.and_then(|n| sprites.and_then(|s| s.entry(n as u32))).cloned();
        let joined = self.tick();
        self.slots[slot] = Some(Running {
            id: id & 0x3fff,
            sequence: k,
            refs: 1,
            active: false,
            joined,
            timer: self.sequences[k].time,
            frame: 0,
            texture,
        });
        Some(slot)
    }

    /// The record already running `id`, held once more (0x8008a2a4: the
    /// stepped list first, then the rest, each newest first), or a new one.
    pub fn start_shared(&mut self, id: u16, sprites: Option<&Sprites>) -> Option<usize> {
        let id = id & 0x3fff;
        let find = |active: bool| {
            self.slots
                .iter()
                .enumerate()
                .filter_map(|(k, r)| r.as_ref().map(|r| (k, r)))
                .filter(|(_, r)| r.id == id && r.active == active)
                .max_by_key(|(_, r)| r.joined)
                .map(|(k, _)| k)
        };
        if let Some(k) = find(true).or_else(|| find(false)) {
            if let Some(r) = self.slots[k].as_mut() {
                r.refs += 1;
            }
            return Some(k);
        }
        self.start(id, sprites)
    }

    /// Puts record `slot` on the stepped list (0x8008a154, 0x80089624).
    pub fn activate(&mut self, slot: usize) {
        let joined = self.tick();
        if let Some(r) = self.slots.get_mut(slot).and_then(Option::as_mut)
            && !r.active
        {
            r.active = true;
            r.joined = joined;
        }
    }

    /// Lets go of record `slot` (0x8008a1b8, 0x8008976c): the last holder
    /// frees it.
    pub fn release(&mut self, slot: usize) {
        let Some(entry) = self.slots.get_mut(slot) else { return };
        match entry {
            Some(r) if r.refs == 1 => *entry = None,
            Some(r) => r.refs -= 1,
            None => {}
        }
    }

    /// One frame of `ms` (0x8008a358, 0x80089f20, 0x80089bb8): each stepped
    /// record's time runs down; at below 0 it gets a frame's time back and
    /// moves on a frame, round to the first when it loops; a sequence that
    /// ends goes off the stepped list showing its last frame.
    pub fn step(&mut self, ms: i32, sprites: Option<&Sprites>) {
        let mut ended = Vec::new();
        for (k, slot) in self.slots.iter_mut().enumerate() {
            let Some(r) = slot.as_mut().filter(|r| r.active) else { continue };
            let s = &self.sequences[r.sequence];
            let last = s.frames.len().saturating_sub(1);
            r.timer -= ms;
            let (mut frame, mut moved, mut done) = (r.frame, false, false);
            while r.timer < 0 {
                r.timer += s.time;
                // 0x80089b8c: the next frame, the first again, or the end.
                let next = if frame != last {
                    Some(frame + 1)
                } else if s.looping {
                    Some(0)
                } else {
                    None
                };
                let Some(next) = next else {
                    done = true;
                    break;
                };
                frame = next;
                moved = true;
            }
            if moved {
                r.frame = frame;
                r.texture = s.frames.get(frame).and_then(|&n| sprites.and_then(|t| t.entry(n as u32))).cloned();
            }
            if done {
                ended.push(k);
            }
        }
        for k in ended {
            let joined = self.tick();
            if let Some(r) = self.slots[k].as_mut() {
                r.active = false;
                r.joined = joined;
            }
        }
    }

    /// The entry record `slot` shows (0x8008a018's 0x800e6690[slot]).
    pub fn texture(&self, slot: usize) -> Option<&Texture> {
        self.slots.get(slot).and_then(Option::as_ref).and_then(|r| r.texture.as_ref())
    }

    /// Record `slot`'s frame and the time left in it, if it is running.
    pub fn frame(&self, slot: usize) -> Option<(usize, i32)> {
        self.slots.get(slot).and_then(Option::as_ref).map(|r| (r.frame, r.timer))
    }

    /// How many records are in use.
    pub fn running(&self) -> usize {
        self.slots.iter().flatten().count()
    }
}

/// The screen's fade (0x8001cf2c): a POLY_F4 over the whole screen,
/// semi-transparent with draw mode 0x640 (blend 2: the screen less
/// `grey`), so 255 is black.
pub fn fade(grey: u8, out: &mut Vec<Vtx>) {
    let g = grey as u32;
    let v = |x: f32, y: f32| Vtx {
        pos: [x, y, 0.0],
        colour: g | g << 8 | g << 16,
        uv: 0,
        mode: Vtx::FLAT | Vtx::SEMI | 2 << 21,
        window: 0,
    };
    out.extend([v(0.0, 0.0), v(512.0, 0.0), v(0.0, 240.0), v(512.0, 0.0), v(512.0, 240.0), v(0.0, 240.0)]);
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
