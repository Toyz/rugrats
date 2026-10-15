//! Models as triangles, placed by a rotation (4.12 fixed point, as the
//! game keeps it) and a position.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::model::Model;

use crate::mesh::Vtx;

fn rgb(c: [u8; 3]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16
}

/// The model's faces, each vertex turned by `rot` and moved to `pos`.
/// Untextured faces are their colour as it is; textured ones modulate the
/// texel by it. Winding as the rooms': front where NCLIP is positive.
pub fn model_triangles(m: &Model, rot: Mat3, pos: Vec3, out: &mut Vec<Vtx>) {
    for f in &m.faces {
        let corner = |i: usize| {
            let v = m.vertices[f.v[i] as usize];
            let p = rot * Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32) + pos;
            let (uv, mode) = match f.texture {
                Some((uv, clut, tpage)) => (uv[i][0] as u32 | (uv[i][1] as u32) << 8, clut as u32 | (tpage as u32) << 16),
                None => (0, Vtx::FLAT),
            };
            Vtx { pos: p.to_array(), colour: rgb(f.rgb[i]), uv, mode, window: Vtx::WHOLE }
        };
        out.extend([corner(0), corner(2), corner(1)]);
        if f.quad {
            out.extend([corner(1), corner(2), corner(3)]);
        }
    }
}

/// A quaternion (x, y, z, w), at any scale, as the rotation it stands for
/// (SLUS_006.50:0x8001fc48 normalises by its squared length).
pub fn quat_matrix(q: [i16; 4]) -> Mat3 {
    let [x, y, z, w] = q.map(|v| v as f32);
    let n = x * x + y * y + z * z + w * w;
    if n <= 0.0 {
        return Mat3::IDENTITY;
    }
    let s = 2.0 / n;
    let (xx, yy, zz) = (x * x * s, y * y * s, z * z * s);
    let (xy, xz, yz) = (x * y * s, x * z * s, y * z * s);
    let (wx, wy, wz) = (w * x * s, w * y * s, w * z * s);
    // The game's matrix is this, row-major (the test against 0x8001fc48
    // pins the signs).
    Mat3::from_cols(
        Vec3::new(1.0 - yy - zz, xy - wz, xz + wy),
        Vec3::new(xy + wz, 1.0 - xx - zz, yz - wx),
        Vec3::new(xz - wy, yz + wx, 1.0 - xx - yy),
    )
}

/// A hierarchy posed: each part's rotation `local[i]` (None at rest),
/// composed from the root down as UpdatePartRotations (0x8004a5dc) does,
/// the whole at `pos`.
pub fn posed_triangles(
    h: &rugrats_data::hierarchy::Hierarchy,
    models: &[Option<Model>],
    local: &[Option<Mat3>],
    pos: Vec3,
    out: &mut Vec<Vtx>,
) {
    let mut world: Vec<(Mat3, Vec3)> = Vec::with_capacity(h.parts.len());
    for (i, part) in h.parts.iter().enumerate() {
        let rot = local.get(i).copied().flatten().unwrap_or(Mat3::IDENTITY);
        // The root sits at the object's position: the game
        // (0x8004a5dc) leaves its offset out.
        let offset = match part.parent {
            Some(_) => Vec3::new(part.offset[0] as f32, part.offset[1] as f32, part.offset[2] as f32),
            None => Vec3::ZERO,
        };
        let (prot, ppos) = part.parent.map_or((Mat3::IDENTITY, pos), |p| world[p]);
        let me = (prot * rot, ppos + prot * offset);
        world.push(me);
        if let Some(Some(m)) = models.get(part.model as usize) {
            model_triangles(m, me.0, me.1, out);
        }
    }
}
