//! Collision against a database's height field (`LEVELnn.COL`,
//! [`rugrats_data::col`]): the tile under a point and its heights
//! (SLUS_006.50:0x8002b240), sloped through a plane (0x8002af20,
//! 0x80068510), and the walk of a segment through the tiles
//! ("vectorwalk", 0x8002cd7c).

use rugrats_data::col::{COLUMNS, Col, Column};

/// What 0x8002b240 answers for a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Query {
    /// 2 outside the field, 1 or 0 inside (0: in the open, between the
    /// tile's ceiling and its floor).
    pub kind: i32,
    /// The ceiling height (its second output) and the floor (its third).
    /// The PlayStation's y points down: the floor is the larger.
    pub ceiling: i32,
    pub floor: i32,
    /// The tile found: (column, index into its tiles).
    pub tile: Option<(usize, usize)>,
}

/// Rounds a signed quotient by 1024 toward zero, as `(v + 1023) >> 10`
/// for negatives.
fn cell(v: i32) -> i32 {
    if v < 0 { (v + 1023) >> 10 } else { v >> 10 }
}

/// `lo(a * b)`: the low word of a product.
fn mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// A vertical line through `(x, z)` from `y0` to `y1` against the plane
/// through `p` with normal `n` (0x80068510): the y where they meet, None
/// when the line runs along a plane it misses.
fn plane_y(line: [[i32; 3]; 2], p: [i32; 3], n: [i32; 3]) -> Option<i32> {
    let d = [line[1][0] - line[0][0], line[1][1] - line[0][1], line[1][2] - line[0][2]];
    let a2 = (mul(n[0], d[0]) >> 12) + (mul(n[1], d[1]) >> 12) + (mul(n[2], d[2]) >> 12);
    let a1 = (mul(n[0], p[0] - line[0][0]) >> 12)
        + (mul(n[1], p[1] - line[0][1]) >> 12)
        + (mul(n[2], p[2] - line[0][2]) >> 12);
    if a2 == 0 {
        return (a1 == 0).then_some(line[0][1]);
    }
    Some(line[0][1] + mul(d[1], a1) / a2)
}

impl Query {
    pub fn outside(y: i32) -> Query {
        Query { kind: 2, ceiling: y, floor: y, tile: None }
    }
}

/// A tile's height (0x8002af20): its floor (bytes 0, 1), else its
/// ceiling (bytes 2, 3); flat as max y - byte * 128, or through the plane
/// at the tile's corner when it names a normal.
pub fn height(col: &Col, pos: [i32; 3], t: &[u8; 8], floor: bool) -> i32 {
    let (byte, normal) = if floor { (t[0], t[1]) } else { (t[2], t[3]) };
    let flat = col.max[1] - ((byte as i32) << 7);
    if normal == 0 {
        return flat;
    }
    let normals = if floor { &col.floor_normals } else { &col.ceiling_normals };
    let Some(n) = normals.get(normal as usize) else { return flat };
    // A tile split one way slopes from its far corner.
    let corner = |v: i32| if (t[6] & 96) >> 5 == 3 { v | 1023 } else { v & -1024 };
    let p = [corner(pos[0]), flat, corner(pos[2])];
    let line = [[pos[0], flat - 1024, pos[2]], [pos[0], flat + 1024, pos[2]]];
    plane_y(line, p, n.map(|v| v as i32)).unwrap_or(flat)
}

/// The flags of the tile a query found (COL record +4: 1 water in level 4,
/// 2 slippery, 4 a puff at a foot, 16 dust, 32 and 64 how deep a character
/// sinks as drawn; 8 is read by nothing found).
pub fn floor_flags(col: &Col, q: &Query) -> u8 {
    q.tile.and_then(|(c, k)| col.columns.get(c).and_then(|c| c.tiles.get(k))).map_or(0, |t| t[4])
}

/// The tile under `pos` and its heights (0x8002b240).
pub fn query(col: &Col, pos: [i32; 3]) -> Query {
    let inside = (0..3).all(|k| col.min[k] <= pos[k] && pos[k] <= col.max[k]);
    if !inside {
        return Query::outside(pos[1]);
    }
    let x = cell(pos[0] - col.min[0]);
    let z = cell(pos[2] - col.min[2]);
    let none = Query { kind: 1, ceiling: pos[1], floor: pos[1], tile: None };
    let Some(column) =
        usize::try_from(x).ok().filter(|&x| x < COLUMNS).and_then(|x| col.columns.get(x).map(|c| (x, c)))
    else {
        return none;
    };
    let (cx, column) = column;
    let Some(entry) = search(&column.cells, z) else { return none };
    let mut k = entry[1] as usize;
    let tile = |k: usize| column.tiles.get(k);
    let Some(&first) = tile(k) else { return none };
    // A halved tile: which half the point is in.
    let mut half = -1i32;
    if first[6] & 128 != 0 {
        let (fx, fz) = (pos[0] & 1023, pos[2] & 1023);
        half =
            if (first[6] >> 5) & 1 != 0 { if fx + fz < 1024 { 1 } else { 3 } } else { (((fz < fx) as i32) ^ 1) << 1 };
        // The stack's tile for that half, or none.
        loop {
            let Some(t) = tile(k) else { return none };
            if ((t[6] & 96) >> 5) as i32 == half {
                break;
            }
            if t[6] & 1 != 0 {
                return none;
            }
            k += 1;
        }
    }
    // Down the stack (from the highest) to the first tile whose ceiling
    // the point is at or below.
    let mut found = None;
    loop {
        let Some(&t) = tile(k) else { break };
        if pos[1] >= height(col, pos, &t, false) {
            found = Some(k);
            break;
        }
        if t[6] & 1 != 0 {
            break;
        }
        k += 1;
        if half > 0 && tile(k).is_some_and(|t| ((t[6] & 96) >> 5) as i32 != half) {
            break;
        }
    }
    let Some(k) = found else { return none };
    let t = *tile(k).unwrap();
    let floor = height(col, pos, &t, true);
    let ceiling = height(col, pos, &t, false);
    let kind = if ceiling < pos[1] && pos[1] < floor { 0 } else { 1 };
    Query { kind, ceiling, floor, tile: Some((cx, k)) }
}

/// The cell entry for z cell `z`, by halves (0x8002aec4).
fn search(cells: &[[u8; 2]], z: i32) -> Option<[u8; 2]> {
    let (mut base, mut n) = (0usize, cells.len());
    while n != 0 {
        let half = n >> 1;
        let c = cells[base + half];
        let d = z - c[0] as i32;
        if d < 0 {
            n = half;
        } else if d == 0 {
            return Some(c);
        } else {
            base += half + 1;
            n = n - 1 - half;
        }
    }
    None
}

/// What the segment test (0x8002cd7c) writes through its four output
/// pointers. A field it does not write keeps what the caller put there, as
/// in the original: a walk that gets through (0) writes only `tile`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Segment {
    /// Its third argument: how far the start is below its floor (start y -
    /// floor) when the start is not in the open, 0 when a wall stops the
    /// walk; a floor or ceiling hit leaves it.
    pub depth: i32,
    /// Its fourth: where the walk stopped (the start when it never left).
    pub point: [i32; 3],
    /// Its first stack argument: the normal of what stopped it (4.12): a
    /// tile side, a diagonal, a floor or ceiling plane; for a start not in
    /// the open, from - to normalized.
    pub normal: [i32; 3],
    /// Its second: the tile it was last in, or stopped by.
    pub tile: Option<(usize, usize)>,
}

/// The normals of a cell's sides (0x800b7018), by how the step into the
/// cell moved: 0 toward -z, 1 +z, 2 -x, 3 +x, 4 not yet (the first cell).
/// Each points back against the move.
const SIDES: [[i32; 3]; 5] = [[0, 0, 4096], [0, 0, -4096], [4096, 0, 0], [-4096, 0, 0], [0, 0, 0]];

/// The normals of a halved tile's diagonal, facing back into the half a
/// step leaves (0x800b7068).
const DIAGONALS: [[i32; 3]; 5] = [[2896, 0, -2896], [-2896, 0, -2896], [-2896, 0, 2896], [2896, 0, 2896], [0, 0, 0]];

/// Walks the segment `from` to `to` through the height field, a cell at a
/// time, until something stops it (0x8002cd7c, "vectorwalk"). Returns
///
/// - **0:** it gets through (or `from == to`, which zeroes the outputs),
/// - **1:** a wall, a missing tile, a diagonal, a floor or a ceiling stops
///   it at `out.point`,
/// - **2:** `from` is not in the open (inside a wall, under a floor, or
///   outside the field): nothing walks.
pub fn segment(col: &Col, from: [i32; 3], to: [i32; 3], out: &mut Segment) -> i32 {
    let start = query(col, from);
    if start.kind != 0 {
        out.normal = normalize(sub(from, to));
        out.depth = from[1].wrapping_sub(start.floor);
        out.point = from;
        out.tile = start.tile;
        return 2;
    }
    if from == to {
        *out = Segment { depth: 0, point: [0; 3], normal: [0; 3], tile: start.tile };
        return 0;
    }
    // The point so far, and the side its cell was entered through.
    let (mut cur, mut side) = (from, 4);
    loop {
        let d = sub(to, cur);
        let (next, next_side) = step(cur, to, d, side);
        let r = in_cell(col, cur, next, d, side, out);
        if r != 0 {
            return r;
        }
        cur = next;
        side = next_side;
        // On while any axis that moves has not reached `to`.
        let more = |k: usize| (d[k] > 0 && cur[k] < to[k]) || (d[k] < 0 && to[k] < cur[k]);
        if !(more(0) || more(1) || more(2)) {
            return 0;
        }
    }
}

fn sub(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0].wrapping_sub(b[0]), a[1].wrapping_sub(b[1]), a[2].wrapping_sub(b[2])]
}

/// `lo / b`: the R3000's division, toward zero.
fn div(a: i32, b: i32) -> i32 {
    a.wrapping_div(b)
}

/// Rounds a 4.12 product toward zero, as `(v + 4095) >> 12` for negatives.
fn fixed(v: i32) -> i32 {
    if v < 0 { v.wrapping_add(4095) >> 12 } else { v >> 12 }
}

/// The next point of the walk from `cur` toward `to` (`d` = to - cur): to
/// the edge of the cell in the axis the line leaves it by first, or to
/// `to` within it, and the side the next cell is entered through (the
/// side stays when the walk is vertical).
fn step(cur: [i32; 3], to: [i32; 3], d: [i32; 3], side: i32) -> ([i32; 3], i32) {
    let (cx, cz) = (cur[0] & -1024, cur[2] & -1024);
    // The first coordinate past the cell, each way.
    let bx = if d[0] > 0 { cx + 1024 } else { cx - 1 };
    let bz = if d[2] > 0 { cz + 1024 } else { cz - 1 };
    // Along x to the edge (or to `to`): z kept within the cell.
    let x_step = |clamp_z: bool| {
        let nx = if d[0] > 0 { bx.min(to[0]) } else { bx.max(to[0]) };
        let along = nx.wrapping_sub(cur[0]);
        let mut z = cur[2].wrapping_add(div(mul(along, d[2]), d[0]));
        if clamp_z {
            z = if d[2] > 0 { z.min(cz + 1023) } else { z.max(cz) };
        }
        let y = cur[1].wrapping_add(div(mul(along, d[1]), d[0]));
        ([nx, y, z], if d[0] > 0 { 3 } else { 2 })
    };
    let z_step = |clamp_x: bool| {
        let nz = if d[2] > 0 { bz.min(to[2]) } else { bz.max(to[2]) };
        let along = nz.wrapping_sub(cur[2]);
        let mut x = cur[0].wrapping_add(div(mul(along, d[0]), d[2]));
        if clamp_x {
            x = if d[0] > 0 { x.min(cx + 1023) } else { x.max(cx) };
        }
        let y = cur[1].wrapping_add(div(mul(along, d[1]), d[2]));
        ([x, y, nz], if d[2] > 0 { 1 } else { 0 })
    };
    match (d[0], d[2]) {
        (0, 0) => (to, side),
        (_, 0) => x_step(false),
        (0, _) => z_step(false),
        _ => {
            // Which edge comes first: where the line meets the other
            // axis's edge, from the axis it moves less in.
            let x_first = if d[0].wrapping_abs() < d[2].wrapping_abs() {
                let x = cur[0].wrapping_add(div(mul(bz.wrapping_sub(cur[2]), d[0]), d[2]));
                if d[0] > 0 { cx + 1023 < x } else { x < cx }
            } else {
                let z = cur[2].wrapping_add(div(mul(bx.wrapping_sub(cur[0]), d[2]), d[0]));
                !(if d[2] > 0 { cz + 1023 < z } else { z < cz })
            };
            if x_first { x_step(true) } else { z_step(true) }
        }
    }
}

/// One step of the walk, `cur` to `next` inside `cur`'s cell, which was
/// entered through `side` (0x8002cb70): across the diagonal of a halved
/// tile when it crosses it, then through the stack of tiles.
fn in_cell(col: &Col, cur: [i32; 3], next: [i32; 3], d: [i32; 3], side: i32, out: &mut Segment) -> i32 {
    let side_normal = SIDES[side.clamp(0, 4) as usize];
    match diagonal(col, cur, next, d) {
        Halves::Whole(tile, half) => through(col, cur, next, Some(tile), half, side_normal, out),
        Halves::Split { first, half, mid, second, other } => {
            let r = through(col, cur, mid, Some(first), half, side_normal, out);
            if r != 0 {
                return r;
            }
            through(col, mid, next, second, other, DIAGONALS[half as usize], out)
        }
        // No tile here: stopped where it came in.
        Halves::None => {
            out.depth = 0;
            out.point = cur;
            out.normal = if (0..4).contains(&side) { SIDES[side as usize] } else { sub(next, cur) };
            out.tile = None;
            1
        }
    }
}

/// What 0x8002bbdc finds of a step in its cell.
enum Halves {
    /// No tile for the start (outside the field, no cell, or no tile for
    /// its half).
    None,
    /// The tile the step stays in and its half, 4 for a whole tile.
    Whole((usize, usize), i32),
    /// The step crosses the diagonal at `mid` from `half` of `first` to
    /// `other`, whose tile the stack may not have.
    Split { first: (usize, usize), half: i32, mid: [i32; 3], second: Option<(usize, usize)>, other: i32 },
}

/// The half of a halved tile a point is in, by the tile's flags bit 5.
fn half_at(t: &[u8; 8], x: i32, z: i32) -> i32 {
    let (fx, fz) = (x & 1023, z & 1023);
    if (t[6] >> 5) & 1 != 0 { if fx + fz < 1024 { 1 } else { 3 } } else { (((fz < fx) as i32) ^ 1) << 1 }
}

fn half_of(t: &[u8; 8]) -> i32 {
    ((t[6] & 0x60) >> 5) as i32
}

/// The first tile of the stack under `pos` (0x8002b114): `pos` must be
/// inside the field's bounds, y included.
fn stack(col: &Col, pos: [i32; 3]) -> Option<(usize, usize)> {
    if !(0..3).all(|k| col.min[k] <= pos[k] && pos[k] <= col.max[k]) {
        return None;
    }
    let x = usize::try_from(cell(pos[0] - col.min[0])).ok().filter(|&x| x < COLUMNS)?;
    let column = col.columns.get(x)?;
    let entry = search(&column.cells, cell(pos[2] - col.min[2]))?;
    Some((x, entry[1] as usize))
}

/// The first tile of the stack from `k` on for half `h`.
fn find_half(column: &Column, mut k: usize, h: i32) -> Option<usize> {
    loop {
        let t = column.tiles.get(k)?;
        if half_of(t) == h {
            return Some(k);
        }
        if t[6] & 1 != 0 {
            return None;
        }
        k += 1;
    }
}

/// The step `p` to `q` against its cell's diagonal (0x8002bbdc; `d` is the
/// whole walk's to - start of the step, which picks the axis y follows).
fn diagonal(col: &Col, p: [i32; 3], q: [i32; 3], d: [i32; 3]) -> Halves {
    let Some((c, first)) = stack(col, p) else { return Halves::None };
    let column = &col.columns[c];
    let Some(t) = column.tiles.get(first) else { return Halves::None };
    if t[6] & 0x80 == 0 {
        return Halves::Whole((c, first), 4);
    }
    let half = half_at(t, p[0], p[2]);
    let Some(k) = find_half(column, first, half) else { return Halves::None };
    if d[0] == 0 && d[2] == 0 {
        return Halves::Whole((c, k), half);
    }
    let (cx, cz) = (p[0] & -1024, p[2] & -1024);
    // The diagonal (from a to b), whether the step heads at it, and where
    // on the diagonal's own cells a crossing at x lands.
    let (a, b, heads) = match half {
        0 => ([cx, cz], [cx + 1023, cz + 1023], d[0] < d[2]),
        1 => ([cx, cz + 1023], [cx + 1023, cz], d[0].wrapping_neg() < d[2]),
        2 => ([cx, cz], [cx + 1023, cz + 1023], d[0] >= d[2]),
        _ => ([cx, cz + 1023], [cx + 1023, cz], d[0].wrapping_neg() >= d[2]),
    };
    let crossing = if heads { cross_x(p, q, a, b) } else { None };
    let Some(x) = crossing else { return Halves::Whole((c, k), half) };
    let (x, z) = match half {
        0 => {
            let x = if x < cx { cx } else { (cx + 1022).min(x) };
            (x, cz + (x - cx) + 1)
        }
        1 => {
            let x = if x < cx + 1 { cx + 1 } else { (cx + 1023).min(x) };
            (x, cz + 1023 - (x - cx - 1))
        }
        2 => {
            let x = if x < cx + 1 { cx + 1 } else { (cx + 1023).min(x) };
            (x, cz + (x - cx - 1))
        }
        _ => {
            let x = if x < cx { cx } else { (cx + 1022).min(x) };
            (x, cz + 1023 - (x - cx + 1))
        }
    };
    let y = if d[0].wrapping_abs() < d[2].wrapping_abs() {
        p[1].wrapping_add(div(mul(z.wrapping_sub(p[2]), d[1]), d[2]))
    } else {
        p[1].wrapping_add(div(mul(x.wrapping_sub(p[0]), d[1]), d[0]))
    };
    let other = [2, 3, 0, 1][half as usize];
    let second = find_half(column, first, other).map(|k| (c, k));
    Halves::Split { first: (c, k), half, mid: [x, y, z], second, other }
}

/// Where the step `p` to `q` crosses the line `a` to `b` (x, z), as its x
/// (0x8002baa0): None when it does not within the step, `p`'s x when they
/// run along each other.
fn cross_x(p: [i32; 3], q: [i32; 3], a: [i32; 2], b: [i32; 2]) -> Option<i32> {
    let (bx, bz) = (b[0].wrapping_sub(a[0]), b[1].wrapping_sub(a[1]));
    let num = fixed(mul(p[2].wrapping_sub(a[1]), bx)).wrapping_sub(fixed(mul(p[0].wrapping_sub(a[0]), bz)));
    let dx = q[0].wrapping_sub(p[0]);
    let den = fixed(mul(dx, bz)).wrapping_sub(fixed(mul(q[2].wrapping_sub(p[2]), bx)));
    if den == 0 {
        return (num == 0).then_some(p[0]);
    }
    let s = div(num.wrapping_shl(12), den);
    if s as u32 >= 4097 {
        return None;
    }
    Some(p[0].wrapping_add(fixed(mul(dx, s))))
}

/// A step `p` to `q` inside one half (or whole) cell against its stack of
/// tiles from `tile` (0x8002c720): the tile whose ceiling `p` is under, a
/// wall when there is none or `p` is below its floor, else its floor and
/// ceiling planes. `side` is the normal a wall answers with.
fn through(
    col: &Col,
    p: [i32; 3],
    q: [i32; 3],
    tile: Option<(usize, usize)>,
    half: i32,
    side: [i32; 3],
    out: &mut Segment,
) -> i32 {
    let mut wall = |tile| {
        out.depth = 0;
        out.point = p;
        out.normal = side;
        out.tile = tile;
        1
    };
    let Some((c, mut k)) = tile else { return wall(None) };
    let tiles = &col.columns[c].tiles;
    let t = |k: usize| tiles.get(k).copied().unwrap_or([0, 0, 0, 0, 0, 0, 1, 0]);
    let last = |k: usize| t(k)[6] & 1 != 0 || k + 1 >= tiles.len();
    let other_half = |k: usize| half != 4 && half_of(&t(k)) != half;
    let flat = |byte: u8| col.max[1] - ((byte as i32) << 7);
    // Down the stack past tiles whose flat ceiling is well above `p`.
    while !last(k) && p[1] < flat(t(k)[2]) - 1024 && !other_half(k) {
        k += 1;
    }
    if p[1] < flat(t(k)[2]) - 1024 || other_half(k) {
        return wall(Some((c, k)));
    }
    // Then by the ceiling's true height.
    let ceiling = |k: usize| height(col, p, &t(k), false);
    while !last(k) && p[1] < ceiling(k) && !other_half(k) {
        k += 1;
    }
    if p[1] < ceiling(k) || other_half(k) {
        return wall(Some((c, k)));
    }
    let here = t(k);
    // Well clear of both: through without a plane test.
    let floor_flat = flat(here[0]) - 1024;
    if p[1] < floor_flat && q[1] < floor_flat && flat(here[2]) + 1024 < q[1] {
        out.tile = Some((c, k));
        return 0;
    }
    if height(col, p, &here, true) < p[1] {
        return wall(Some((c, k)));
    }
    out.tile = Some((c, k));
    match planes(col, p, q, &here) {
        Some((point, normal)) => {
            out.point = point;
            out.normal = normal;
            1
        }
        None => 0,
    }
}

/// The step `p` to `q` against a tile's floor and ceiling planes
/// (0x8002c3a0): the nearer one it crosses, where, and its normal.
fn planes(col: &Col, p: [i32; 3], q: [i32; 3], t: &[u8; 8]) -> Option<([i32; 3], [i32; 3])> {
    let corner = |v: i32| if half_of(t) == 3 { v | 1023 } else { v & -1024 };
    let normal = |normals: &[[i16; 3]], k: u8| normals.get(k as usize).map_or([0; 3], |n| n.map(|v| v as i32));
    let hit = |byte: u8, n: [i32; 3]| {
        let plane = [corner(p[0]), col.max[1] - ((byte as i32) << 7), corner(p[2])];
        plane_hit([p, q], plane, n).filter(|&(_, s)| (0..4097).contains(&s)).map(|(at, s)| (at, s, n))
    };
    let floor = hit(t[0], normal(&col.floor_normals, t[1]));
    let ceiling = hit(t[2], normal(&col.ceiling_normals, t[3]));
    let s = |h: &Option<([i32; 3], i32, [i32; 3])>| h.map_or(i32::MAX, |h| h.1);
    let nearer = if s(&ceiling) < s(&floor) { ceiling } else { floor };
    nearer.map(|(at, _, n)| (at, n))
}

/// Where the line `line[0]` to `line[1]` meets the plane through `p` with
/// normal `n` (0x80068510): the point and how far along (4.12), or None
/// when it runs along a plane it misses (along the plane: its start, 0).
fn plane_hit(line: [[i32; 3]; 2], p: [i32; 3], n: [i32; 3]) -> Option<([i32; 3], i32)> {
    let d = sub(line[1], line[0]);
    let dot =
        |v: [i32; 3]| (mul(n[0], v[0]) >> 12).wrapping_add(mul(n[1], v[1]) >> 12).wrapping_add(mul(n[2], v[2]) >> 12);
    let (a2, a1) = (dot(d), dot(sub(p, line[0])));
    if a2 == 0 {
        return (a1 == 0).then_some((line[0], 0));
    }
    let at = |k: usize| line[0][k].wrapping_add(div(mul(d[k], a1), a2));
    Some(([at(0), at(1), at(2)], div(a1.wrapping_shl(12), a2)))
}

/// `VectorNormal` (0x8009f278, on the GTE): `v` scaled to length 4096,
/// from each component's low 16 bits as the GTE's IR registers hold them.
/// The scale is 1 / sqrt by a 192-entry table (0x800bd584), which is
/// floor(sqrt(2^30 / i)) for i = 64..256. When the squares pass 2^31 (two
/// components near 32768) the original's `add` (0x8009f2fc) raises an
/// overflow exception; this answers zero there.
pub fn normalize(v: [i32; 3]) -> [i32; 3] {
    let v = v.map(|c| c as i16 as i32);
    let Ok(sum) = i32::try_from(v.iter().map(|&c| (c * c) as i64).sum::<i64>()) else { return [0; 3] };
    if sum == 0 {
        return [0; 3];
    }
    let lz = sum.leading_zeros() & !1;
    let i = if lz >= 24 { sum << (lz - 24) } else { sum >> (24 - lz) };
    let scale = ((1u64 << 30) / i as u64).isqrt() as i32;
    let shift = (31 - lz as i32) >> 1;
    v.map(|c| (c * scale) >> shift)
}

/// 0x8007652c: [`normalize`] after shifting `v` down so its largest axis,
/// shifted by 12, fits: by 32 less the leading zeros of max |c| >> 12.
pub fn unit(v: [i32; 3]) -> [i32; 3] {
    let m = v.iter().map(|c| c.unsigned_abs()).max().unwrap_or(0) >> 12;
    let shift = 32 - m.leading_zeros();
    normalize(v.map(|c| c >> shift))
}

/// 0x80076784: |v| by the integer square root, 0 when any axis is 65536
/// or more.
pub fn length(v: [i32; 3]) -> i32 {
    if v.iter().any(|c| c.unsigned_abs() >> 16 != 0) {
        return 0;
    }
    let sq = v.iter().map(|&c| c.wrapping_mul(c) as u32).fold(0u32, |a, b| a.wrapping_add(b));
    sq.isqrt() as i16 as i32
}

/// `a * b >> 12` rounded toward zero, as the game's `(x + 4095) >> 12`
/// for negatives.
pub fn fix(a: i32, b: i32) -> i32 {
    let x = a.wrapping_mul(b);
    if x < 0 { (x + 4095) >> 12 } else { x >> 12 }
}

/// 0x800763d0's direction: `a` along the plane of normal `n`, as a unit
/// vector (n (n . a) - a with both made unit first).
pub fn along_plane(a: [i32; 3], n: [i32; 3]) -> [i32; 3] {
    let (a, n) = (normalize(a), normalize(n));
    let d = fix(n[0], a[0]) + fix(n[1], a[1]) + fix(n[2], a[2]);
    normalize([0, 1, 2].map(|k| fix(d, n[k]) - a[k]))
}

/// The tile's floor plane normal under `pos` (COL record byte +1), or
/// straight up (0, -4096, 0) when it names none or there is no tile.
pub fn floor_normal(col: &Col, pos: [i32; 3]) -> [i32; 3] {
    let q = query(col, pos);
    q.tile
        .and_then(|(c, k)| col.columns.get(c).and_then(|c| c.tiles.get(k)))
        .filter(|t| t[1] != 0)
        .and_then(|t| col.floor_normals.get(t[1] as usize))
        .map_or([0, -4096, 0], |n| n.map(|v| v as i32))
}
