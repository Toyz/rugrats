//! A character's animation (SLUS_006.50:0x80056834 and the samplers
//! 0x80055170, 0x800552bc): four tracks (object +0xa8, +0xb4, +0xc0,
//! +0xcc), each playing its column of the state's entry, a later track
//! over the parts it keys; each frame blended from the one before
//! (0x800202ec).
//!
//! The tables by character (0x8008a42c, into object +0xe8 and +0xec): a
//! state's entry (s16 a state) and per entry four ANM ids (s16, -1 none).

use rrt::glam::Mat3;
use rugrats_data::anim::{Anims, Key};
use rugrats_data::hierarchy::Hierarchy;
use rugrats_render::models::quat_matrix;

use crate::disc::GameDisc;

/// dt a frame at 30 fps.
const DT: i32 = 10;
const REMAP_COUNT: usize = 132;

/// (entries, remap) by character (0x8008a42c's cases, character - 2).
fn tables(character: u16) -> (u32, u32) {
    match character {
        2 => (0x800b_a9e4, 0x800b_9dbc),
        3 | 37 => (0x800b_ab6c, 0x800b_9e40),
        4 | 5 => (0x800b_a5fc, 0x800b_9d38),
        6 => (0x800b_ac9c, 0x800b_9ec4),
        10 | 11 => (0x800b_a7e4, 0x800b_9fcc),
        12 | 21 => (0x800b_ad8c, 0x800b_9f48),
        16 | 47 => (0x800b_a714, 0x800b_a2e4),
        22 => (0x800b_ae3c, 0x800b_a578),
        23 => (0x800b_a80c, 0x800b_a050),
        33 => (0x800b_a6fc, 0x800b_a1dc),
        34 => (0x800b_ae0c, 0x800b_a368),
        35 => (0x800b_add4, 0x800b_a260),
        39 => (0x800b_ae2c, 0x800b_a470),
        41 => (0x800b_ae1c, 0x800b_a3ec),
        _ => (0x800b_a824, 0x800b_9cb4),
    }
}

/// One track playing: state, its animation's frames, the frame before,
/// the frame showing, and time left in it.
#[derive(Clone, Copy, Debug, Default)]
struct Track {
    anim: i16,
    first: usize,
    count: usize,
    prev: usize,
    cur: usize,
    left: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Animator {
    tracks: [Track; 4],
    remap: Vec<i16>,
    entries: Vec<[i16; 4]>,
}

impl Animator {
    /// Character `character`'s tables.
    pub fn load(disc: &GameDisc, character: u16) -> Result<Animator, String> {
        let (entries_at, remap_at) = tables(character);
        let s16s = |addr: u32, n: usize| -> Result<Vec<i16>, String> {
            Ok(disc.exe_bytes(addr, 2 * n)?.chunks_exact(2).map(|h| i16::from_le_bytes([h[0], h[1]])).collect())
        };
        let remap = s16s(remap_at, REMAP_COUNT)?;
        let count = remap.iter().copied().filter(|&e| (0..256).contains(&e)).max().unwrap_or(0) as usize + 1;
        let entries = s16s(entries_at, 4 * count)?.chunks_exact(4).map(|e| [e[0], e[1], e[2], e[3]]).collect();
        Ok(Animator { tracks: [Track::default(); 4], remap, entries })
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left).
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        let t = &self.tracks[0];
        (t.anim, t.first, t.count, t.cur, t.left)
    }

    /// Whether state `a` plays on the base track and has frames to go.
    pub fn playing(&self, a: i16) -> bool {
        let t = &self.tracks[0];
        t.anim == a && t.cur + 1 < t.first + t.count
    }

    /// The ANM animation of state `a` on track `k` (0x80055f7c: -1 none).
    pub fn anm(&self, a: i16) -> Option<usize> {
        self.anm_on(a, 0)
    }

    fn anm_on(&self, a: i16, k: usize) -> Option<usize> {
        let e = *self.remap.get(a as usize)?;
        let id = self.entries.get(e as usize)?[k];
        (id >= 0).then_some(id as usize)
    }

    /// Plays state `a` (0x800572a8), unless it is playing.
    pub fn set(&mut self, a: i16, anims: &Anims) {
        if self.tracks[0].anim == a && self.tracks[0].count > 0 {
            return;
        }
        for k in 0..4 {
            let old = self.tracks[k];
            let Some((first, count)) = self.anm_on(a, k).and_then(|id| anims.index.get(id).copied().flatten()) else {
                self.tracks[k] = Track { anim: a, ..Track::default() };
                continue;
            };
            let (first, count) = (first as usize, count as usize);
            // Blend from the frame showing into the new animation's first.
            let prev = if old.count > 0 { old.cur } else { first };
            let left = anims.frames.get(first).map_or(1, |f| f.duration.max(1) as i32);
            self.tracks[k] = Track { anim: a, first, count, prev, cur: first, left };
        }
    }

    /// One frame (dt) of every track.
    pub fn step(&mut self, anims: &Anims) {
        for t in &mut self.tracks {
            if t.count == 0 {
                continue;
            }
            t.left -= DT;
            while t.left <= 0 {
                t.prev = t.cur;
                t.cur = if t.cur + 1 < t.first + t.count { t.cur + 1 } else { t.first };
                let d = anims.frames.get(t.cur).map_or(1, |f| f.duration.max(1) as i32);
                t.left += d;
            }
        }
    }

    /// The parts' rotations, each track blended from the frame before
    /// into the frame showing (0x800552bc, 0x800202ec).
    pub fn locals(&self, hierarchy: &Hierarchy, anims: &Anims) -> Vec<Option<Mat3>> {
        let mut out = vec![None; hierarchy.parts.len()];
        let quat =
            |q: [i16; 4]| rrt::glam::Quat::from_xyzw(q[0] as f32, q[1] as f32, q[2] as f32, q[3] as f32).normalize();
        for t in self.tracks.iter().filter(|t| t.count > 0) {
            let (Some(cur), Some(prev)) = (anims.frames.get(t.cur), anims.frames.get(t.prev)) else { continue };
            let dur = cur.duration.max(1) as f32;
            let f = ((dur - t.left as f32) / dur).clamp(0.0, 1.0);
            for key in anims.keys_of(cur) {
                let Key::Rotation { channel, q } = *key else { continue };
                let part = hierarchy.header.get(0x14 + channel as usize).copied().unwrap_or(0xff);
                let Some(slot) = out.get_mut(part as usize) else { continue };
                let from = anims.keys_of(prev).iter().find_map(|k| match *k {
                    Key::Rotation { channel: c, q } if c == channel => Some(q),
                    _ => None,
                });
                let q = match from {
                    Some(p) => {
                        let s = quat(p).slerp(quat(q), f);
                        [(s.x * 4096.0) as i16, (s.y * 4096.0) as i16, (s.z * 4096.0) as i16, (s.w * 4096.0) as i16]
                    }
                    None => q,
                };
                *slot = Some(quat_matrix(q));
            }
        }
        out
    }
}
