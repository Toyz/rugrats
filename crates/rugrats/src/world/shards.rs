//! An object breaking into its faces (SLUS_006.50:0x8004cdfc): each flat
//! textured face of the model (sections 0x24 and 0x2c, from the first;
//! the walk stops at the first section of another kind, 32 faces a
//! section at most) is cut in four, again and again, until no edge spans
//! more than 1024 or the object's radius on any axis (0x8004c2e8 for a
//! triangle, 0x8004b850 for a quad). Each piece takes one of ten slots
//! (0x800e2788, 96 bytes each, 0x8004ab18) and flies as a kind 0 flight
//! (0x8004b580, 0x8004b2b0): away from the point struck, from the piece's
//! centre, turning about a random axis. Pieces past the tenth are not made.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::model::{Face, Model, Shading};

/// The pieces' slots (0x800e2788).
pub const SLOTS: usize = 10;
/// A section's faces at most (0x8004cce4).
const SECTION: usize = 32;
/// The longest a piece's edge may span on an axis (0x8004c2e8).
const SPAN: f32 = 1024.0;

/// A piece in flight: its face about its own centre, as a one-face model,
/// and the object's turn when it broke.
#[derive(Clone, Debug)]
pub struct Shard {
    pub model: Model,
    pub base: Mat3,
}

/// A face being cut: its corners' places in the model, texels and colours.
#[derive(Clone, Copy, Debug)]
struct Piece {
    at: [Vec3; 4],
    uv: [[u8; 2]; 4],
    rgb: [[u8; 3]; 4],
    quad: bool,
}

fn mid(a: Vec3, b: Vec3) -> Vec3 {
    (a + b) / 2.0
}

fn mid_uv(a: [u8; 2], b: [u8; 2]) -> [u8; 2] {
    [((a[0] as u16 + b[0] as u16) >> 1) as u8, ((a[1] as u16 + b[1] as u16) >> 1) as u8]
}

fn mid_rgb(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    [0, 1, 2].map(|i| ((a[i] as u16 + b[i] as u16) >> 1) as u8)
}

impl Piece {
    fn corners(&self) -> usize {
        if self.quad { 4 } else { 3 }
    }

    /// No edge spans more than `limit` on any axis.
    fn small(&self, limit: f32) -> bool {
        let n = self.corners();
        let edges = (0..n).map(|i| (self.at[i], self.at[(i + 1) % n]));
        let span = edges.fold(Vec3::ZERO, |m, (a, b)| m.max((a - b).abs()));
        span.max_element() <= limit.clamp(1.0, SPAN)
    }

    /// The four it is cut into (0x8004c2e8: the corners' triangles, then
    /// the middle; 0x8004b850: a quad's four corners about its centre).
    fn cut(&self) -> [Piece; 4] {
        let p = self;
        let point = |a: usize, b: usize| (mid(p.at[a], p.at[b]), mid_uv(p.uv[a], p.uv[b]), mid_rgb(p.rgb[a], p.rgb[b]));
        let tri = |c: [(Vec3, [u8; 2], [u8; 3]); 3]| Piece {
            at: [c[0].0, c[1].0, c[2].0, c[2].0],
            uv: [c[0].1, c[1].1, c[2].1, c[2].1],
            rgb: [c[0].2, c[1].2, c[2].2, c[2].2],
            quad: false,
        };
        let corner = |i: usize| (p.at[i], p.uv[i], p.rgb[i]);
        if !p.quad {
            let (m01, m12, m02) = (point(0, 1), point(1, 2), point(0, 2));
            return [
                tri([corner(0), m01, m02]),
                tri([m01, corner(1), m12]),
                tri([m02, m12, corner(2)]),
                tri([m01, m12, m02]),
            ];
        }
        // Quad corners 0 1 / 2 3 (the strip order the models use).
        let (m01, m23, m02, m13) = (point(0, 1), point(2, 3), point(0, 2), point(1, 3));
        let centre = (
            (p.at[0] + p.at[1] + p.at[2] + p.at[3]) / 4.0,
            mid_uv(mid_uv(p.uv[0], p.uv[1]), mid_uv(p.uv[2], p.uv[3])),
            mid_rgb(mid_rgb(p.rgb[0], p.rgb[1]), mid_rgb(p.rgb[2], p.rgb[3])),
        );
        let quad = |c: [(Vec3, [u8; 2], [u8; 3]); 4]| Piece {
            at: c.map(|x| x.0),
            uv: c.map(|x| x.1),
            rgb: c.map(|x| x.2),
            quad: true,
        };
        [
            quad([corner(0), m01, m02, centre]),
            quad([m01, corner(1), centre, m13]),
            quad([m02, centre, corner(2), m23]),
            quad([centre, m13, m23, corner(3)]),
        ]
    }

    /// Its centre and the one-face model about it.
    fn shard(&self, face: &Face) -> (Vec3, Model) {
        let n = self.corners();
        let centre = self.at[..n].iter().copied().sum::<Vec3>() / n as f32;
        let radius = self.at[..n].iter().map(|&a| (a - centre).length()).fold(0.0, f32::max);
        let vertices = (0..4).map(|i| {
            let v = self.at[i.min(n - 1)] - centre;
            [v.x as i16, v.y as i16, v.z as i16, 0]
        });
        let (_, clut, tpage) = face.texture.unwrap_or_default();
        let face = Face {
            v: [0, 1, 2, 3],
            quad: self.quad,
            shading: Shading::Flat,
            rgb: self.rgb,
            texture: Some((self.uv, clut, tpage)),
            flags: face.flags,
        };
        let model = Model {
            radius: radius as u16,
            vertices: vertices.collect(),
            palette: Vec::new(),
            faces: vec![face],
            boxes: Vec::new(),
        };
        (centre, model)
    }
}

/// The pieces `model` breaks into, at most `free`, each with its centre
/// in the model's space. `radius` is the object's (+0x86).
pub fn pieces(model: &Model, radius: f32, free: usize) -> Vec<(Vec3, Model)> {
    let mut out = Vec::new();
    let faces = model.faces.iter().take_while(|f| f.shading == Shading::Flat && f.texture.is_some());
    for face in faces.take(SECTION) {
        let Some((uv, ..)) = face.texture else { continue };
        let corner = |i: usize| {
            let v = model.vertices[face.v[i] as usize];
            Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32)
        };
        let piece = Piece { at: [0, 1, 2, 3].map(corner), uv, rgb: face.rgb, quad: face.quad };
        let mut stack = vec![piece];
        while let Some(p) = stack.pop() {
            if out.len() >= free {
                return out;
            }
            if p.small(radius) {
                out.push(p.shard(face));
            } else {
                // Depth first, in cut order.
                stack.extend(p.cut().into_iter().rev());
            }
        }
    }
    out
}
