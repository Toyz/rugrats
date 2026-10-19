//! The rooms the render draws (SLUS_006.50:0x80034058 -> 0x80033ac4):
//! from the camera's room out through its portals, each room reached with
//! the screen rectangle its portal shows, which the draw of its objects
//! (0x80033818, the culls of 0x80031834) keeps to.
//!
//! The walk is breadth first. An entry's portal stamped this frame is
//! passed over (each portal is stamped as its entry is taken, and those of
//! a room reached that lead back are stamped as it is put on). A room
//! 65000 or more ahead of the eye (0x8001f0ac: the camera's z of its
//! centre) is not drawn and leads nowhere. Its portals, unless shut
//! (corner 0's flag 2), or to a room the first room's list (+0x5e, +0x60)
//! leaves out: with the eye within 2048 of a portal's box it shows the
//! whole rectangle (0x800307f8); else its corners are taken to the camera,
//! cut at z 0 (0x8003311c) and projected, and it shows where they fall
//! inside the rectangle, unless all fall off one side (0x80033278). The
//! rooms are then drawn by their depth's 4096 band, the farthest first,
//! each band the last put on first (0x80033f80).

use super::*;
use crate::media::cine::Portal;

/// The screen (0x800c4a00): x, y, width, height.
pub const SCREEN: [i32; 4] = [-256, -120, 512, 240];
const NEAR_PORTAL: i32 = 2048;
const TOO_FAR: i32 = 0xfde7;

/// A room drawn, and the rectangle it is drawn into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawn {
    pub room: usize,
    pub rect: [i32; 4],
}

/// The camera as the render takes it: the eye and the rows, row 0 scaled
/// by 512 / 320 (0x8001e0f4), and the projection distance.
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    pub at: [i32; 3],
    pub rows: [[i32; 3]; 3],
    pub h: i32,
}

impl Eye {
    /// `v` (relative to the eye) in the camera (the GTE's rotation, the
    /// halving of 0x8001f0ac for a vector of 32768 or more across).
    pub fn camera(&self, v: [i32; 3]) -> [i32; 3] {
        let big = v.iter().any(|&c| c >= 0x8000 || c < -0x8000);
        let v = if big { v.map(|c| (c + ((c as u32) >> 31) as i32) >> 1) } else { v };
        let m = self.rows.map(|r| ((0..3).map(|k| r[k] as i64 * v[k] as i64).sum::<i64>() >> 12) as i32);
        if big { m.map(|c| c << 1) } else { m }
    }
}

impl Game {
    /// The camera as the render takes it.
    pub fn render_eye(&self) -> Eye {
        let (eye, rows) = self.view;
        let rows = self.rows.unwrap_or_else(|| {
            [0, 1, 2].map(|r| {
                let v = rows.row(r);
                [v.x, v.y, v.z].map(|c| (c * 4096.0).round() as i32)
            })
        });
        let scale = |v: i32| v * 512 / 320;
        Eye {
            at: [eye.x, eye.y, eye.z].map(|c| c.round() as i32),
            rows: [rows[0].map(scale), rows[1], rows[2]],
            h: self.focal.round() as i32,
        }
    }

    /// The camera's room (camera +0x80): the room holding the eye, by the
    /// last as a hint (0x8002b660); the player's outside them all.
    pub(super) fn camera_room(&self, eye: [i32; 3], hint: Option<usize>) -> Option<usize> {
        let inside =
            |k: usize| self.set.room_info.get(k).is_some_and(|r| (0..3).all(|i| r.min[i] <= eye[i] && eye[i] <= r.max[i]));
        match hint.filter(|&h| inside(h)) {
            Some(h) => Some(h),
            None => hint
                .and_then(|h| self.set.neighbours.get(h))
                .and_then(|n| n.iter().copied().find(|&k| inside(k)))
                .or_else(|| (0..self.set.room_info.len()).find(|&k| inside(k)))
                .or(self.room),
        }
    }

    /// The rooms drawn from `start` as `eye` sees them, in the order
    /// drawn.
    pub fn rooms_drawn(&self, start: usize, eye: &Eye) -> Vec<Drawn> {
        let set = &self.set;
        // (A portal's stamp: room, index.)
        let mut stamped: std::collections::HashSet<(usize, usize)> = Default::default();
        // The queue: room, the portal it came through, its rectangle.
        let mut queue: Vec<(usize, Option<(usize, usize)>, [i32; 4])> = vec![(start, None, SCREEN)];
        let list = set.room_lists.get(start).cloned().unwrap_or_default();
        let mut bands: Vec<Vec<usize>> = vec![Vec::new(); 16];
        let mut i = 0;
        while i < queue.len() {
            let (room, via, rect) = queue[i];
            i += 1;
            if let Some(p) = via
                && !stamped.insert(p)
            {
                continue;
            }
            let Some(&centre) = set.centres.get(room) else { continue };
            let tr = eye.camera([0, 1, 2].map(|k| centre[k] - eye.at[k]));
            let depth = tr[2];
            if depth > TOO_FAR {
                continue;
            }
            bands[(depth.max(0) >> 12).min(15) as usize].push(i - 1);
            for (j, p) in set.portals[room].iter().enumerate() {
                if p.shut {
                    continue;
                }
                let to = p.to as usize;
                if !list.is_empty() && !list.contains(&(p.to as u8)) {
                    continue;
                }
                let shown = if near_portal(p, centre, eye.at) {
                    Some(rect)
                } else if stamped.contains(&(room, j)) {
                    None
                } else {
                    portal_rect(p, eye, tr, rect)
                };
                if stamped.contains(&(room, j)) {
                    continue;
                }
                let Some(shown) = shown else { continue };
                // Its target's portals back stamped.
                if let Some(back) = set.portals.get(to) {
                    for (b, q) in back.iter().enumerate() {
                        if q.to as usize == room {
                            stamped.insert((to, b));
                        }
                    }
                }
                queue.push((to, Some((room, j)), shown));
            }
        }
        let mut out = Vec::new();
        for band in bands.iter().rev() {
            for &e in band.iter().rev() {
                out.push(Drawn { room: queue[e].0, rect: queue[e].2 });
            }
        }
        out
    }
}

/// Whether the eye is within 2048 of the portal's box (0x800307f8, about
/// its room's centre).
fn near_portal(p: &Portal, centre: [i32; 3], eye: [i32; 3]) -> bool {
    (0..3).all(|k| {
        let lo = p.corners.iter().map(|c| c[k]).min().unwrap_or(0);
        let hi = p.corners.iter().map(|c| c[k]).max().unwrap_or(0);
        let e = eye[k] - centre[k];
        lo - NEAR_PORTAL <= e && e <= hi + NEAR_PORTAL
    })
}

/// Where portal `p` shows inside `rect` (0x80033278): its corners in the
/// camera (`tr` the room's centre there), cut at z 0, projected; none when
/// all fall off one side; the whole rectangle when one is at z 0 or
/// nearer; else their box within it.
fn portal_rect(p: &Portal, eye: &Eye, tr: [i32; 3], rect: [i32; 4]) -> Option<[i32; 4]> {
    let cam: Vec<[i32; 3]> = p
        .corners
        .iter()
        .map(|c| {
            let m = eye.rows.map(|r| ((0..3).map(|k| r[k] as i64 * c[k] as i64).sum::<i64>() >> 12) as i32);
            [0, 1, 2].map(|k| m[k] + tr[k])
        })
        .collect();
    let cut = clip_near(&cam);
    if cut.len() < 3 {
        return None;
    }
    let screen: Vec<[i32; 2]> = cut
        .iter()
        .map(|v| {
            let (mut x, mut y) = (v[0].wrapping_mul(eye.h), v[1].wrapping_mul(eye.h));
            if v[2] >= 2 {
                let half = v[2] / 2;
                x = (x + half) / v[2];
                y = (y + half) / v[2];
            }
            [x.clamp(-0x8000, 0x7fff) as i16 as i32, y.clamp(-0x8000, 0x7fff) as i16 as i32]
        })
        .collect();
    let mut all_out = 0xf;
    for s in &screen {
        let mut code = 0;
        if s[0] < rect[0] {
            code |= 1;
        }
        if rect[0] + rect[2] < s[0] {
            code |= 2;
        }
        if s[1] < rect[1] {
            code |= 4;
        }
        if rect[1] + rect[3] < s[1] {
            code |= 8;
        }
        all_out &= code;
    }
    if all_out != 0 {
        return None;
    }
    if cut.iter().any(|v| v[2] <= 0) {
        return Some(rect);
    }
    let (mut x0, mut y0, mut x1, mut y1) = (screen[0][0], screen[0][1], screen[0][0], screen[0][1]);
    for s in &screen[1..] {
        x0 = x0.min(s[0]);
        y0 = y0.min(s[1]);
        x1 = x1.max(s[0]);
        y1 = y1.max(s[1]);
    }
    let x = x0.max(rect[0]);
    let y = y0.max(rect[1]);
    let r = x1.min(rect[0] + rect[2]);
    let b = y1.min(rect[1] + rect[3]);
    Some([x, y, r - x, b - y].map(|v| v as i16 as i32))
}

/// The polygon cut at z 0 (0x8003311c): the corners at z 0 or beyond, and
/// where each edge crosses (x and y by (0 - z0) << 12 / (z1 - z0)).
fn clip_near(v: &[[i32; 3]]) -> Vec<[i32; 3]> {
    let mut out = Vec::new();
    let n = v.len();
    let mut inside = v[0][2] >= 0;
    for k in 0..n {
        let (a, b) = (v[k], v[(k + 1) % n]);
        if inside {
            out.push(a);
        }
        let next = b[2] >= 0;
        if inside != next {
            let t = ((-a[2]) << 12) / (b[2] - a[2]);
            let lerp = |i: usize| {
                let d = (b[i] - a[i]) * t;
                a[i] + if d < 0 { (d + 4095) >> 12 } else { d >> 12 }
            };
            out.push([lerp(0), lerp(1), 0]);
        }
        inside = next;
    }
    out
}
