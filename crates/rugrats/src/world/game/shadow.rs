//! The characters' shadows (SLUS_006.50:0x8003414c, called from the
//! character draw 0x8003205c): TEXPOS entry 261 on a quad laid flat at the
//! floor under the body, subtracted from what is drawn under it.
//!
//! The quad is turned by the body's facing as its first part shows it and
//! sized to six of its parts' joints, at least 128 behind and 164 ahead.
//! The draw leaves it out for a body that sinks into its floor, one riding
//! another (+0x00 0x80000) and while level 3's cine 15 runs (+0x30 bit
//! 128). The opening cine's init (0x8008f078) turns all shadows off
//! (0x800c3fc4) and its end (0x8008f308) back on; the other cines draw
//! through the same render and have them.

use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::level::Texture;
use rugrats_render::Vtx;

use super::Game;
use crate::actors::pose::PartMatrix;
use crate::gte::Trig;
use crate::world::collide;

/// The slots whose parts size the quad (0x80012ff0: header offsets, each
/// part at +0x14 past it).
const SLOTS: [usize; 6] = [0x10, 0xe, 0xb, 6, 2, 0x11];
/// The shadow's TEXPOS entry (0x800c4484 + 3132): a 64-texel blob.
pub const SPRITE: u32 = 261;
/// Its texels times 128 / 128 (0x800c4060..62).
const GREY: u8 = 128;
/// Its page's blend: subtracted (the page | 64, mode 2).
const SUBTRACT: u16 = 64;
/// The least it reaches behind and ahead of the body.
const BEHIND: i32 = -128;
const AHEAD: i32 = 164;

impl Game {
    /// The floor a shadow lies on (0x8006c118): the top of the nearest
    /// object between the body's place and its floor (+0x13c) by a
    /// segment down (0x8006a9cc on its room's two lists, an object of
    /// template flag 16 never, a character not), else that floor.
    pub(super) fn shadow_floor(&self, at: [i32; 3], floor: i32) -> i32 {
        let below = [at[0], floor, at[2]];
        let mut nearest: Option<(i32, i32)> = None;
        for (e, n) in self.solids.iter().zip(&self.npc_of) {
            let flags = self.set.templates.get(e.template as usize).map_or(0, |t| t.flags);
            if n.is_some() || flags & 16 != 0 {
                continue;
            }
            let mut normal = [0; 3];
            if let Some((_, hit)) = e.solid.meet(&self.trig, at, below, &mut normal)
                && hit.t >= 0
                && nearest.is_none_or(|(t, _)| hit.t < t)
            {
                nearest = Some((hit.t, hit.point[1]));
            }
        }
        nearest.map_or(floor, |(_, y)| y)
    }

    /// The player's shadow's quad as this frame would draw it (for tools).
    pub fn player_shadow(&self) -> Option<Quad> {
        let at = self.player.pos;
        let floor = self.shadow_floor(at, tile_floor(&self.col, at, self.player.floor));
        quad(&self.trig, &self.player.hierarchy, &self.player.posed(&self.anims, &self.trig).0, floor)
    }

    /// The shadow of a body posed as `parts` (by `hierarchy`'s slots) on
    /// the floor at `floor`.
    pub(super) fn shadow(&self, hierarchy: &Hierarchy, parts: &[PartMatrix], floor: i32, out: &mut Vec<Vtx>) {
        let Some(t) = self.sprites.as_ref().and_then(|s| s.texture(SPRITE)) else { return };
        if let Some(q) = quad(&self.trig, hierarchy, parts, floor) {
            q.triangles(t, out);
        }
    }
}

/// The shadow's quad for a body posed as `parts` (by `hierarchy`'s slots)
/// over the floor at `floor`.
pub fn quad(trig: &Trig, hierarchy: &Hierarchy, parts: &[PartMatrix], floor: i32) -> Option<Quad> {
    let part = |slot: usize| hierarchy.header.get(0x14 + slot).filter(|&&p| p != 0xff).and_then(|&p| parts.get(p as usize));
    let hip = part(0)?;
    // Its facing: the first part's third row across, or its second back
    // when that row stands more upright (|+0xe| not under |+0x8|).
    let m = hip.rot;
    let d = if m[2][1].abs() < m[1][1].abs() { [m[2][0], 0, m[2][2]] } else { [-m[1][0], 0, -m[1][2]] };
    let d = trig.normal(d);
    let q = Quad { facing: [d[0], d[2]], lo: [0x7fff; 3], hi: [-0x8000; 3], at: [hip.joint[0], floor, hip.joint[2]] };
    let r = q.matrix();
    // Each slot's joint from the first part's, flat, turned into that frame
    // (0x8009fc64: the GTE's rotation, >> 12, kept to 16 bits).
    let turn = |v: [i32; 3]| [0, 1, 2].map(|row| ((0..3).map(|k| r[row][k] as i64 * v[k] as i64).sum::<i64>() >> 12).clamp(-32768, 32767) as i32);
    let (mut lo, mut hi) = (q.lo, q.hi);
    for slot in SLOTS {
        let Some(p) = part(slot) else { continue };
        let v = turn([(p.joint[0] - q.at[0]) as i16 as i32, 0, (p.joint[2] - q.at[2]) as i16 as i32]);
        for k in 0..3 {
            lo[k] = lo[k].min(v[k]);
            hi[k] = hi[k].max(v[k]);
        }
    }
    lo[2] = lo[2].min(BEHIND);
    hi[2] = hi[2].max(AHEAD);
    Some(Quad { lo, hi, ..q })
}

/// A shadow's quad as 0x8003414c builds it: its facing (x, z, 4096 long),
/// its extents in that frame (local +0x50..+0x5c: x, y, z), and where it
/// is set (the first part's x and z over the floor, local +0x44..+0x4c).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    pub facing: [i32; 2],
    pub lo: [i32; 3],
    pub hi: [i32; 3],
    pub at: [i32; 3],
}

impl Quad {
    /// Its matrix (rows: (z, 0, -x), (0, 4096, 0), (x, 0, z) of the
    /// facing).
    fn matrix(&self) -> [[i32; 3]; 3] {
        let [dx, dz] = self.facing;
        [[dz, 0, -dx], [0, 4096, 0], [dx, 0, dz]]
    }

    /// Its triangles in texture `t` (the shadow's sprite).
    pub fn triangles(&self, t: &Texture, out: &mut Vec<Vtx>) {
        let r = self.matrix();
        // The corners (lo x, hi z), (hi x, hi z), (lo x, lo z), (hi x,
        // lo z), turned by the same matrix and set at the first part over
        // the floor.
        let corner = |x: i32, z: i32| {
            let c = [x, self.lo[1], z];
            let w = [0, 1, 2].map(|row| (0..3).map(|k| r[row][k] as f32 * c[k] as f32).sum::<f32>() / 4096.0);
            [w[0] + self.at[0] as f32, w[1] + self.at[1] as f32, w[2] + self.at[2] as f32]
        };
        let (lo, hi) = (self.lo, self.hi);
        let pos = [corner(lo[0], hi[2]), corner(hi[0], hi[2]), corner(lo[0], lo[2]), corner(hi[0], lo[2])];
        let mode = t.clut as u32 | ((t.tpage | SUBTRACT) as u32) << 16 | Vtx::SEMI;
        let window = Vtx::window_of(
            [t.uv[0][0], t.uv[1][0], t.uv[2][0], t.uv[3][0]],
            [t.uv[0][1], t.uv[1][1], t.uv[2][1], t.uv[3][1]],
        );
        let colour = u32::from_le_bytes([GREY, GREY, GREY, 0]);
        let vtx = |k: usize| Vtx { pos: pos[k], colour, uv: Vtx::uv_of(t.uv[k][0] as u32, t.uv[k][1] as u32), mode, window };
        // (A POLY_FT4: 0 1 2, then 1 3 2.)
        out.extend([vtx(0), vtx(1), vtx(2), vtx(1), vtx(3), vtx(2)]);
    }
}

/// The floor under a body for its shadow: the tile's (+0x13c, the floor
/// query's at its place) or, with none, `fallback`.
pub fn tile_floor(col: &rugrats_data::col::Col, at: [i32; 3], fallback: i32) -> i32 {
    let q = collide::query(col, at);
    if q.tile.is_some() { q.floor } else { fallback }
}
