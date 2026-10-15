//! Models as triangles, placed by a rotation (4.12 fixed point, as the
//! game keeps it) and a position.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::model::Model;

use crate::mesh::Vtx;

fn rgb(c: [u8; 3]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16
}

/// A face flag: drawn from both sides (the renderer skips NCLIP).
pub const FACE_TWO_SIDED: u32 = 1;
/// A face flag: semi-transparent.
pub const FACE_SEMI: u32 = 1 << 7;

/// The model's faces, each vertex turned by `rot` and moved to `pos`.
/// Untextured faces are their colour as it is; textured ones modulate the
/// texel by it. Winding as the rooms': front where NCLIP is positive.
///
/// The model renderer (hand-written, SLUS_006.50:0x80010000) makes a
/// face's packet semi-transparent with its flags' bit 7 ((flags & 0x80)
/// << 18 into the code); an untextured one also gets a draw mode whose
/// blend is the flags' bits 5-6 (0x800106c0), a textured one its page's.
/// A face with bit 0 skips the back-face test (0x800102b4), so it goes in
/// both windings.
pub fn model_triangles(m: &Model, rot: Mat3, pos: Vec3, out: &mut Vec<Vtx>) {
    for f in &m.faces {
        let semi = if f.flags & FACE_SEMI != 0 { Vtx::SEMI } else { 0 };
        let corner = |i: usize| {
            let v = m.vertices[f.v[i] as usize];
            let p = rot * Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32) + pos;
            let (uv, mode) = match f.texture {
                Some((uv, clut, tpage)) => (Vtx::uv_of(uv[i][0] as u32, uv[i][1] as u32), clut as u32 | (tpage as u32) << 16 | semi),
                None => (0, Vtx::FLAT | semi | (f.flags >> 5 & 3) << 21),
            };
            Vtx { pos: p.to_array(), colour: rgb(f.rgb[i]), uv, mode, window: Vtx::WHOLE }
        };
        out.extend([corner(0), corner(2), corner(1)]);
        if f.quad {
            out.extend([corner(1), corner(2), corner(3)]);
        }
        if f.flags & FACE_TWO_SIDED != 0 {
            out.extend([corner(0), corner(1), corner(2)]);
            if f.quad {
                out.extend([corner(1), corner(3), corner(2)]);
            }
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
/// composed from the object's rotation `base` down as UpdatePartRotations
/// (0x8004a5dc) does, the whole at `pos`.
pub fn posed_triangles(
    h: &rugrats_data::hierarchy::Hierarchy,
    models: &rugrats_data::model::Models,
    local: &[Option<Mat3>],
    base: Mat3,
    pos: Vec3,
    out: &mut Vec<Vtx>,
) {
    for (part, me) in h.parts.iter().zip(posed_parts(h, local, base, pos)) {
        if let Some(m) = models.get(part.model) {
            model_triangles(m, me.0, me.1, out);
        }
    }
}

/// Each part's rotation and joint in the world, as UpdatePartRotations
/// (0x8004a5dc) composes them.
pub fn posed_parts(h: &rugrats_data::hierarchy::Hierarchy, local: &[Option<Mat3>], base: Mat3, pos: Vec3) -> Vec<(Mat3, Vec3)> {
    let mut world: Vec<(Mat3, Vec3)> = Vec::with_capacity(h.parts.len());
    for (i, part) in h.parts.iter().enumerate() {
        let rot = local.get(i).copied().flatten().unwrap_or(Mat3::IDENTITY);
        // The root sits at the object's position: the game
        // (0x8004a5dc) leaves its offset out.
        let offset = match part.parent {
            Some(_) => Vec3::new(part.offset[0] as f32, part.offset[1] as f32, part.offset[2] as f32),
            None => Vec3::ZERO,
        };
        let (prot, ppos) = part.parent.map_or((base, pos), |p| world[p]);
        world.push((prot * rot, ppos + prot * offset));
    }
    world
}
