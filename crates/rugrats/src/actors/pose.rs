//! A hierarchy posed as the original poses it (UpdatePartRotations,
//! SLUS_006.50:0x8004a5dc), in its integers: on the scratchpad a stack of
//! matrices by depth, the object's angles at the bottom (0x80020d10), each
//! part's turn (its kept quaternion's matrix, 0x8001fc48) composed onto
//! its parent's (CompMatrix, 0x8009f598) with its offset; each part's
//! matrix then transposed into +0x158 (32 bytes a part), its joint moved
//! by the object's position.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::hierarchy::Hierarchy;

use crate::gte::Trig;

/// A part as the original keeps it at +0x158: the world into the part
/// (rows, 4096 = 1: its rotation transposed) and its joint in the world.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartMatrix {
    pub rot: [[i32; 3]; 3],
    pub joint: [i32; 3],
}

impl PartMatrix {
    /// The part's rotation (the part into the world) and joint, as floats.
    pub fn float(&self) -> (Mat3, Vec3) {
        let r = |c: usize| Vec3::new(self.rot[c][0] as f32, self.rot[c][1] as f32, self.rot[c][2] as f32) / 4096.0;
        (Mat3::from_cols(r(0), r(1), r(2)), Vec3::new(self.joint[0] as f32, self.joint[1] as f32, self.joint[2] as f32))
    }
}

/// A turn's matrix (0x8001fc48): the quaternion (x, y, z, w) scaled by
/// 2^24 over its squared length, the products >> 12 toward zero, laid out
/// as the transpose of 0x8001f9fc's, then MatrixNormal.
fn turn_matrix(trig: &Trig, q: [i32; 4]) -> [[i32; 3]; 3] {
    let m = trig.quat_matrix(q);
    trig.matrix_normal([0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r])))
}

/// CompMatrix (0x8009f598): `a` times `b`, a column at a time through the
/// GTE (>> 12, each kept to 16 bits), and `a`'s turn of `b`'s offset (its
/// components taken as 16 bits) plus `a`'s.
fn compose(a: &([[i32; 3]; 3], [i32; 3]), b: &([[i32; 3]; 3], [i32; 3])) -> ([[i32; 3]; 3], [i32; 3]) {
    let mac = |v: [i32; 3]| [0, 1, 2].map(|r| ((0..3).map(|k| a.0[r][k] as i64 * v[k] as i64).sum::<i64>() >> 12) as i32);
    let mut rot = [[0; 3]; 3];
    for c in 0..3 {
        let col = mac([b.0[0][c], b.0[1][c], b.0[2][c]]);
        for r in 0..3 {
            rot[r][c] = col[r].clamp(-0x8000, 0x7fff);
        }
    }
    let t = mac(b.1.map(|v| v as i16 as i32));
    (rot, [0, 1, 2].map(|k| t[k].wrapping_add(a.1[k])))
}

/// Each part's matrix (+0x158) for a body at `pos` turned by `angles`,
/// each part turned by `turns` (its kept quaternion, none the disc's
/// zeros: no turn); and the joints' greatest y (+0x68). The root's own
/// offset is left out.
pub fn posed(trig: &Trig, h: &Hierarchy, turns: &[Option<[i32; 4]>], angles: [i16; 3], pos: [i32; 3]) -> (Vec<PartMatrix>, i32) {
    let base = (trig.rotation(angles), [0; 3]);
    let mut world: Vec<([[i32; 3]; 3], [i32; 3])> = Vec::with_capacity(h.parts.len());
    let mut out = Vec::with_capacity(h.parts.len());
    let mut low = 0;
    for (i, part) in h.parts.iter().enumerate() {
        let q = turns.get(i).copied().flatten().unwrap_or([0; 4]);
        let offset = if i == 0 { [0; 3] } else { part.offset };
        let parent = part.parent.map_or(&base, |p| &world[p]);
        let me = compose(parent, &(turn_matrix(trig, q), offset));
        let joint = [0, 1, 2].map(|k| me.1[k].wrapping_add(pos[k]));
        if i == 0 || low < joint[1] {
            low = joint[1];
        }
        out.push(PartMatrix { rot: [0, 1, 2].map(|r| [0, 1, 2].map(|c| me.0[c][r])), joint });
        world.push(me);
    }
    (out, low)
}
