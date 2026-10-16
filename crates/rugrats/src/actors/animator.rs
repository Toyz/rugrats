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
        // Reptar; the golf holes' table (0x800c4584, 0x800ba158) has no
        // Reptar to use it.
        9 => (0x800b_a754, 0x800b_a0d4),
        43 => (0x800c_458c, 0x800b_a4f4),
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

/// The table state of logical animation `a` (0x80056fe4), which every
/// 0x800572a8 call goes through: the numbers the game's code asks for
/// are not the table's. `climb` is the height a climb rises (+0x144),
/// which picks one of six climbs. None keeps the state playing (17, the
/// idle's "as you were"), or names none (a climb over 1033).
pub fn table_state(a: i16, climb: i32, current: i16) -> Option<i16> {
    const MAP: [i16; 54] = [
        0, 1, 3, 5, 8, -1, 32, 27, 30, 31, 33, 20, 22, 15, 16, 7, 60, -1, 26, 25, 19, 17, 34, 36, 37, 38, 39, 40, 41,
        42, 43, 44, 0, 45, 46, 47, 48, 49, 0, 50, 51, 52, 53, 54, 0, 55, 56, 57, 58, 59, 61, 62, 64, 63,
    ];
    match a {
        // 0x80056e44: the rise's band, 128 wide from under 394 to 1033.
        5 => {
            let h = climb.abs();
            (0..6).find(|&k| h < 394 + 128 * k).map(|k| 9 + k as i16)
        }
        17 => Some(current),
        // 0x80056e10 gives 28 when 0x800590d4 says running; nothing
        // asks for 6 that the port plays.
        0..=53 => Some(MAP[a as usize]),
        _ => Some(0),
    }
}

/// What a state does at its last frame (0x800b9c30, by table state; read
/// by 0x8005562c): 4 turns back (ping-pong), 2 loops, 1 completes into the
/// next state (0x80056484), else it holds the last frame (8 or nothing).
/// 32 adds the root's motion (not ported).
const ENDS: [u16; 66] = [
    1, 1, 1, 1, 1, 33, 40, 8, 1, 33, 33, 33, 33, 33, 33, 33, 97, 1, 1, 2, 33, 2, 33, 1, 1, 1, 1, 1, 33, 33, 33, 34, 33,
    4, 1, 1, 1, 33, 33, 33, 2, 40, 33, 2, 1, 1, 8, 2, 1, 1, 1, 1, 33, 1, 33, 1, 2, 1, 1, 1, 8, 1, 1, 1, 1, 0,
];

/// The state a completed one goes on to (0x80056484): the halves of the
/// walk (1, 2), the run (3, 4) and the idle (17, 18) and others in turn,
/// the jump's rise into its fall (5, 6). None goes back to the state
/// asked for after (+0xe6, logical 0: standing).
fn next(state: i16) -> Option<i16> {
    match state {
        1 => Some(2),
        2 => Some(1),
        3 => Some(4),
        4 => Some(3),
        5 => Some(6),
        17 => Some(18),
        18 => Some(17),
        34 => Some(35),
        35 => Some(34),
        39 | 58 => Some(40),
        45 => Some(46),
        _ => None,
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
    /// Running backward (a ping-pong's way back, +0xb).
    back: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Animator {
    tracks: [Track; 4],
    remap: Vec<i16>,
    entries: Vec<[i16; 4]>,
    /// The logical animation last asked for, and the climb's rise for
    /// the next climb (logical 5).
    asked: i16,
    pub climb: i32,
}

impl Animator {
    /// Character `character`'s tables.
    pub fn load(disc: &GameDisc, character: u16) -> Result<Animator, String> {
        let (entries_at, remap_at) = tables(character);
        let s16s = |addr: u32, n: usize| -> Result<Vec<i16>, String> {
            Ok(disc
                .exe_bytes(addr, 2 * n)?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|h| i16::from_le_bytes([h[0], h[1]]))
                .collect())
        };
        let remap = s16s(remap_at, REMAP_COUNT)?;
        let count = remap.iter().copied().filter(|&e| (0..256).contains(&e)).max().unwrap_or(0) as usize + 1;
        let entries =
            s16s(entries_at, 4 * count)?.as_chunks::<4>().0.iter().map(|e| [e[0], e[1], e[2], e[3]]).collect();
        Ok(Animator { tracks: [Track::default(); 4], remap, entries, asked: 0, climb: 0 })
    }

    /// State `a` plays `ids` on its four tracks (0x8008a668).
    pub fn set_state(&mut self, a: i16, ids: [i16; 4]) {
        if let Some(r) = self.remap.get_mut(a as usize) {
            *r = self.entries.len() as i16;
            self.entries.push(ids);
        }
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left).
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        let t = &self.tracks[0];
        (t.anim, t.first, t.count, t.cur, t.left)
    }

    /// The logical animation last asked for.
    pub fn asked(&self) -> i16 {
        self.asked
    }

    /// Whether logical animation `a` plays on the base track and has
    /// frames to go.
    pub fn playing(&self, a: i16) -> bool {
        let t = &self.tracks[0];
        self.asked == a
            && table_state(a, self.climb, t.anim) == Some(t.anim)
            && t.cur + 1 < t.first + t.count
            && t.left != i32::MAX
    }

    /// The ANM animation of logical animation `a` on the base track
    /// (0x80055f7c: -1 none).
    pub fn anm(&self, a: i16) -> Option<usize> {
        self.anm_on(table_state(a, self.climb, self.tracks[0].anim)?, 0)
    }

    fn anm_on(&self, a: i16, k: usize) -> Option<usize> {
        let e = *self.remap.get(a as usize)?;
        let id = self.entries.get(e as usize)?[k];
        (id >= 0).then_some(id as usize)
    }

    /// Asks for logical animation `a` (0x800572a8), unless it is the one
    /// last asked for: the game asks on entering a state, and what it
    /// asked for then runs on through its states.
    pub fn set(&mut self, a: i16, anims: &Anims) {
        if self.asked == a && self.tracks[0].count > 0 {
            return;
        }
        self.start(a, anims);
    }

    /// Starts logical animation `a` from its first frame, even if it is
    /// playing.
    pub fn start(&mut self, a: i16, anims: &Anims) {
        let Some(s) = table_state(a, self.climb, self.tracks[0].anim) else { return };
        self.asked = a;
        self.tracks[0].count = 0;
        self.play(s, anims);
    }

    /// Plays table state `a`, unless it is playing.
    fn play(&mut self, a: i16, anims: &Anims) {
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
            self.tracks[k] = Track { anim: a, first, count, prev, cur: first, left, back: false };
        }
    }

    /// One frame (dt) of every track. The base track ends as its state
    /// says (0x8005562c); the others loop.
    pub fn step(&mut self, anims: &Anims) {
        let mut done = None;
        for (k, t) in self.tracks.iter_mut().enumerate() {
            if t.count == 0 {
                continue;
            }
            t.left -= DT;
            while t.left <= 0 {
                let last = t.first + t.count - 1;
                let ends = if k == 0 { ENDS.get(t.anim as usize).copied().unwrap_or(1) } else { 2 };
                t.prev = t.cur;
                if t.back {
                    if t.cur > t.first {
                        t.cur -= 1;
                    } else {
                        t.back = false;
                        t.cur = (t.first + 1).min(last);
                    }
                } else if t.cur < last {
                    t.cur += 1;
                } else if ends & 4 != 0 {
                    t.back = t.count > 1;
                    t.cur = if t.count > 1 { last - 1 } else { t.first };
                } else if ends & 2 != 0 {
                    t.cur = t.first;
                } else if ends & 1 != 0 {
                    done = Some(t.anim);
                    break;
                } else {
                    // Held on the last frame.
                    t.left = i32::MAX;
                    break;
                }
                let d = anims.frames.get(t.cur).map_or(1, |f| f.duration.max(1) as i32);
                t.left += d;
            }
        }
        if let Some(s) = done {
            let to = next(s).unwrap_or(0);
            self.tracks[0].count = 0;
            self.play(to, anims);
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
