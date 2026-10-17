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
use crate::world::boxhit::{Shape, Solid};
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
/// under it less the body's (+0x144), the wall's normal met on the way
/// to it (+0x114, 4096 a unit; zero for none), and the object met there
/// (+0xf4, +0xfc: its place in the nearby list and its box).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub delta: i32,
    pub normal: [i32; 3],
    pub hit: Option<(usize, u8)>,
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
    turned_off(back, wall, trig, false)
}

/// [`tangent`], or with `reflect` (the player's flag 0x100000, 0x8007627c)
/// the way turned back reflected off the wall: 2 (n.a) n - a.
pub fn turned_off(back: [i32; 3], wall: [i32; 3], trig: &Trig, reflect: bool) -> ([i32; 3], i32) {
    let a = trig.normal(back);
    let n = trig.normal(wall);
    let dot = toward_zero12(n[0] * a[0]) + toward_zero12(n[1] * a[1]) + toward_zero12(n[2] * a[2]);
    // (dot + rcos 1024, which is 0) n - a.
    let t = if reflect {
        [0, 1, 2].map(|k| toward_zero12(dot * (n[k] << 1)) - a[k])
    } else {
        [0, 1, 2].map(|k| toward_zero12(dot * n[k]) - a[k])
    };
    (trig.normal(t), trig.acos(dot as i16 as i32))
}

/// The three probes of a step along `way` (a unit, 4096) from `pos`
/// (its floor `floor`), `step` long, for a body `body` that steps up
/// `limit` at most (0x80064a34), with no objects near.
#[allow(clippy::too_many_arguments)]
pub fn probe(col: &Col, trig: &Trig, pos: [i32; 3], floor: i32, way: [i32; 3], step: i32, body: Body, limit: i32) -> Probed {
    let g = Ground { col, trig, near: &[], body, limit, angles: [0; 3], reflect: false, low: i32::MIN };
    g.probe(pos, floor, way, step, &mut Probes::default())
}

/// What the probes keep on the player between calls: the thing stood on
/// (+0xf8, +0xfd: its place in the nearby list and its box) and the
/// record the last probe left (+0xf4 .. +0x11c).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Probes {
    pub below: Option<(usize, u8)>,
    pub met: Met,
    /// The way turned off the last wall and its slant (+0x124, +0x130).
    pub turned: ([i32; 3], i32),
}

/// What the run's probe of its own step (0x800645d8) does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ahead {
    /// Nothing more (a level floor, or held).
    Kept,
    /// Up or down a rise followed: the position and floor moved by it.
    Followed(i32),
    /// A step up or down to ease onto (+0x188 flag 0x80, from the floor to
    /// +0x14c).
    Step { from: i32, to: i32 },
    /// A drop past the limit: the fall (state 4).
    Fall,
}

/// What the air's move meets (0x80066014's answer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Air {
    /// Nothing (0).
    Clear,
    /// A wall (1): the way turned off it and the slant across (+0x124,
    /// +0x130), none set when the move went into the floor.
    Wall { turned: Option<([i32; 3], i32)> },
    /// Landed (2): the floor moved to where.
    Land,
    /// A slope overhead (3): its tangent.
    Slide([i32; 3]),
}

/// The world the probes see: the height field, the objects near (the
/// nearby list), the body and the step it takes, and the body's angles
/// (+0x24, for 0x80067374).
pub struct Ground<'a> {
    pub col: &'a Col,
    pub trig: &'a Trig,
    pub near: &'a [Near<'a>],
    pub body: Body,
    pub limit: i32,
    pub angles: [i16; 3],
    /// The player's flag 0x100000: a wall turns the way back off it
    /// rather than along it (0x8007627c).
    pub reflect: bool,
    /// +0x68: the air probes' bottom is never above it.
    pub low: i32,
}

impl Ground<'_> {
    /// One probe `off` ahead of `pos` at `point` (0x80063d3c, 0x80067448,
    /// 0x80064178 in turn, each writing the player's record).
    fn one(&self, pos: [i32; 3], floor: i32, off: [i32; 3], point: [i32; 3], st: &mut Probes) -> Record {
        let top_y = pos[1] + self.body.lo[1];
        objects(self.col, self.trig, pos, self.body, st.below, off, self.near, &mut st.met);
        let delta = self.height_at(pos, floor, point, st);
        // The wall on the way: an object met keeps its face; going down,
        // the thing stood on is met on the way back from the drop.
        if delta < 0 {
            if st.met.hit.is_none() {
                st.met.normal = wall(self.col, self.trig, pos, off, delta, floor, top_y);
            }
        } else if delta > 0 {
            match st.below {
                Some((k, _)) => {
                    let h = floor + if delta < 64 { (delta + (delta < 0) as i32) >> 1 } else { 64 };
                    let o = set_length(off, 1024, self.trig);
                    let behind = [pos[0] - o[0], h - o[1], pos[2] - o[2]];
                    let ahead = [pos[0] + o[0], h + o[1], pos[2] + o[2]];
                    match self.near[k].solid.meet(self.trig, ahead, behind, &mut st.met.normal) {
                        Some((b, hit)) => {
                            st.met.point = hit.point;
                            st.met.index = b as i32;
                            if hit.t <= 0 {
                                st.met.normal = [0; 3];
                            }
                        }
                        None => st.met.normal = [0; 3],
                    }
                }
                None => st.met.normal = wall(self.col, self.trig, pos, off, delta, floor, top_y),
            }
        } else {
            st.met.normal = [0; 3];
        }
        Record { delta, normal: st.met.normal, hit: st.met.hit }
    }

    /// The floor under `point` less `floor` (0x80067448): on the object
    /// the probe met if any ([`Self::height_on`]), else the height field's;
    /// then the thing stood on under the point, its top if as high (which
    /// it then is the object met by).
    fn height_at(&self, pos: [i32; 3], floor: i32, point: [i32; 3], st: &mut Probes) -> i32 {
        let top_y = pos[1] + self.body.lo[1];
        let mut delta = match st.met.hit {
            Some((k, b)) => self.height_on(k, b, pos, point, floor),
            None => height(self.col, point, top_y, floor),
        };
        if let Some((k, _)) = st.below {
            let from = [point[0], point[1] + self.body.lo[1], point[2]];
            let to = [point[0], floor + 2048, point[2]];
            if let Some((_, h)) = self.near[k].solid.meet(self.trig, from, to, &mut [0; 3])
                && h.t > 0
                && floor + delta >= h.point[1]
            {
                st.met.hit = Some((k, st.met.index as u8));
                delta = h.point[1] - floor;
            }
        }
        delta
    }

    /// 0x80067448 for an object met (`k`, its box `b`): across at the
    /// box's middle height (turned by the object's angles) toward the
    /// point, then down through it from 2 * +0x86 above the body's origin
    /// to as far below; met again on the same box, that is the floor (none
    /// if met at the start); else the box's own bottom face, unturned.
    fn height_on(&self, k: usize, b: u8, pos: [i32; 3], point: [i32; 3], floor: i32) -> i32 {
        let s = self.near[k].solid;
        let (lo, hi) = s.box_of(b);
        let half = |a: i32| (a + (a < 0) as i32) >> 1;
        let mid = self.trig.turn_by(s.angles, [0, 1, 2].map(|i| half(lo[i] + hi[i])));
        let y = mid[1].wrapping_add(s.pos[1]);
        let reach = 2 * s.size;
        let mut scratch = [0; 3];
        let first = s.meet(self.trig, [pos[0], y, pos[2]], [point[0], y, point[2]], &mut scratch);
        let (from, was) = match first {
            None => ([point[0], pos[1] - reach, point[2]], -1),
            Some((j, h)) if h.t == 0 => ([point[0], pos[1] - reach, point[2]], j as i32),
            Some((j, h)) => (self.nudge(pos, [h.point[0], pos[1] - reach, h.point[2]]), j as i32),
        };
        let down = s.meet(self.trig, from, [from[0], pos[1] + reach, from[2]], &mut scratch);
        match down {
            Some((j, h)) if j as i32 == was && h.t == 0 => VOID,
            Some((j, h)) if j as i32 == was => h.point[1] - floor,
            _ => s.pos[1] + lo[1] - floor,
        }
    }

    /// 0x80067374: `p` 16 further along the body's own z (away from it,
    /// in its frame turned by its angles).
    fn nudge(&self, pos: [i32; 3], p: [i32; 3]) -> [i32; 3] {
        let mut v = self.trig.unturn_by(self.angles, [0, 1, 2].map(|i| p[i] - pos[i]));
        v[2] += if v[2] < 0 { -16 } else { 16 };
        let w = self.trig.turn_by(self.angles, v);
        [0, 1, 2].map(|i| w[i] + pos[i])
    }

    /// A segment through the height field (0x800671f8): stopped, the point
    /// and normal it stops at left in the record (+0x104, +0x114) and the
    /// face's kind ([`face_kind`]); else 0, the record as it was.
    fn col_seg(&self, from: [i32; 3], to: [i32; 3], met: &mut Met) -> u8 {
        let mut out = Segment::default();
        if collide::segment(self.col, from, to, &mut out) != 1 {
            return 0;
        }
        met.point = out.point;
        met.normal = out.normal;
        face_kind(out.normal, self.trig)
    }

    /// A segment against object `k` (0x80067284): met past its start, the
    /// face's kind; the meeting written to the record either way.
    fn obj_seg(&self, k: usize, from: [i32; 3], to: [i32; 3], met: &mut Met) -> u8 {
        match self.near[k].solid.meet(self.trig, from, to, &mut met.normal) {
            Some((b, h)) => {
                met.point = h.point;
                met.index = b as i32;
                if h.t > 0 { face_kind(met.normal, self.trig) } else { 0 }
            }
            None => 0,
        }
    }

    /// The five segments of an air probe `off` ahead (0x80065210 for the
    /// objects, 0x80065578 for the height field): up the body's axis to
    /// its `top`, across at the top, down to its `bottom`, across at the
    /// bottom, and down at the probe from top to bottom; met on that last,
    /// across again just under where it was met (+0x108 + 5). Its kind,
    /// and whether the last segment met (the answer then the one just
    /// under it, which ends an object pass however it comes out).
    fn air_segs(&self, pos: [i32; 3], off: [i32; 3], top: i32, bottom: i32, met: &mut Met, seg: &mut dyn FnMut(&Self, [i32; 3], [i32; 3], &mut Met) -> u8) -> (u8, bool) {
        let [x, y, z] = pos;
        let at = |h: i32| [x + off[0], h + off[1], z + off[2]];
        for (from, to) in [([x, y, z], [x, top, z]), ([x, top, z], at(top)), ([x, y, z], [x, bottom, z]), ([x, bottom, z], at(bottom))] {
            let r = seg(self, from, to, met);
            if r != 0 {
                return (r, false);
            }
        }
        if seg(self, at(top), at(bottom), met) == 0 {
            return (0, false);
        }
        let y5 = met.point[1] + 5;
        (seg(self, [x, y5, z], [x + off[0], y5, z + off[2]], met), true)
    }

    /// The objects' air probe (0x80065210): each object in turn, those of
    /// type flag 8 passed by; the one met is +0xf4, +0xfc.
    fn air_objects(&self, pos: [i32; 3], off: [i32; 3], top: i32, bottom: i32, st: &mut Probes) -> u8 {
        st.met.hit = None;
        for (k, n) in self.near.iter().enumerate() {
            if n.skip {
                continue;
            }
            let (r, last) = self.air_segs(pos, off, top, bottom, &mut st.met, &mut |g: &Self, a, b, m: &mut Met| g.obj_seg(k, a, b, m));
            if r != 0 {
                st.met.hit = Some((k, st.met.index as u8));
                return r;
            }
            if last {
                return 0;
            }
        }
        0
    }

    /// The air probe `off` ahead (0x800658cc): the objects', else the
    /// height field's (0x80065578). Its face's kind: 0 clear, 1 a slope
    /// overhead, 2 a wall, 3 a floor to stand on.
    fn air_probe(&self, pos: [i32; 3], off: [i32; 3], top: i32, bottom: i32, st: &mut Probes) -> u8 {
        match self.air_objects(pos, off, top, bottom, st) {
            0 => self.air_segs(pos, off, top, bottom, &mut st.met, &mut |g: &Self, a, b, m: &mut Met| g.col_seg(a, b, m)).0,
            r => r,
        }
    }

    /// The objects' probe straight up or down (0x8006592c): up the body's
    /// axis from its origin to its top and to its bottom, each carried on
    /// by a move down.
    fn air_objects_v(&self, pos: [i32; 3], dy: i32, top: i32, bottom: i32, st: &mut Probes) -> u8 {
        let [x, y, z] = pos;
        let down = if dy > 0 { dy } else { 0 };
        st.met.hit = None;
        for (k, n) in self.near.iter().enumerate() {
            if n.skip {
                continue;
            }
            for to in [top + down, bottom + down] {
                let r = self.obj_seg(k, [x, y, z], [x, to, z], &mut st.met);
                if r != 0 {
                    st.met.hit = Some((k, st.met.index as u8));
                    return r;
                }
            }
        }
        0
    }

    /// The height field's probe straight up or down (0x80065b3c): to the
    /// top carried by a move up, to the bottom by a move down.
    fn air_col_v(&self, pos: [i32; 3], dy: i32, top: i32, bottom: i32, st: &mut Probes) -> u8 {
        let [x, y, z] = pos;
        for to in [if dy < 0 { top + dy } else { top }, if dy > 0 { bottom + dy } else { bottom }] {
            let r = self.col_seg([x, y, z], [x, to, z], &mut st.met);
            if r != 0 {
                return r;
            }
        }
        0
    }

    /// Landing (0x80065cbc): the move's own probe (straight when it has no
    /// across) meeting a floor to stand on (kind 3), a thing's (which is
    /// then stood on) or else the height field's; the floor moved to the
    /// height there (0x80067448).
    fn landing(&self, pos: [i32; 3], d: [i32; 3], floor: &mut i32, st: &mut Probes) -> bool {
        let (lo, hi) = (self.body.lo, self.body.hi);
        let (top, bottom) = (pos[1] + lo[1], (pos[1] + hi[1]).max(self.low));
        let vertical = d[0] == 0 && d[2] == 0;
        let off = if vertical { [0, d[1], 0] } else { d };
        let on_thing = if vertical { self.air_objects_v(pos, d[1], top, bottom, st) } else { self.air_objects(pos, off, top, bottom, st) } == 3;
        let on_floor = on_thing
            || if vertical {
                self.air_col_v(pos, d[1], top, bottom, st)
            } else {
                self.air_segs(pos, off, top, bottom, &mut st.met, &mut |g: &Self, a, b, m: &mut Met| g.col_seg(a, b, m)).0
            } == 3;
        if !on_floor {
            return false;
        }
        let p = st.met.point;
        let delta = self.height_at(pos, *floor, [p[0], p[1] + lo[1], p[2]], st);
        *floor += delta;
        if on_thing {
            st.below = st.met.hit;
        }
        true
    }

    /// The air's move `d` (+0xd8) from `pos` (0x80066014): at or past the
    /// plane below (`plane`, +0x13c), landed there; across into the floor,
    /// a wall facing back; else air probes the body's depth past the move
    /// along it (its middle, then its sides), their faces' kinds deciding
    /// the record kept and the answer; a floor to stand on met by a move
    /// down is landed on ([`Self::landing`]).
    pub fn air(&self, pos: [i32; 3], d: [i32; 3], plane: i32, floor: &mut i32, st: &mut Probes) -> Air {
        if pos[1] >= plane {
            *floor = plane;
            return Air::Land;
        }
        let trig = self.trig;
        if d[0] != 0 || d[2] != 0 {
            let q = collide::query(self.col, [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]]);
            if pos[1] + d[1] >= q.floor {
                st.met.point = pos;
                st.met.normal = collide::unit([-d[0], 0, -d[2]]);
                return Air::Wall { turned: None };
            }
        }
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) as u32;
        let r = if len < 2 { len as i32 } else { len.isqrt() as i16 as i32 };
        let (lo, hi) = (self.body.lo, self.body.hi);
        let reach = lo[2].abs().max(hi[2].abs()) + r;
        let side = self.body.side();
        let bottom = (pos[1] + hi[1]).max(self.low);
        let top = pos[1] + lo[1];
        if d[0] == 0 && d[2] == 0 {
            let off = [0, d[1], 0];
            let mut k = match self.air_objects_v(pos, d[1], top, bottom, st) {
                0 => self.air_col_v(pos, d[1], top, bottom, st),
                k => k,
            };
            if k == 3 {
                k = 0;
                if d[1] > 0 && self.landing(pos, d, floor, st) {
                    return Air::Land;
                }
            }
            return self.turned(k, off, st);
        }
        let u = collide::unit(d);
        let angles = [trig.ratan2(u[0], u[2]) as i16, -(trig.asin(u[1] as i16 as i32)) as i16, 0];
        let at = |x: i32| trig.turn_by(angles, [x, 0, reach]);
        let centre = at(0);
        let mut c = self.air_probe(pos, centre, top, bottom, st);
        if c == 3 {
            if d[1] > 0 && self.landing(pos, d, floor, st) {
                return Air::Land;
            }
            c = 0;
        }
        let cm = st.met;
        let r = match self.air_probe(pos, at(side), top, bottom, st) {
            3 => 0,
            k => k,
        };
        let rm = st.met;
        let l = match self.air_probe(pos, at(-side), top, bottom, st) {
            3 => 0,
            k => k,
        };
        let lm = st.met;
        // The record kept and the kind answered, by the middle's, the
        // right's and the left's kinds (0x800664c8 on); all three walls,
        // held with no way along (+0x124, +0x130 zero).
        let (kept, kind) = match (c, r, l) {
            (0, 0, 0) => (cm, Some(0)),
            (0, 0, 1) => (rm, Some(0)),
            (0, 0, 2) => (lm, Some(2)),
            (0, 1, 0) | (0, 1, 1) => (rm, Some(1)),
            (0, 1, 2) => (lm, Some(2)),
            (0, 2, 0) | (0, 2, 1) => (rm, Some(2)),
            (1, 0, 0) => (cm, Some(1)),
            (1, 0, 1) => (lm, Some(1)),
            (1, 0, 2) => (lm, Some(2)),
            (1, 1, 0) | (1, 1, 2) => (rm, Some(1)),
            (1, 1, 1) => (cm, Some(1)),
            (1, 2, 0) | (1, 2, 1) => (rm, Some(2)),
            (2, 0, 0) | (2, 0, 1) | (2, 1, 0) | (2, 1, 1) => (cm, Some(2)),
            (2, 0, 2) | (2, 1, 2) => (lm, Some(2)),
            (2, 2, 0) | (2, 2, 1) => (rm, Some(2)),
            (_, 2, 2) => (cm, None),
            _ => (st.met, Some(0)),
        };
        st.met = kept;
        match kind {
            None => Air::Wall { turned: Some(([0; 3], 0)) },
            Some(k) => self.turned(k, centre, st),
        }
    }

    /// 0x800657d0: the answer for a kind, `off` the probe's offset: a wall
    /// (2) its tangent off the way back and its slant across (0x800761c8,
    /// x and z only); a slope overhead (1) its tangent, to slide along.
    fn turned(&self, kind: u8, off: [i32; 3], st: &mut Probes) -> Air {
        let back = off.map(|v| -v);
        match kind {
            1 => Air::Slide(tangent(back, st.met.normal, self.trig).0),
            2 => {
                let (t, _) = tangent(back, st.met.normal, self.trig);
                let n = st.met.normal;
                let (a, b) = (self.trig.normal([back[0], 0, back[2]]), self.trig.normal([n[0], 0, n[2]]));
                let dot = toward_zero12(b[0] * a[0]) + toward_zero12(b[1] * a[1]) + toward_zero12(b[2] * a[2]);
                Air::Wall { turned: Some((t, self.trig.acos(dot as i16 as i32))) }
            }
            3 => Air::Land,
            _ => Air::Clear,
        }
    }

    /// The run's probe of its own step (0x800645d8): one probe the step's
    /// length along the way (+0x178, at the body's origin height), its
    /// kind deciding what is stood on next, a rise followed, a step eased
    /// onto or a fall.
    pub fn ahead(&self, pos: [i32; 3], floor: i32, way: [i32; 3], step: i32, stepping: bool, st: &mut Probes) -> Ahead {
        let o = set_length(way, step, self.trig);
        let off = [o[0], 0, o[2]];
        let point = [pos[0] + off[0], pos[1], pos[2] + off[2]];
        let r = self.one(pos, floor, off, point, st);
        let kind = kind(&r, self.limit, self.trig);
        let stand = |st: &mut Probes, always: bool| match st.met.hit {
            Some(h) => st.below = Some(h),
            None if always || st.below.is_some() => st.below = None,
            None => {}
        };
        match kind {
            0 => {
                stand(st, true);
                Ahead::Kept
            }
            1 | 2 => {
                stand(st, false);
                // 0x80066f28: eased onto unless a slope (not an upright
                // face) or within 64.
                let [nx, ny, nz] = st.met.normal;
                let upright = ny == 0 && (nx != 0 || nz != 0);
                if stepping {
                    Ahead::Kept
                } else if !upright || r.delta.abs() < 64 {
                    Ahead::Followed(r.delta)
                } else {
                    Ahead::Step { from: floor, to: floor + r.delta }
                }
            }
            3 | 4 => {
                stand(st, false);
                Ahead::Followed(r.delta)
            }
            6 => Ahead::Fall,
            _ => Ahead::Kept,
        }
    }

    /// The three probes (0x80064a34): the middle at the body's depth past
    /// the step, one to each side at its half width; the record they
    /// decide on is left on the player (`st.met`).
    /// A character's probe (0x80064838, which 0x80064a34 hands an object
    /// of flag 0x4000 to): the middle one alone, at the body's depth past
    /// the step; held, the slide along what held it (0x800647b4).
    pub fn probe_middle(&self, pos: [i32; 3], floor: i32, way: [i32; 3], step: i32, st: &mut Probes) -> Probed {
        let (trig, body, limit) = (self.trig, self.body, self.limit);
        let reach = body.depth() + step;
        let yaw = trig.ratan2(way[0], way[2]);
        let m = trig.heading_matrix(yaw);
        let coff = trig.apply(&m, [0, 0, reach]);
        let point = [pos[0] + coff[0], pos[1] + coff[1] + body.lo[1], pos[2] + coff[2]];
        let centre = self.one(pos, floor, coff, point, st);
        if kind(&centre, limit, trig) == 5 {
            let back = [-coff[0], 0, -coff[2]];
            let n = [centre.normal[0], 0, centre.normal[2]];
            Probed::Held { record: centre, slide: Some(turned_off(back, n, trig, self.reflect)) }
        } else {
            Probed::Clear(centre)
        }
    }

    pub fn probe(&self, pos: [i32; 3], floor: i32, way: [i32; 3], step: i32, st: &mut Probes) -> Probed {
        let (trig, body, limit) = (self.trig, self.body, self.limit);
        let reach = body.depth() + step;
        let side = body.side();
        // The probe's offset: (x, 0, z) in the way's own frame: the way as
        // angles (0x800760f4: ratan2 of its x and z, the arc sine of its y),
        // their matrix applied (0x80075ee8).
        // (The way here is level: its pitch 0.)
        let yaw = trig.ratan2(way[0], way[2]);
        let m = trig.heading_matrix(yaw);
        let at = |x: i32, z: i32| trig.apply(&m, [x, 0, z]);
        let one = |off: [i32; 3], st: &mut Probes| {
            let point = [pos[0] + off[0], pos[1] + off[1] + body.lo[1], pos[2] + off[2]];
            (self.one(pos, floor, off, point, st), st.met)
        };
        let coff = at(0, reach);
        let (centre, cm) = one(coff, st);
        let (left, lm) = one(at(-side, reach), st);
        let (right, rm) = one(at(side, reach), st);
        let (l, r, c) = (kind(&left, limit, trig), kind(&right, limit, trig), kind(&centre, limit, trig));
        let held = |k: u8| k == 5;
        // The slide along the record's wall, from the middle probe's way back
        // (0x800647b4).
        let slide = |rec: Record| {
            let back = [-coff[0], 0, -coff[2]];
            let n = [rec.normal[0], 0, rec.normal[2]];
            Probed::Held { record: rec, slide: Some(turned_off(back, n, trig, self.reflect)) }
        };
        // A stop outright leaves no floor to climb to (+0x144 0xffff0000,
        // 0x800650cc).
        let stop = |rec: Record| Probed::Held { record: Record { delta: VOID, ..rec }, slide: None };
        // Lower first (a floor higher than the middle's stops it outright).
        let lower = |a: &Record, b: &Record| a.delta < b.delta;
        // The record copied back to the player: the one decided on (none
        // copied, the right's, the last run, stays).
        let (out, kept) = match (held(l), held(r), held(c)) {
            (false, false, false) => (Probed::Clear(centre), cm),
            (false, false, true) => (slide(centre), cm),
            (false, true, false) => (slide(right), rm),
            (false, true, true) if lower(&right, &centre) => (stop(centre), cm),
            (false, true, true) => (slide(centre), cm),
            (true, false, false) => (slide(left), lm),
            // Stopped on the last probe's record (0x80065034).
            (true, true, false) => (stop(right), rm),
            (true, false, true) if lower(&left, &centre) => (stop(centre), cm),
            (true, false, true) => (slide(centre), cm),
            (true, true, true) if lower(&right, &centre) || lower(&left, &centre) => (stop(centre), cm),
            (true, true, true) => (slide(centre), cm),
        };
        st.met = kept;
        st.turned = match out {
            Probed::Held { slide: Some(t), .. } => t,
            Probed::Held { slide: None, .. } => ([0; 3], 0),
            Probed::Clear(_) => st.turned,
        };
        out
    }
}

/// The player's object record as the probes write it (+0xf4 .. +0x11c):
/// the object met and its box (+0xf4, +0xfc: by its place in the nearby
/// list), and the point and normal the meetings wrote (+0x104, +0x114),
/// kept or not, as the buffers the original passes down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Met {
    pub hit: Option<(usize, u8)>,
    pub point: [i32; 3],
    /// +0x110: the box index the last meeting wrote after its point.
    pub index: i32,
    pub normal: [i32; 3],
}

/// An object near the body (the nearby list, +0x100): its solid, and
/// whether the collision tests pass it by (its type's flag 8).
#[derive(Clone, Copy, Debug)]
pub struct Near<'a> {
    pub solid: &'a Solid,
    pub skip: bool,
}

/// The object probe (0x80063d3c) for the probe `off` ahead of `pos`: each
/// object in turn (but those skipped), for a model each box but the one
/// stood on (`below`: its place in the list and its box), a segment from
/// the body's x and z at the box's middle height (kept within the body's
/// span) to `off` ahead (a character's at its origin's height); met past
/// its start (0x80067284), not on the box stood on, and with no wall of
/// the height field as near or nearer on the way (0x80063c3c), it is the
/// object met.
#[allow(clippy::too_many_arguments)]
pub fn objects(col: &Col, trig: &Trig, pos: [i32; 3], body: Body, below: Option<(usize, u8)>, off: [i32; 3], near: &[Near], met: &mut Met) {
    met.hit = None;
    // Within the body's span, the top first.
    let span = |y: i32| {
        let (top, bottom) = (pos[1] + body.lo[1], pos[1] + body.hi[1]);
        if y < top {
            top
        } else if bottom < y {
            bottom
        } else {
            y
        }
    };
    // A meeting kept unless the height field's wall is as near.
    let nearer_wall = |from: [i32; 3], to: [i32; 3], at: [i32; 3]| {
        let mut out = Segment::default();
        if collide::segment(col, from, to, &mut out) != 1 {
            return false;
        }
        let d = |p: [i32; 3]| (0..3).fold(0u32, |a, k| a.wrapping_add((from[k].wrapping_sub(p[k])).wrapping_mul(from[k].wrapping_sub(p[k])) as u32));
        d(at) >= d(out.point)
    };
    for (k, n) in near.iter().enumerate() {
        if n.skip {
            continue;
        }
        let s = n.solid;
        let heights: Vec<(Option<u8>, i32)> = match &s.shape {
            Shape::Model(boxes) => boxes
                .iter()
                .enumerate()
                .map(|(j, (lo, hi))| (Some(j as u8), s.pos[1].wrapping_add(toward_zero_half(lo[1] + hi[1]))))
                .collect(),
            Shape::Body(..) | Shape::Parts(..) if below.is_some_and(|(b, _)| b == k) => continue,
            Shape::Body(..) | Shape::Parts(..) => vec![(None, s.pos[1])],
        };
        for (j, y) in heights {
            if let (Some(j), Some((b, bj))) = (j, below)
                && b == k
                && bj == j
            {
                continue;
            }
            let from = [pos[0], span(y), pos[2]];
            let to = [from[0].wrapping_add(off[0]), from[1].wrapping_add(off[1]), from[2].wrapping_add(off[2])];
            let Some((bx, h)) = s.meet(trig, from, to, &mut met.normal) else { continue };
            met.point = h.point;
            met.index = bx as i32;
            if h.t <= 0 {
                continue;
            }
            if j.is_some() && below == Some((k, bx as u8)) {
                continue;
            }
            if nearer_wall(from, to, h.point) {
                continue;
            }
            met.hit = Some((k, bx as u8));
            return;
        }
    }
}

/// The kind of face a normal makes (0x80067168): level across (y 0) a
/// wall (2); facing down (y > 0) a slope overhead (1) within 512 of
/// straight down, else a wall; facing up a floor to stand on (3) when its
/// sine's angle plus a quarter turn is under 796, else a wall.
pub fn face_kind(n: [i32; 3], trig: &Trig) -> u8 {
    let ny = n[1];
    if ny == 0 {
        return 2;
    }
    let a = trig.asin(ny as i16 as i32) as i16 as i32;
    if ny > 0 {
        if 1024 - a < 512 { 1 } else { 2 }
    } else if a + 1024 < 796 {
        3
    } else {
        2
    }
}

/// (a + (a < 0)) >> 1: half, toward zero.
fn toward_zero_half(a: i32) -> i32 {
    (a + (a < 0) as i32) >> 1
}
