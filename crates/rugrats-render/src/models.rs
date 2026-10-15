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
