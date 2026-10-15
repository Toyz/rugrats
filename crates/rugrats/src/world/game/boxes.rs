//! Collision boxes of the placed objects, and the hazards among them.

use super::*;

/// The world boxes of the placed models (the player's placement left out):
/// each model box with the placement's rotation and position, and the
/// heights its corners span.
/// The boxes of the objects that hurt on touch, with their +0x6d.
pub(super) fn hazards(set: &Set, apart: &[bool]) -> Vec<(crate::actors::player::Box3, i32)> {
    let mut out = Vec::new();
    for (k, place) in set.placements.iter().enumerate() {
        let Some(t) = set.templates.get(place.template as usize) else { continue };
        let damage = t.raw[0x32] as i32;
        if place.kind != 1 || apart[k] || damage == 0 || t.hierarchy().is_some() {
            continue;
        }
        let Some(m) = set.models.get(t.shape as u32).filter(|_| set.models.has(t.shape as u32)) else { continue };
        let rot = rugrats_render::models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        out.extend(world_boxes(m, rot, pos).into_iter().map(|b| (b, damage)));
    }
    out
}

pub(super) fn object_boxes(set: &Set, skip: impl Fn(usize) -> bool) -> Vec<crate::actors::player::Box3> {
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
        let rot = rugrats_render::models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        let climb = t.flags & 64 != 0;
        out.extend(world_boxes(m, rot, pos).into_iter().map(|mut b| {
            b.climb = climb;
            b
        }));
    }
    out
}

/// A model's boxes at `rot`, `pos`, with the heights their corners span.
pub(super) fn world_boxes(m: &rugrats_data::model::Model, rot: Mat3, pos: Vec3) -> Vec<crate::actors::player::Box3> {
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
        out.push(crate::actors::player::Box3 {
            lo: lo.map(|v| v as i32),
            hi: hi.map(|v| v as i32),
            rot,
            pos,
            min_y: a as i32,
            max_y: b as i32,
            climb: false,
            character: false,
        });
    }
    out
}

impl Game {
    /// How far `y` is from a placed object's model in height (0 inside
    /// it): a door hung high is used from the floor below.
    pub(super) fn use_dy(&self, d: &rugrats_data::placement::Placement, y: i32) -> i32 {
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
