//! libgte's fixed-point trigonometry, from its table in the executable
//! (rsin_tbl at 0x800bcd50: sin over a quarter turn, 1025 entries, 4096 =
//! 1), and the projection distance a camera's field of view gives.

use crate::disc::GameDisc;

const TABLE: u32 = 0x800b_cd50;
/// The arc cosine's table (0x800b4598: acos((4096 - k) / 4096), 4097
/// entries, 4096 to a turn), read by 0x80021500 and 0x80021460.
const ACOS: u32 = 0x800b_4598;
/// VectorNormal's table (0x800bd584: 1 / sqrt over 192 steps, read at the
/// sum of squares' top bits).
const RSQRT: u32 = 0x800b_d584;
/// ratan2's table (0x800c1710: atan over 0..1 in 1025 steps).
const ATAN: u32 = 0x800c_1710;

#[derive(Clone, Debug)]
pub struct Trig {
    sin: Vec<i16>,
    acos: Vec<i16>,
    rsqrt: Vec<i16>,
    atan: Vec<i16>,
}

impl Trig {
    pub fn load(disc: &GameDisc) -> Result<Trig, String> {
        let halves = |raw: &[u8]| raw.as_chunks::<2>().0.iter().map(|h| i16::from_le_bytes([h[0], h[1]])).collect();
        Ok(Trig {
            sin: halves(disc.exe_bytes(TABLE, 2 * 1025)?),
            acos: halves(disc.exe_bytes(ACOS, 2 * 4097)?),
            rsqrt: halves(disc.exe_bytes(RSQRT, 2 * 192)?),
            atan: halves(disc.exe_bytes(ATAN, 2 * 1025)?),
        })
    }

    /// atan2(a, b) in 4096 a turn (ratan2, 0x800a0328): the angle of the
    /// point (b, a) from +b toward +a, by a 1025-entry table; 0 for (0, 0).
    pub fn ratan2(&self, a: i32, b: i32) -> i32 {
        let (neg_b, b) = (b < 0, b.wrapping_abs());
        let (neg_a, a) = (a < 0, a.wrapping_abs());
        if a == 0 && b == 0 {
            return 0;
        }
        let t = |k: i32| self.atan.get(k as usize).copied().unwrap_or(0) as i32;
        let mut r = if a < b {
            let q = if a & 0x7fe0_0000 != 0 { a.wrapping_div(b >> 10) } else { (a << 10).wrapping_div(b) };
            t(q)
        } else {
            let q = if b & 0x7fe0_0000 != 0 { b.wrapping_div(a >> 10) } else { (b << 10).wrapping_div(a) };
            1024 - t(q)
        };
        if neg_b {
            r = 2048 - r;
        }
        if neg_a { -r } else { r }
    }

    /// The matrix of a heading `a` alone, as 0x80020d10 builds it for
    /// angles (a, 0, 0) and MatrixNormal (0x8009f398) leaves it: rows,
    /// 4096 = 1; forward (0, 0, 1) turns to (sin a, 0, cos a).
    pub fn heading_matrix(&self, a: i32) -> [[i32; 3]; 3] {
        let (s, c) = (self.sin(a), self.cos(a));
        self.matrix_normal([[c, 0, s], [0, 4096, 0], [-s, 0, c]])
    }

    /// MatrixNormal (0x8009f398): the third row the GTE's outer product of
    /// the first two, the first the second's with that (its IR kept to 16
    /// bits), then each row through VectorNormal.
    pub fn matrix_normal(&self, m: [[i32; 3]; 3]) -> [[i32; 3]; 3] {
        let m = m.map(|r| r.map(|c| c as i16 as i32));
        // OP, sf = 1: the diagonal d (the rotation's r11, r22, r33) across
        // the vector i.
        let op = |d: [i32; 3], i: [i32; 3]| {
            let (d, i) = (d.map(|c| c as i64), i.map(|c| c as i64));
            [d[1] * i[2] - d[2] * i[1], d[2] * i[0] - d[0] * i[2], d[0] * i[1] - d[1] * i[0]].map(|c| (c >> 12) as i32)
        };
        let third = op(m[0], m[1]);
        let ir = third.map(|c| c.clamp(-0x8000, 0x7fff));
        let first = op(m[1], ir);
        [self.normal(first), self.normal(m[1]), self.normal(third)]
    }

    /// m v (ApplyMatrixLV, 0x8009f6f8): each component split at bit 15,
    /// the high parts times m unshifted (then times 8), the low parts times
    /// m >> 12.
    pub fn apply(&self, m: &[[i32; 3]; 3], v: [i32; 3]) -> [i32; 3] {
        let split = |c: i32| if c < 0 { (-((-c) >> 15), -((-c) & 0x7fff)) } else { (c >> 15, c & 0x7fff) };
        let parts = v.map(split);
        [0, 1, 2].map(|r| {
            let hi: i64 = (0..3).map(|k| m[r][k] as i64 * parts[k].0 as i64).sum();
            let lo: i64 = (0..3).map(|k| m[r][k] as i64 * parts[k].1 as i64).sum();
            ((lo >> 12) + hi * 8) as i32
        })
    }

    /// `v` turned by the heading `a`'s matrix (0x80075ee8 for angles
    /// (a, 0, 0)).
    pub fn turn(&self, a: i32, v: [i32; 3]) -> [i32; 3] {
        self.apply(&self.heading_matrix(a), v)
    }

    /// libgte's VectorNormal (0x8009f278, 0x8009f2d8): `v` (its axes cut
    /// to 16 bits, as the GTE's IR registers hold them) as a unit, 4096
    /// long: the sum of squares' leading zeros (even) pick a step of the
    /// table, its value times each axis, shifted down by half the rest.
    pub fn normal(&self, v: [i32; 3]) -> [i32; 3] {
        let v = v.map(|c| c as i16 as i32);
        let sum = v.iter().map(|&c| c * c).fold(0i32, |a, b| a.wrapping_add(b));
        let lz = (if sum < 0 { (!sum).leading_zeros() } else { sum.leading_zeros() } as i32) & !1;
        let shift = (31 - lz) >> 1;
        let at = if lz >= 24 { sum << (lz - 24) } else { sum >> (24 - lz) };
        let k = (at - 64).clamp(0, 191) as usize;
        let r = self.rsqrt[k] as i32;
        v.map(|c| (c * r) >> shift)
    }

    /// The arc cosine of `x` (4096 = 1), 0..2048 (0x80021500).
    pub fn acos(&self, x: i32) -> i32 {
        let x = x.clamp(-4096, 4096);
        let t = |k: i32| self.acos[k as usize] as u16 as i32;
        match x {
            ..0 => (2048 - t(x + 4096)) as i16 as i32,
            0 => 1024,
            _ => t(4096 - x) as i16 as i32,
        }
    }

    /// The arc sine of `x` (4096 = 1), -1024..1024 (0x80021460).
    pub fn asin(&self, x: i32) -> i32 {
        let x = x.clamp(-4096, 4096);
        let t = |k: i32| self.acos[k as usize] as u16 as i32;
        match x {
            ..0 => (t(x + 4096) - 1024) as i16 as i32,
            0 => 0,
            _ => (1024 - t(4096 - x)) as i16 as i32,
        }
    }

    /// rsin (0x8009efd8): 4096 to a turn.
    pub fn sin(&self, a: i32) -> i32 {
        let t = |k: i32| self.sin[k as usize] as i32;
        let quarter = |a: i32| match a {
            0..=1024 => t(a),
            1025..=2048 => t(2048 - a),
            2049..=3072 => -t(a - 2048),
            _ => -t(4096 - a),
        };
        if a < 0 { -quarter(-a & 4095) } else { quarter(a & 4095) }
    }

    /// rcos (0x8009f0a8).
    pub fn cos(&self, a: i32) -> i32 {
        let a = a.unsigned_abs() as i32 & 4095;
        let t = |k: i32| self.sin[k as usize] as i32;
        match a {
            0..=1024 => t(1024 - a),
            1025..=2048 => -t(a - 1024),
            2049..=3072 => -t(3072 - a),
            _ => t(a - 3072),
        }
    }

    /// The GTE's projection distance H for a field of view `fov` (4096 to
    /// a turn), as 0x8001dfb8 sets it: 160 over the tangent of half the
    /// angle, in its integer steps. None outside 2..2048, where it does
    /// nothing.
    pub fn geom_screen(&self, fov: i32) -> Option<i32> {
        if !(2..2048).contains(&fov) {
            return None;
        }
        let half = fov / 2;
        let tan = (self.sin(half) << 12) / self.cos(half);
        // Kept, and given to SetGeomScreen, as 16 bits.
        Some((0xa_0000 / tan as i16 as i32) as i16 as i32)
    }
}

impl Trig {
    /// The rotation matrix angles (a, b, c) give, as 0x80020ee0 builds it
    /// (row-major, 4096 = 1, each product cut to 12 fractional bits in the
    /// original's order), before its MatrixNormal pass.
    pub fn euler(&self, angles: [i16; 3]) -> [[i32; 3]; 3] {
        let (a, b, c) = (angles[0] as i32, angles[1] as i32, angles[2] as i32);
        let (sa, sb, sc) = (self.sin(a), self.sin(b), self.sin(c));
        let (ca, cb, cc) = (self.cos(a), self.cos(b), self.cos(c));
        let m = |x: i32, y: i32| (x * y) / 4096;
        let (scsa, scca, ccsa, ccca) = (m(sc, sa), m(sc, ca), m(cc, sa), m(cc, ca));
        [
            [ccca + m(scsa, sb), m(sc, cb), -ccsa + m(scca, sb)],
            [-scca + m(ccsa, sb), m(cc, cb), scsa + m(ccca, sb)],
            [m(cb, sa), -sb, m(cb, ca)],
        ]
    }
}
