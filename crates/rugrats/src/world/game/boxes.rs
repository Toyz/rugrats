//! Collision boxes of the placed objects, and the hazards among them.

use rugrats_data::model::Model;
use rugrats_data::placement::Placement;
use rugrats_render::models;

use super::*;
use crate::actors::player::{Box3, Obj, Outline};
use crate::world::boxhit::{Exact, Shape, Solid};

/// The world boxes of the placed models (the player's placement left out):
/// each model box with the placement's rotation and position, and the
/// heights its corners span.
/// An object's box that hurts on touch by its +0x6d.
#[derive(Clone, Copy, Debug)]
pub(super) struct Hazard {
    pub area: Box3,
    pub damage: i32,
}

/// The boxes of the objects that hurt on touch.
pub(super) fn hazards(set: &Set, apart: &[bool]) -> Vec<Hazard> {
    let mut out = Vec::new();
    for (k, place) in set.placements.iter().enumerate() {
        let Some(t) = set.templates.get(place.template as usize) else { continue };
        let damage = t.raw[0x32] as i32;
        if place.kind != 1 || apart[k] || damage == 0 || t.hierarchy().is_some() {
            continue;
        }
        let Some(m) = set.models.get(t.shape as u32).filter(|_| set.models.has(t.shape as u32)) else { continue };
        let rot = models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        out.extend(world_boxes(m, rot, pos).into_iter().map(|area| Hazard { area, damage }));
    }
    out
}

pub(super) fn object_boxes(set: &Set, trig: &Trig, skip: impl Fn(usize) -> bool) -> Vec<Box3> {
    let mut out = Vec::new();
    for (k, place) in set.placements.iter().enumerate() {
        if place.kind != 1 || skip(k) {
            continue;
        }
        let Some(t) = set.templates.get(place.template as usize) else { continue };
        // Template flag 8: skipped by the object collision tests
        // (0x80063d3c, 0x80065210 ..).
        if t.hierarchy().is_some() || set.models.find(t.shape as u32) == 0 || t.flags & 8 != 0 {
            continue;
        }
        let Some(m) = set.models.get(t.shape as u32) else { continue };
        let rot = models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        let climb = t.flags & 64 != 0;
        let angles = trig.placed_angles(place.rotation);
        let size = set.sizes.get(k).copied().filter(|&s| s > 0).unwrap_or_else(|| crate::world::boxhit::bound_size(&m.boxes));
        out.extend(exact(world_boxes(m, rot, pos), place.pos, angles, t.shape as u32, size).into_iter().map(|mut b| {
            b.climb = climb;
            b.template = place.template as u16;
            b
        }));
    }
    out
}

/// `boxes` (a model's, in order) marked as the object at `pos` turned by
/// `angles` with model `shape` and size `size` (+0x86).
pub(super) fn exact(boxes: Vec<Box3>, pos: [i32; 3], angles: [i16; 3], shape: u32, size: i32) -> Vec<Box3> {
    boxes
        .into_iter()
        .enumerate()
        .map(|(j, mut b)| {
            b.obj = Some(Obj { pos, angles, shape, index: j as u8, size });
            b
        })
        .collect()
}

/// The angles (+0x24) of an object turned by `rot` (its model to the
/// world): its world-to-model matrix's (0x8002114c), that matrix the
/// one 0x80020ee0 makes of them.
pub(super) fn angles_of(trig: &Trig, rot: Mat3) -> [i16; 3] {
    trig.matrix_angles([0, 1, 2].map(|r| [0, 1, 2].map(|c| (rot.col(r)[c] * 4096.0).round() as i32)))
}

/// A position as the original's integers.
pub(super) fn ipos(v: Vec3) -> [i32; 3] {
    [v.x.round() as i32, v.y.round() as i32, v.z.round() as i32]
}

/// This frame's solids, one an object, from the boxes marked with theirs
/// (each object's boxes together, in order), each box linked to its solid.
pub(super) fn solids(boxes: &mut [Box3], set: &Set) -> Vec<Exact> {
    let mut out: Vec<Exact> = Vec::new();
    let mut at: Option<Obj> = None;
    for b in boxes.iter_mut() {
        let Some(o) = b.obj else { continue };
        let same = at.is_some_and(|a| a.pos == o.pos && a.angles == o.angles && a.shape == o.shape && o.index > a.index);
        if !same {
            let Some(m) = set.models.get(o.shape) else { continue };
            let r = o.size;
            let shape = Shape::Model(m.boxes.iter().map(|(lo, hi)| (lo.map(|v| v as i32), hi.map(|v| v as i32))).collect());
            out.push(Exact { solid: Solid { pos: o.pos, angles: o.angles, bound: r, size: r, shape }, climb: b.climb, template: b.template });
        }
        at = Some(o);
        b.solid = Some(((out.len() - 1) as u16, o.index));
    }
    out
}

/// The object in focus (player +0x1e8, 0x8006af4c), as the original finds
/// it: each placed object of template flag 8 whose ball (its model's
/// radius, +0x86, about its bound's middle) the body's cylinder overlaps
/// (0x8006abbc: the body box's widest across, `radius`, and tallest,
/// `half`), and whose model the segment from the body's part `part_at`
/// (the hierarchy's header +0x24) down `half` meets (0x8006a714). Its
/// placement.
pub(super) fn focus_at(set: &Set, trig: &Trig, pos: [i32; 3], radius: i32, half: i32, part_at: [i32; 3]) -> Option<usize> {
    set.placements.iter().enumerate().filter(|(_, p)| p.kind == 1).find_map(|(k, place)| {
        let t = set.templates.get(place.template as usize)?;
        if t.flags & 8 == 0 || t.hierarchy().is_some() {
            return None;
        }
        let m = set.models.get(t.shape as u32)?;
        let size = set.sizes.get(k).copied().filter(|&s| s > 0).unwrap_or_else(|| crate::world::boxhit::bound_size(&m.boxes));
        let c = place.pos;
        let (dx, dz) = (c[0].wrapping_sub(pos[0]), c[2].wrapping_sub(pos[2]));
        let reach = (size + radius).wrapping_mul(size + radius) as u32;
        if reach < (dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz))) as u32 {
            return None;
        }
        if c[1] + size < pos[1] - half || pos[1] + half < c[1] - size {
            return None;
        }
        let shape = Shape::Model(m.boxes.iter().map(|(lo, hi)| (lo.map(|v| v as i32), hi.map(|v| v as i32))).collect());
        let solid = Solid { pos: c, angles: trig.placed_angles(place.rotation), bound: size, size, shape };
        let to = [part_at[0], part_at[1] + half, part_at[2]];
        solid.meet(trig, part_at, to, &mut [0; 3]).map(|_| k)
    })
}

/// A model's boxes at `rot`, `pos`, with the heights their corners span.
pub(super) fn world_boxes(m: &Model, rot: Mat3, pos: Vec3) -> Vec<Box3> {
    let mut out = Vec::new();
    for &(lo, hi) in &m.boxes {
        let (mut a, mut b) = (f32::MAX, f32::MIN);
        for c in 0..8 {
            let v = Vec3::new(
                if c & 1 == 0 { lo[0] } else { hi[0] } as f32,
                if c & 2 == 0 { lo[1] } else { hi[1] } as f32,
                if c & 4 == 0 { lo[2] } else { hi[2] } as f32,
            );
            let y = (rot * v + pos).y;
            a = a.min(y);
            b = b.max(y);
        }
        let (lo, hi) = (lo.map(|v| v as i32), hi.map(|v| v as i32));
        out.push(Box3 {
            lo,
            hi,
            rot,
            pos,
            min_y: a as i32,
            max_y: b as i32,
            climb: false,
            character: false,
            template: 0,
            outline: Outline::of(lo, hi, rot, pos),
            obj: None,
            solid: None,
        });
    }
    out
}

impl Game {
    /// How far `y` is from a placed object's model in height (0 inside
    /// it): a door hung high is used from the floor below.
    pub(super) fn use_dy(&self, d: &Placement, y: i32) -> i32 {
        let span = self
            .set
            .templates
            .get(d.template as usize)
            .and_then(|t| self.set.models.get(t.shape as u32).filter(|_| self.set.models.has(t.shape as u32)))
            .and_then(|m| Some((m.boxes.iter().map(|b| b.0[1]).min()?, m.boxes.iter().map(|b| b.1[1]).max()?)));
        let (lo, hi) = span.map_or((d.pos[1], d.pos[1]), |(a, b)| (d.pos[1] + a as i32, d.pos[1] + b as i32));
        if y < lo {
            lo - y
        } else if y > hi {
            y - hi
        } else {
            0
        }
    }
}
