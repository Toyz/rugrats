//! libgte's fixed-point trigonometry, from its table in the executable
//! (rsin_tbl at 0x800bcd50: sin over a quarter turn, 1025 entries, 4096 =
//! 1), and the projection distance a camera's field of view gives.

use crate::disc::GameDisc;

const TABLE: u32 = 0x800b_cd50;

#[derive(Clone, Debug)]
pub struct Trig {
    sin: Vec<i16>,
}

impl Trig {
    pub fn load(disc: &GameDisc) -> Result<Trig, String> {
        let raw = disc.exe_bytes(TABLE, 2 * 1025)?;
        Ok(Trig { sin: raw.chunks_exact(2).map(|h| i16::from_le_bytes([h[0], h[1]])).collect() })
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
