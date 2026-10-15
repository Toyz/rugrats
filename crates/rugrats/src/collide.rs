//! Collision against a database's height field (`LEVELnn.COL`,
//! [`rugrats_data::col`]): the tile under a point and its heights
//! (SLUS_006.50:0x8002b240), sloped through a plane (0x8002af20,
//! 0x80068510).

use rugrats_data::col::{COLUMNS, Col};

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

/// The flags of the tile a query found (COL record +4: bit 0 water in
/// level 4, 16 dust, 32 unknown).
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
