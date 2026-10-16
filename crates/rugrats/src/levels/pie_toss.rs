//! The pie toss (0x8007bc60, 0x80061c98's rolling throw): pies thrown
//! from a counter roll along the floor at their targets. Level 3's bakery
//! and training's booth share it.

use rrt::glam::{Mat3, Vec3};
use rugrats_render::{Vtx, models};

use crate::actors::player::DT;
use crate::world::collide;
use crate::world::game::Game;

/// A splat's axis where no surface is known: straight up.
const UP: [i32; 3] = [0, -4096, 0];

/// A pie (0x80061c98): rolled along (0, 35, 1024), speed 225 with the
/// target within 13000, else 450; model 885.
const PIE_NEAR: i32 = 13000;
const PIE_MODEL: u32 = 885;
/// A hit: the character's half size (at least 300) and the pie's (guess).
const PIE_REACH: i32 = 300;
const PIE_RADIUS: i32 = 300;
const PIE_RANGE: i32 = 30000;

/// The face a pie came at on a character's bounds (axis-aligned about
/// it, as world::game::things takes a thrown thing's), as a normal.
fn face_normal(from: [i32; 3], at: [i32; 3]) -> [i32; 3] {
    let (dx, dz) = (at[0] - from[0], at[2] - from[2]);
    if dx.abs() > dz.abs() { [dx.signum() * 4096, 0, 0] } else { [0, 0, dz.signum() * 4096] }
}

/// A pie rolling.
pub struct Pie {
    pos: [i32; 3],
    v: [i32; 2],
    travelled: i32,
}

impl Pie {
    /// A pie thrown from `from` along `heading` (0x80061c98): speed 225
    /// with `aim` within 13000, else 450; from the pie spot 512 ahead.
    pub fn thrown(g: &Game, from: [i32; 3], heading: i32, aim: Option<[i32; 3]>) -> Pie {
        let near = aim.is_some_and(|p| {
            let d = [(p[0] - from[0]) as f64, (p[1] - from[1]) as f64, (p[2] - from[2]) as f64];
            ((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() as i32) < PIE_NEAR
        });
        let speed = if near { 225 } else { 450 } * 65536 / 6000;
        let (s, c) = (g.trig.sin(heading), g.trig.cos(heading));
        let pos = [from[0] + ((s * 512) >> 12), from[1], from[2] + ((c * 512) >> 12)];
        Pie { pos, v: [(s * speed) >> 12, (c * speed) >> 12], travelled: 0 }
    }
}

/// Pies roll on the floor below the counter (gravity 1, flags
/// 0x0a070400) until a wall, a hit on one of `targets` (NPC indices) or
/// the range; each ends in a splat about the surface it met (a wall's
/// normal from the segment test; the face of the character's bounds it
/// came at; straight up at the range). Returns the characters hit.
pub fn roll_pies(g: &mut Game, pies: &mut Vec<Pie>, targets: &[usize]) -> Vec<usize> {
    let mut hits = Vec::new();
    pies.retain_mut(|p| {
        // In quarter steps, so a fast pie cannot pass through.
        for _ in 0..4 {
            let before = [p.pos[0], p.pos[1] - 1, p.pos[2]];
            for k in 0..2 {
                p.pos[2 * k] += (p.v[k] * DT) >> 8;
            }
            p.travelled += ((p.v[0] as f64).hypot(p.v[1] as f64) as i32 * DT) >> 8;
            let q = collide::query(&g.col, [p.pos[0], p.pos[1] - 1, p.pos[2]]);
            if q.kind == 2 || q.tile.is_none() {
                let mut hit = collide::Segment::default();
                let to = [p.pos[0], p.pos[1] - 1, p.pos[2]];
                let normal = if collide::segment(&g.col, before, to, &mut hit) != 0 { hit.normal } else { UP };
                g.flights.splat(normal, p.pos, g.sprites.as_ref());
                return false;
            }
            if p.travelled > PIE_RANGE {
                g.flights.splat(UP, p.pos, g.sprites.as_ref());
                return false;
            }
            if q.kind == 0 {
                p.pos[1] = (p.pos[1] + 30).min(q.floor);
            }
            let hit = targets.iter().copied().find(|&k| {
                let n = &g.npcs[k];
                let (dx, dz) = ((n.pos[0] - p.pos[0]) as i64, (n.pos[2] - p.pos[2]) as i64);
                let reach = (n.size.max(PIE_REACH) + PIE_RADIUS) as i64;
                dx * dx + dz * dz < reach * reach
            });
            if let Some(k) = hit {
                hits.push(k);
                let normal = face_normal(g.npcs[k].pos, p.pos);
                g.flights.splat(normal, p.pos, g.sprites.as_ref());
                return false;
            }
        }
        true
    });
    hits
}

/// The pies rolling and the one in hand (model 885); the splats are drawn
/// with the game's flights.
pub fn draw_pies(g: &Game, pies: &[Pie], holding: bool, moving: &mut Vec<Vtx>) {
    let Some(m) = g.set.models.get(PIE_MODEL) else { return };
    for p in pies {
        let pos = Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32);
        models::model_triangles(m, Mat3::IDENTITY, pos, moving);
    }
    if holding {
        let parts = g.player.parts(&g.anims, &g.trig);
        let rot = g.player.rotation(&g.trig);
        if let Some(hand) = g.player.slot_part(15).and_then(|i| parts.get(i)) {
            models::model_triangles(m, rot, hand.1 + rot * Vec3::new(0.0, -100.0, 200.0), moving);
        }
    }
}
