//! The oracle's side of the collision checks: the player's nearby list
//! rebuilt and read back as the port's solids. (Shared by the checks; not
//! every one uses all of it.)
#![allow(dead_code)]
use rugrats::world::boxhit::{Part, Shape, Solid};
use rugrats_hle::play::{PLAYER, Play};

/// The game's gp.
pub const GP: u32 = 0x800c_3e80;
/// The type table (56 bytes a type, flags first).
const TYPES: u32 = 0x800c_4300;

/// Calls `func(args)` in the original with the game's gp.
pub fn call(p: &mut Play, func: u32, args: &[u32]) -> Result<u32, String> {
    let gp = std::mem::replace(&mut p.hle.m.cpu.r[28], GP);
    let r = p.hle.call_nested(func, args);
    p.hle.m.cpu.r[28] = gp;
    r.map_err(|e| format!("{e:?}"))
}

/// The player's nearby list rebuilt where it stands (0x80062ed4).
pub fn rebuild(p: &mut Play) -> Result<(), String> {
    call(p, 0x8006_2ed4, &[PLAYER]).map(|_| ())
}

/// The original's nearby list (+0x100, rebuilt by [`rebuild`]) as solids:
/// each its address, solid and whether it is passed by (type flag 8).
pub fn nearby(p: &mut Play) -> Vec<(u32, Solid, bool)> {
    let types = p.read_u32(TYPES);
    let mut out = Vec::new();
    let mut node = p.read_u32(PLAYER + 0x100);
    while node != 0 {
        let o = p.read_u32(node + 8);
        let kind = p.read_i16(o + 0x96) as u16 as u32;
        let flags = p.read_u32(types + 56 * kind);
        let v3 = |p: &mut Play, a: u32| [0, 1, 2].map(|k| p.read_i16(a + 2 * k) as i32);
        let pos = [0, 4, 8].map(|k| p.read_i32(o + 0x18 + k));
        let angles = [0, 1, 2].map(|k| p.read_i16(o + 0x24 + 2 * k));
        let bound = Solid::bound_of(v3(p, o + 0x78), v3(p, o + 0x80));
        let size = p.read_i16(o + 0x86) as i32;
        let model = p.read_u32(o + 0x88);
        let shape = if model != 0 {
            let (count, at) = (p.read_i16(model + 10) as u16 as u32, p.read_u32(model + 12));
            Shape::Model((0..count).map(|j| (v3(p, at + 16 * j), v3(p, at + 16 * j + 8))).collect())
        } else if flags & 0x2000 != 0 {
            // The parts as posed (+0x158), or built where the original
            // builds them when none are kept (0x8004a5dc into 0x800dfcc0).
            let h = p.read_u32(o + 0x154);
            let mut mats = p.read_u32(o + 0x158);
            let mut none = false;
            if mats == 0 {
                mats = 0x800d_fcc0;
                none = call(p, 0x8004_a5dc, &[o, mats]).map_or(true, |v| v != 0);
            }
            let count = if none { 0 } else { p.read_i16(h).max(0) as u32 };
            Shape::Parts(
                (0..count)
                    .map(|i| {
                        let m = mats + 32 * i;
                        let rot = [0, 1, 2].map(|r| [0, 1, 2].map(|c| p.read_i16(m + 6 * r + 2 * c) as i32));
                        let trans = [0, 4, 8].map(|k| p.read_i32(m + 0x14 + k));
                        let model = p.read_u32(h + 40 * i + 0x58);
                        let boxes = if model == 0 {
                            Vec::new()
                        } else {
                            let (n, at) = (p.read_i16(model + 10) as u16 as u32, p.read_u32(model + 12));
                            (0..n).map(|j| (v3(p, at + 16 * j), v3(p, at + 16 * j + 8))).collect()
                        };
                        Part { rot, trans, boxes }
                    })
                    .collect(),
            )
        } else {
            let h = p.read_u32(o + 0x154);
            Shape::Body(v3(p, h + 4), v3(p, h + 12))
        };
        out.push((o, Solid { pos, angles, bound, size, shape }, flags & 8 != 0));
        node = p.read_u32(node + 4);
    }
    out
}

