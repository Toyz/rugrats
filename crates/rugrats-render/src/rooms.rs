//! A database's rooms as triangles: each geometry blob's faces, textured
//! from the level's table, coloured by their vertices.

use rugrats_data::level::{Level, Mesh, Texture};

use crate::mesh::Vtx;

fn rgb(c: [u8; 3]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16
}

fn uv(t: [u8; 2]) -> u32 {
    t[0] as u32 | (t[1] as u32) << 8
}

/// One mesh's faces as triangles. A quad's corners go to the GPU in its
/// order 0 1 2 3 (triangles 0 1 2 and 1 2 3), front where the game's NCLIP
/// is positive; the renderer culls clockwise, so they are emitted 0 2 1 and
/// 1 2 3. A face with flag 0x200 is drawn from both sides.
pub fn mesh_triangles(m: &Mesh, at: [i32; 3], textures: &[Texture], out: &mut Vec<Vtx>) {
    for f in &m.faces {
        let Some(t) = textures.get(f.texture as usize) else { continue };
        let corner = |i: usize| {
            let v = m.vertices[f.v[i] as usize];
            Vtx {
                pos: [(at[0] + v.pos[0] as i32) as f32, (at[1] + v.pos[1] as i32) as f32, (at[2] + v.pos[2] as i32) as f32],
                colour: rgb(v.rgb),
                uv: uv(t.uv[i]),
                mode: t.clut as u32 | (t.tpage as u32) << 16,
                window: Vtx::WHOLE,
            }
        };
        let mut tri = vec![corner(0), corner(2), corner(1)];
        if f.quad {
            tri.extend([corner(1), corner(2), corner(3)]);
        }
        if f.flags & 0x200 != 0 {
            let back: Vec<Vtx> = tri.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).collect();
            tri.extend(back);
        }
        out.extend(tri);
    }
}

/// Every room of the level, each at its centre.
pub fn level_triangles(level: &Level) -> Vec<Vtx> {
    let textures = level.textures();
    let mut out = Vec::new();
    for room in &level.rooms {
        for blob in &room.blobs {
            if let Ok(m) = Mesh::parse(blob) {
                mesh_triangles(&m, room.centre(), &textures, &mut out);
            }
        }
    }
    out
}
