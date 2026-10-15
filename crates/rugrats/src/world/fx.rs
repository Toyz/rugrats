//! Meshes the executable keeps for effects (drawn by 0x800243c4 through
//! 0x80023c2c): level 5's remote beam (0x80090d34) and level 12's
//! flashlight (0x80090e70). A mesh is faces (a flag word each: the top two
//! bits 2 a triangle, 3 a quad; 0x20000000 semi-transparent), corner
//! indices, vertices (s16 x, y, z, pad) and per-corner indices into a
//! small palette (r, g, b, pad).

use rrt::glam::{Mat3, Vec3};
use rugrats_render::Vtx;

use crate::disc::GameDisc;

pub struct Mesh {
    faces: Vec<(u32, [u8; 4], [u8; 4])>,
    vertices: Vec<Vec3>,
    palette: Vec<[u8; 3]>,
}

/// Where a mesh's tables are: faces, vertices, flags, corner indices,
/// vertices, colour indices, palette (the descriptor 0x80090d34 builds).
pub struct Tables {
    pub faces: usize,
    pub vertices: usize,
    pub flags: u32,
    pub corners: u32,
    pub at: u32,
    pub colours: u32,
    pub palette: u32,
    pub palette_len: usize,
}

/// The beam (0x80090d34): 36 faces over 26 vertices, three nested
/// spindles along z, a white core in a red glow.
pub const BEAM: Tables = Tables {
    faces: 36,
    vertices: 26,
    flags: 0x800b_b0ec,
    corners: 0x800b_afcc,
    at: 0x800b_aefc,
    colours: 0x800b_b05c,
    palette: 0x800b_aee4,
    palette_len: 6,
};

/// The flashlight (0x80090e70): 12 faces over 13 vertices, an additive
/// grey cone, scale 15000.
pub const LIGHT: Tables = Tables {
    faces: 12,
    vertices: 13,
    flags: 0x800b_b25c,
    corners: 0x800b_b1fc,
    at: 0x800b_b194,
    colours: 0x800b_b22c,
    palette: 0x800b_b17c,
    palette_len: 6,
};

impl Mesh {
    pub fn load(disc: &GameDisc, t: &Tables) -> Result<Mesh, String> {
        let words = |at: u32, n: usize| -> Result<Vec<u32>, String> {
            Ok(disc
                .exe_bytes(at, 4 * n)?
                .as_chunks::<4>()
                .0
                .iter()
                .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
                .collect())
        };
        let flags = words(t.flags, t.faces)?;
        let corners = words(t.corners, t.faces)?;
        let colours = words(t.colours, t.faces)?;
        let faces = (0..t.faces).map(|k| (flags[k], corners[k].to_le_bytes(), colours[k].to_le_bytes())).collect();
        let raw = disc.exe_bytes(t.at, 8 * t.vertices)?;
        let vertices = raw
            .as_chunks::<8>()
            .0
            .iter()
            .map(|v| {
                let s = |i: usize| i16::from_le_bytes([v[i], v[i + 1]]) as f32;
                Vec3::new(s(0), s(2), s(4))
            })
            .collect();
        let palette =
            words(t.palette, t.palette_len)?.into_iter().map(|w| [w as u8, (w >> 8) as u8, (w >> 16) as u8]).collect();
        Ok(Mesh { faces, vertices, palette })
    }

    /// The mesh stretched from `from` to `to` (its z along the segment,
    /// scaled by the length / 4096, 0x8002474c) and `width` / 4096 across,
    /// centred on the midpoint.
    pub fn along(&self, from: Vec3, to: Vec3, width: f32, out: &mut Vec<Vtx>) {
        let d = to - from;
        let len = d.length();
        if len < 1.0 {
            return;
        }
        let z = d / len;
        let x = if z.y.abs() < 0.9 { Vec3::Y.cross(z).normalize() } else { Vec3::X.cross(z).normalize() };
        let y = z.cross(x);
        let rot = Mat3::from_cols(x * width / 4096.0, y * width / 4096.0, z * len / 4096.0);
        let mid = (from + to) / 2.0;
        let world: Vec<Vec3> = self.vertices.iter().map(|v| mid + rot * *v).collect();
        for &(flags, c, k) in &self.faces {
            let semi = if flags & 0x2000_0000 != 0 { Vtx::SEMI | 1 << 21 } else { 0 };
            let v = |i: usize| {
                let p = world.get(c[i] as usize).copied().unwrap_or(mid);
                let rgb = self.palette.get(k[i] as usize).copied().unwrap_or([128; 3]);
                Vtx {
                    pos: p.to_array(),
                    colour: rgb[0] as u32 | (rgb[1] as u32) << 8 | (rgb[2] as u32) << 16,
                    uv: 0,
                    mode: Vtx::FLAT | semi,
                    window: 0,
                }
            };
            // Both windings: the world pass culls back faces.
            match flags >> 30 {
                2 => out.extend([v(0), v(1), v(2), v(0), v(2), v(1)]),
                3 => out.extend([v(0), v(1), v(2), v(1), v(3), v(2), v(0), v(2), v(1), v(1), v(2), v(3)]),
                _ => {}
            }
        }
    }
}
