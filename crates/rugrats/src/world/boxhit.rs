//! A segment against a box, as the original tests the player's probes
//! against an object's model boxes, in its integers: the slab test in the
//! box's own frame (0x80068e68, its plane crossings 0x8006876c, 0x80068914,
//! 0x80068ac8 and their checks 0x80068dc4, 0x80068d20, 0x80068c7c), and the
//! object's boxes, turned by its angles (0x80069c40).

use crate::gte::Trig;

/// Where a crossing is and how far along (4096 the whole segment).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hit {
    /// The fraction along, 0..4096 (0 when the start is in the box).
    pub t: i32,
    /// The point (in the box's frame for [`slab`], the world's for
    /// [`object`]).
    pub point: [i32; 3],
    /// The face's outward normal (4096 a unit); zero when the start is in
    /// the box.
    pub normal: [i32; 3],
}

/// (v << 12) >> 12: the low 20 bits, signed.
fn sext20(v: i32) -> i32 {
    (v << 12) >> 12
}

/// The crossing of `seg` (from `a` to `b`) with the plane where axis `k`
/// is `at` (0x8006876c for x, 0x80068914 for y, 0x80068ac8 for z): its
/// fraction and point, or None when the segment runs along the plane off
/// it. Along it, on it: the start, at 0.
fn cross(a: [i32; 3], b: [i32; 3], k: usize, at: i32) -> Option<(i32, [i32; 3])> {
    let d = [b[0].wrapping_sub(a[0]), b[1].wrapping_sub(a[1]), b[2].wrapping_sub(a[2])];
    let off = sext20(at.wrapping_sub(a[k]));
    let span = sext20(d[k]);
    if span == 0 {
        return (off == 0).then_some((0, a));
    }
    let t = (off << 12).wrapping_div(span);
    let along = |j: usize| a[j].wrapping_add(d[j].wrapping_mul(off).wrapping_div(span));
    Some((t, [along(0), along(1), along(2)]))
}

/// A crossing kept only when it is on the segment (0..4096) and within the
/// box's other two extents (0x80068dc4, 0x80068d20, 0x80068c7c).
fn face(a: [i32; 3], b: [i32; 3], size: [i32; 3], k: usize, at: i32) -> Option<(i32, [i32; 3])> {
    let (t, p) = cross(a, b, k, at)?;
    if t as u32 > 4096 {
        return None;
    }
    let within = |j: usize| p[j] >= 0 && p[j] <= size[j];
    (0..3).filter(|&j| j != k).all(within).then_some((t, p))
}

/// The segment from `a` to `b` against the box from 0 to `size` (its own
/// frame), 0x80068e68: the first face it enters by, or the start when it
/// starts inside. None for a miss (-1) and for the cases the original
/// gives up on (-2).
pub fn slab(a: [i32; 3], b: [i32; 3], size: [i32; 3]) -> Option<Hit> {
    let mut normal = [0; 3];
    slab_into(a, b, size, &mut normal).map(|(t, point)| Hit { t, point, normal })
}

/// [`slab`] with the normal written where the original writes it: the
/// face's when the start is out on one or two axes, hit or not; the face's
/// on a hit when out on all three; untouched when the start is inside.
fn slab_into(a: [i32; 3], b: [i32; 3], size: [i32; 3], normal: &mut [i32; 3]) -> Option<(i32, [i32; 3])> {
    // The start's outcode (1 past the far side, 2 before the near, by
    // axis x, y, z at bits 0, 2, 4), the axes it is within and how many.
    let (mut out_a, mut inside, mut count) = (0u32, 0u32, 0u32);
    let mut out_b = 0u32;
    for k in 0..3 {
        let bit = 1 << (2 * k);
        if a[k] < 0 {
            out_a |= bit << 1;
        } else if size[k] < a[k] {
            out_a |= bit;
        } else {
            inside |= bit;
            count += 1;
        }
        if b[k] < 0 {
            out_b |= bit << 1;
        } else if size[k] < b[k] {
            out_b |= bit;
        }
    }
    if count == 3 {
        return Some((0, a));
    }
    if out_a & out_b != 0 {
        return None;
    }
    let unit = |k: usize, far: bool| {
        let mut n = [0; 3];
        n[k] = if far { 4096 } else { -4096 };
        n
    };
    let mut on = |k: usize, far: bool| {
        *normal = unit(k, far);
        face(a, b, size, k, if far { size[k] } else { 0 })
    };
    match count {
        2 => {
            // One axis out: its own face.
            let k = match inside ^ 21 {
                1 => 0,
                4 => 1,
                16 => 2,
                _ => return None,
            };
            let bits = (out_a >> (2 * k)) & 3;
            match bits {
                1 => on(k, true),
                2 => on(k, false),
                _ => None,
            }
        }
        1 => {
            // Two out: the face of the one the segment crosses first, by a
            // cross product's sign (the axes in the original's order).
            let (i, j) = match inside {
                1 => (1, 2),
                4 => (0, 2),
                16 => (0, 1),
                _ => return None,
            };
            let (fi, fj) = ((out_a >> (2 * i)) & 3, (out_a >> (2 * j)) & 3);
            if !(1..=2).contains(&fi) || !(1..=2).contains(&fj) {
                return None;
            }
            let (far_i, far_j) = (fi == 1, fj == 1);
            let di = b[i].wrapping_sub(a[i]);
            let dj = b[j].wrapping_sub(a[j]);
            let to = |k: usize, far: bool| (if far { size[k] } else { 0 }).wrapping_sub(a[k]);
            // Which plane the segment reaches first, by the sign of
            // di (j's plane - a_j) - dj (i's plane - a_i), turned over when
            // one plane is the near side and the other the far.
            let s = di.wrapping_mul(to(j, far_j)).wrapping_sub(dj.wrapping_mul(to(i, far_i)));
            let i_first = if far_i == far_j { s <= 0 } else { s > 0 };
            if i_first { on(i, far_i) } else { on(j, far_j) }
        }
        _ => {
            // All three out: the z face, then the x, then the y; the
            // normal's sign by the plane (+ off 0, - at 0, so a flat box's
            // far face reads as its near one).
            let at = |k: usize| if (out_a >> (2 * k)) & 3 == 1 { size[k] } else { 0 };
            let hit = [2, 0, 1].into_iter().find_map(|k| face(a, b, size, k, at(k)).map(|h| (k, h)));
            hit.map(|(k, h)| {
                *normal = unit(k, at(k) != 0);
                h
            })
        }
    }
}

/// The rows' matrix of angles `a` as 0x80020ee0 builds it, MatrixNormal'd
/// (row r, column c).
fn matrix(trig: &Trig, a: [i16; 3]) -> [[i32; 3]; 3] {
    trig.matrix_normal(trig.euler(a))
}

/// The segment `from`..`to` against an object at `pos` turned by `angles`
/// with model boxes `boxes` ((lo, hi), each its own frame, as the model's
/// halfwords), 0x80069c40: the first box it meets, by index, with the point
/// and normal in the world. `normal` is the caller's buffer as the original
/// uses it: each box's test writes its face's normal there (in the box's
/// frame, hit or not, [`slab_into`]); a hit's is turned back into the
/// world, a start inside a box turning back whatever it held.
pub fn object(
    trig: &Trig,
    from: [i32; 3],
    to: [i32; 3],
    pos: [i32; 3],
    angles: [i16; 3],
    boxes: &[([i32; 3], [i32; 3])],
    normal: &mut [i32; 3],
) -> Option<(usize, Hit)> {
    let m = matrix(trig, angles);
    let mt = [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]));
    let n = normal;
    for (k, (lo, hi)) in boxes.iter().enumerate() {
        let size = [0, 1, 2].map(|i| hi[i] - lo[i]);
        let local = |p: [i32; 3]| {
            let q = trig.apply(&m, [p[0].wrapping_sub(pos[0]), p[1].wrapping_sub(pos[1]), p[2].wrapping_sub(pos[2])]);
            [q[0].wrapping_sub(lo[0]), q[1].wrapping_sub(lo[1]), q[2].wrapping_sub(lo[2])]
        };
        if let Some((t, point)) = slab_into(local(from), local(to), size, n) {
            let p = trig.apply(&mt, [point[0] + lo[0], point[1] + lo[1], point[2] + lo[2]]);
            *n = trig.apply(&mt, *n);
            return Some((k, Hit { t, point: [p[0] + pos[0], p[1] + pos[1], p[2] + pos[2]], normal: *n }));
        }
    }
    None
}

/// What an object is met by: its model's boxes (an object with a model,
/// +0x88), its hierarchy's one box (a character, 0x8006a008), or its
/// hierarchy's parts as posed (type flag 0x2000, 0x8006a318).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    Model(Vec<([i32; 3], [i32; 3])>),
    Body([i32; 3], [i32; 3]),
    Parts(Vec<Part>),
}

/// A hierarchy's part as posed (+0x158, 32 bytes a part): its matrix
/// (rows, 4096 = 1) and translation, and its model's boxes (none for a
/// part with no model).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    pub rot: [[i32; 3]; 3],
    pub trans: [i32; 3],
    pub boxes: Vec<([i32; 3], [i32; 3])>,
}

/// The segment against posed parts (0x8006a318): each part's matrix
/// applied to the segment less its translation, each box's slab test; the
/// nearest hit of all (not the first) kept, by part, its point and normal
/// turned back by the matrix's transpose. `normal` the caller's buffer
/// ([`object`]): a hit no nearer leaves the box's own-frame normal there.
pub fn parts(trig: &Trig, from: [i32; 3], to: [i32; 3], parts: &[Part], normal: &mut [i32; 3]) -> Option<(usize, Hit)> {
    let mut best = i32::MAX;
    let mut found = None;
    for (i, part) in parts.iter().enumerate() {
        let m = part.rot;
        let mt = [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]));
        let local = |p: [i32; 3]| trig.apply(&m, [0, 1, 2].map(|k| p[k].wrapping_sub(part.trans[k])));
        let (a, b) = (local(from), local(to));
        for (lo, hi) in &part.boxes {
            let size = [0, 1, 2].map(|k| hi[k] - lo[k]);
            let off = |p: [i32; 3]| [0, 1, 2].map(|k| p[k].wrapping_sub(lo[k]));
            let Some((t, point)) = slab_into(off(a), off(b), size, normal) else { continue };
            if t >= best {
                continue;
            }
            best = t;
            let p = trig.apply(&mt, [0, 1, 2].map(|k| point[k] + lo[k]));
            *normal = trig.apply(&mt, *normal);
            found = Some((i, Hit { t, point: [0, 1, 2].map(|k| p[k].wrapping_add(part.trans[k])), normal: *normal }));
        }
    }
    found
}

/// An object as the collision tests see it: its position (+0x18), angles
/// (+0x24), the half side of the cube that screens it first (the largest
/// of its bound's extents, +0x78), its size (+0x86: the floor probe looks
/// twice that above and below) and its shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solid {
    pub pos: [i32; 3],
    pub angles: [i16; 3],
    pub bound: i32,
    pub size: i32,
    pub shape: Shape,
}

impl Solid {
    /// Box `b` (0x80067b14): the model's, or the hierarchy's one.
    pub fn box_of(&self, b: u8) -> ([i32; 3], [i32; 3]) {
        match &self.shape {
            Shape::Model(boxes) => boxes.get(b as usize).copied().unwrap_or_default(),
            Shape::Body(lo, hi) => (*lo, *hi),
            // The part's model's first box (0x80067b14: its +0x58 model's
            // boxes).
            Shape::Parts(ps) => ps.get(b as usize).and_then(|p| p.boxes.first().copied()).unwrap_or_default(),
        }
    }

    /// The bound's cube half side (0x800698fc): the largest magnitude of
    /// the bound box's six extents.
    pub fn bound_of(lo: [i32; 3], hi: [i32; 3]) -> i32 {
        (0..3).map(|k| lo[k].abs().max(hi[k].abs())).max().unwrap_or(0)
    }

    /// The segment against the object (0x8006a714): screened by the
    /// bound's cube about its position (0x800698fc: any answer but a miss
    /// passes), then its boxes. The box met, its point and normal; `normal`
    /// the caller's buffer ([`object`]).
    pub fn meet(&self, trig: &Trig, from: [i32; 3], to: [i32; 3], normal: &mut [i32; 3]) -> Option<(usize, Hit)> {
        let r = self.bound;
        let corner = [0, 1, 2].map(|k| self.pos[k].wrapping_sub(r));
        let local = |p: [i32; 3]| [0, 1, 2].map(|k| p[k].wrapping_sub(corner[k]));
        let mut n = [0; 3];
        slab_into(local(from), local(to), [2 * r; 3], &mut n)?;
        match &self.shape {
            Shape::Model(boxes) => object(trig, from, to, self.pos, self.angles, boxes, normal),
            Shape::Body(lo, hi) => object(trig, from, to, self.pos, self.angles, &[(*lo, *hi)], normal),
            Shape::Parts(ps) => parts(trig, from, to, ps, normal),
        }
    }
}

/// An object as the player's probes meet it this frame: its solid, and
/// what the player does on meeting it: whether it can be climbed (its
/// type's flag 64, 0x80058da4) and its template (the bump's damage).
#[derive(Clone, Debug)]
pub struct Exact {
    pub solid: Solid,
    pub climb: bool,
    pub template: u16,
}
