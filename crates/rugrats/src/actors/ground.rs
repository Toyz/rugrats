//! The ground move's probes against the COL, as the original runs them
//! (0x80064a34 and what it calls): three points ahead of the body, its
//! middle and its two sides, each asked for the floor under it from the
//! body's top (0x80067448) and for a wall on the way to it (0x80064178),
//! each sorted into a kind (0x80064444), and the three kinds deciding
//! whether the step is had, slid along the wall's tangent (0x800647b4,
//! 0x80065120) or stopped. Objects are the player's own probes'
//! (`Player::probes`); these are the height field's.

use rugrats_data::col::Col;

use crate::gte::Trig;
use crate::world::collide::{self, Segment};

/// A floor delta that means "no floor": the probe is out of the field or
/// in a tile with no floor over it (0xffff0000).
pub const VOID: i32 = -65536;

/// The body's box (the hierarchy header at +4: lo x, y, z and hi x, y, z).
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub lo: [i32; 3],
    pub hi: [i32; 3],
}

impl Body {
    /// How far ahead the probes reach past the step: the box's depth
    /// (the larger of its z extents), and its half width (of its x).
    fn depth(&self) -> i32 {
        self.lo[2].abs().max(self.hi[2].abs())
    }
    fn side(&self) -> i32 {
        self.lo[0].abs().max(self.hi[0].abs())
    }
}

/// One probe's record (0xf4 .. 0x154 as 0x80064a34 copies it): the floor
/// under it less the body's (+0x144), and the wall's normal met on the way
/// to it (+0x114, 4096 a unit; zero for none).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub delta: i32,
    pub normal: [i32; 3],
}

/// What the probes decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probed {
    /// The step is had (0x80064a34 returns 0).
    Clear(Record),
    /// The step is held up (1): the record that held it, and, when it is
    /// slid along, the tangent (+0x124, a unit) and the slant (+0x130:
    /// the angle between the step turned back and the wall's normal);
    /// None when it is stopped outright (+0x130 0).
    Held { record: Record, slide: Option<([i32; 3], i32)> },
}

impl Probed {
    pub fn held(&self) -> bool {
        matches!(self, Probed::Held { .. })
    }
    /// +0x130: the slant, 0 for a stop.
    pub fn slant(&self) -> i32 {
        match self {
            Probed::Held { slide: Some((_, a)), .. } => *a,
            _ => 0,
        }
    }
}

fn toward_zero12(v: i32) -> i32 {
    if v < 0 { (v + 4095) >> 12 } else { v >> 12 }
}

/// 0x80076944: `v` set to `len` long along its own way (VectorNormal, then
/// each axis times `len` >> 12 toward zero).
pub fn set_length(v: [i32; 3], len: i32, trig: &Trig) -> [i32; 3] {
    trig.normal(v).map(|c| toward_zero12(c * len))
}

/// 0x80067448 for the height field: the floor under `point` (asked from
/// the body's top at `top_y`, the point's own y) less the body's floor
/// `floor`; [`VOID`] for none.
pub fn height(col: &Col, point: [i32; 3], top_y: i32, floor: i32) -> i32 {
    let q = collide::query(col, point);
    if q.kind != 0 && q.tile.is_none() {
        return VOID;
    }
    if q.tile.is_none() {
        return VOID;
    }
    // The point above the tile's ceiling: the floor on top of it, asked
    // from just over that ceiling.
    if top_y < q.ceiling {
        let q2 = collide::query(col, [point[0], q.ceiling - 64, point[2]]);
        if q2.tile.is_none() {
            return VOID;
        }
        return q2.floor - floor;
    }
    q.floor - floor
}

/// 0x80064178 for the height field: the wall met on the way to a probe
/// (`off` ahead of `pos`) with floor delta `delta`; the body's floor
/// `floor`, its top `top_y`. Zero for none.
pub fn wall(col: &Col, trig: &Trig, pos: [i32; 3], off: [i32; 3], delta: i32, floor: i32, top_y: i32) -> [i32; 3] {
    let mut out = Segment::default();
    if delta < 0 {
        // Up: across at half the rise, or a little over the higher floor,
        // not over the body's top.
        let h = if delta >= -63 {
            floor + ((delta + (delta < 0) as i32) >> 1)
        } else {
            let up = floor + delta;
            if up < top_y { top_y } else { up + 64 }
        };
        let from = [pos[0], h, pos[2]];
        let to = [pos[0] + off[0], h + off[1], pos[2] + off[2]];
        if collide::segment(col, from, to, &mut out) == 1 {
            return out.normal;
        }
    } else if delta > 0 {
        // Down: across just over the floor, from 1024 ahead back to 1024
        // behind (the drop's near side).
        let h = floor + if delta < 64 { (delta + (delta < 0) as i32) >> 1 } else { 64 };
        let o = set_length(off, 1024, trig);
        let behind = [pos[0] - o[0], h - o[1], pos[2] - o[2]];
        let ahead = [pos[0] + o[0], h + o[1], pos[2] + o[2]];
        if collide::segment(col, ahead, behind, &mut out) == 1 {
            return out.normal;
        }
    }
    [0; 3]
}

/// 0x80064444: a probe's kind for a body that steps up `limit` at most.
/// 0 level; 3 a rise to walk up; 1 a step up to climb; 4 a small drop; 2
/// a step down; 6 a drop past the limit; 5 held up.
pub fn kind(r: &Record, limit: i32, trig: &Trig) -> u8 {
    let [nx, ny, nz] = r.normal;
    let flat = r.delta.abs() < 64;
    let none = nx == 0 && ny == 0 && nz == 0;
    let upright = ny == 0 && (nx != 0 || nz != 0);
    if r.delta == VOID {
        return 5;
    }
    if r.delta < 0 {
        // A normal too steep to walk up (its sine under -228 / 4096 of a
        // turn short of straight up) holds the body.
        if !none && trig.asin(ny) + 1024 >= 796 {
            return 5;
        }
        if !upright || none || flat {
            return 3;
        }
        return if r.delta < -limit { 5 } else { 1 };
    }
    if r.delta == 0 {
        return 0;
    }
    if !upright || none || flat {
        return 4;
    }
    if limit < r.delta { 6 } else { 2 }
}

/// 0x800763d0: the tangent of `wall` (a normal) for a way turned back
/// (`back`), a unit, and the angle between them (acos of their dot).
pub fn tangent(back: [i32; 3], wall: [i32; 3], trig: &Trig) -> ([i32; 3], i32) {
    let a = trig.normal(back);
    let n = trig.normal(wall);
    let dot = toward_zero12(n[0] * a[0]) + toward_zero12(n[1] * a[1]) + toward_zero12(n[2] * a[2]);
    // (dot + rcos 1024, which is 0) n - a.
    let t = [0, 1, 2].map(|k| toward_zero12(dot * n[k]) - a[k]);
    (trig.normal(t), trig.acos(dot as i16 as i32))
}

/// The three probes of a step along `way` (a unit, 4096) from `pos`
/// (its floor `floor`), `step` long, for a body `body` that steps up
/// `limit` at most (0x80064a34, the height field's part).
#[allow(clippy::too_many_arguments)]
pub fn probe(col: &Col, trig: &Trig, pos: [i32; 3], floor: i32, way: [i32; 3], step: i32, body: Body, limit: i32) -> Probed {
    let reach = body.depth() + step;
    let side = body.side();
    let top_y = pos[1] + body.lo[1];
    // The probe's offset: (x, 0, z) in the way's own frame: the way as
    // angles (0x800760f4: ratan2 of its x and z, the arc sine of its y),
    // their matrix applied (0x80075ee8).
    // (The way here is level: its pitch 0.)
    let yaw = trig.ratan2(way[0], way[2]);
    let m = trig.heading_matrix(yaw);
    let at = |x: i32, z: i32| trig.apply(&m, [x, 0, z]);
    let one = |off: [i32; 3]| {
        let point = [pos[0] + off[0], pos[1] + off[1] + body.lo[1], pos[2] + off[2]];
        let delta = height(col, point, top_y, floor);
        let normal = wall(col, trig, pos, off, delta, floor, top_y);
        (Record { delta, normal }, off)
    };
    let (centre, coff) = one(at(0, reach));
    let (left, _) = one(at(-side, reach));
    let (right, _) = one(at(side, reach));
    let (l, r, m) = (kind(&left, limit, trig), kind(&right, limit, trig), kind(&centre, limit, trig));
    let held = |k: u8| k == 5;
    // The slide along the record's wall, from the middle probe's way back
    // (0x800647b4).
    let slide = |rec: Record| {
        let back = [-coff[0], 0, -coff[2]];
        let n = [rec.normal[0], 0, rec.normal[2]];
        Probed::Held { record: rec, slide: Some(tangent(back, n, trig)) }
    };
    // A stop outright leaves no floor to climb to (+0x144 0xffff0000,
    // 0x800650cc).
    let stop = |rec: Record| Probed::Held { record: Record { delta: VOID, ..rec }, slide: None };
    // Lower first (a floor higher than the middle's stops it outright).
    let lower = |a: &Record, b: &Record| a.delta < b.delta;
    match (held(l), held(r), held(m)) {
        (false, false, false) => Probed::Clear(centre),
        (false, false, true) => slide(centre),
        (false, true, false) => slide(right),
        (false, true, true) => {
            if lower(&right, &centre) {
                stop(centre)
            } else {
                slide(centre)
            }
        }
        (true, false, false) => slide(left),
        (true, true, false) => stop(centre),
        (true, false, true) => {
            if lower(&left, &centre) {
                stop(centre)
            } else {
                slide(centre)
            }
        }
        (true, true, true) => {
            if lower(&right, &centre) || lower(&left, &centre) {
                stop(centre)
            } else {
                slide(centre)
            }
        }
    }
}
