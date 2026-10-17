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
/// and normal in the world. A segment starting inside a box meets it at its
/// start with the normal the boxes before it left (zero for the first).
pub fn object(trig: &Trig, from: [i32; 3], to: [i32; 3], pos: [i32; 3], angles: [i16; 3], boxes: &[([i32; 3], [i32; 3])]) -> Option<(usize, Hit)> {
    let m = matrix(trig, angles);
    let mt = [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]));
    // The normal as the boxes before leave it (a miss writes one too).
    let mut n = [0; 3];
    for (k, (lo, hi)) in boxes.iter().enumerate() {
        let size = [0, 1, 2].map(|i| hi[i] - lo[i]);
        let local = |p: [i32; 3]| {
            let q = trig.apply(&m, [p[0].wrapping_sub(pos[0]), p[1].wrapping_sub(pos[1]), p[2].wrapping_sub(pos[2])]);
            [q[0].wrapping_sub(lo[0]), q[1].wrapping_sub(lo[1]), q[2].wrapping_sub(lo[2])]
        };
        if let Some((t, point)) = slab_into(local(from), local(to), size, &mut n) {
            let p = trig.apply(&mt, [point[0] + lo[0], point[1] + lo[1], point[2] + lo[2]]);
            let normal = trig.apply(&mt, n);
            return Some((k, Hit { t, point: [p[0] + pos[0], p[1] + pos[1], p[2] + pos[2]], normal }));
        }
    }
    None
}
