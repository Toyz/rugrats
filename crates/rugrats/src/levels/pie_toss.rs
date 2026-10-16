//! The pie toss (0x8007bc60, 0x80061c98's rolling throw): pies thrown
//! from a counter roll along the floor at their targets. Level 3's bakery
//! and training's booth share it.

use rrt::glam::{Mat3, Vec3};
use rugrats_render::{Vtx, models};

use crate::actors::player::DT;
use crate::ui::hud::Look;
use crate::world::collide;
use crate::world::game::Game;

/// A pie (0x80061c98): rolled along (0, 35, 1024), speed 225 with the
/// target within 13000, else 450; model 885.
const PIE_NEAR: i32 = 13000;
const PIE_MODEL: u32 = 885;
/// A hit: the character's half size (at least 300) and the pie's (guess).
const PIE_REACH: i32 = 300;
const PIE_RADIUS: i32 = 300;
const PIE_RANGE: i32 = 30000;

/// A pie's splat (0x8005282c, the pie template's flight end: +0x0b = 1):
/// five flights of sprite 359 (TEXPOS + 0x10d4) thrown from where the pie
/// stopped, flags 0x30705: gravity 13, the world collides, drawn as a
/// camera-facing quad (kind 5, 0x8008afc4) of half-size 200 x 200 with
/// face flags 0xd0800000. Each lives 2000 dt and bounces by 1000 / 4096.
const SPLAT_SPRITE: u32 = 359;
const SPLAT_SIZE: f32 = 200.0;
const SPLAT_LIFE: i32 = 2000;
const SPLAT_BOUNCE: i32 = 1000;
const SPLAT_GRAVITY: i32 = 13;
const SPLAT_BITS: usize = 5;

/// The two LCGs 0x8005282c draws from (x 214013 + 0x269ec3): the speeds'
/// (0x800c41a4) and the directions' (0x800c445c, 0x80085f44), seeded as
/// the executable has them.
fn lcg(x: &mut u32) -> u32 {
    *x = x.wrapping_mul(214013).wrapping_add(0x269ec3);
    *x
}

struct Bit {
    pos: [i32; 3],
    v: [i32; 3],
    left: i32,
}

/// The splats flying.
pub struct Splats {
    bits: Vec<Bit>,
    seeds: (u32, u32),
}

impl Default for Splats {
    fn default() -> Splats {
        Splats { bits: Vec::new(), seeds: (48732, 1092843) }
    }
}

impl Splats {
    /// A pie's five bits at `at`: speed min(rand mod 150, 50); direction
    /// tilted rand mod 2048 from straight up and spun rand mod 4096
    /// (0x80088bac(.., 0, 2048)).
    pub fn spawn(&mut self, g: &Game, at: [i32; 3]) {
        for _ in 0..SPLAT_BITS {
            let speed = ((lcg(&mut self.seeds.0) >> 16) & 0x7fff) as i32 % 150;
            let speed = speed.min(50) * 65536 / 6000;
            let tilt = ((lcg(&mut self.seeds.1) >> 16) & 4095) as i32 % 2048;
            let spin = ((lcg(&mut self.seeds.1) >> 16) & 4095) as i32;
            let (st, ct) = (g.trig.sin(tilt), g.trig.cos(tilt));
            let (ss, cs) = (g.trig.sin(spin), g.trig.cos(spin));
            let dir = [(st * cs) >> 12, -ct, (st * ss) >> 12];
            let v = dir.map(|d| (d * speed) >> 12);
            self.bits.push(Bit { pos: at, v, left: SPLAT_LIFE });
        }
    }

    /// One frame (0x80087c5c): gravity, the floor, the lifetime.
    pub fn step(&mut self, g: &Game) {
        self.bits.retain_mut(|b| {
            for k in [0, 2] {
                b.pos[k] += (b.v[k] * DT) >> 6;
            }
            b.pos[1] += (b.v[1] * DT + SPLAT_GRAVITY * DT * DT / 2) >> 6;
            b.v[1] += SPLAT_GRAVITY * DT;
            let q = collide::query(&g.col, [b.pos[0], b.pos[1] - SPLAT_SIZE as i32, b.pos[2]]);
            if q.tile.is_some() && b.pos[1] > q.floor && b.v[1] > 0 {
                b.pos[1] = q.floor;
                b.v = [b.v[0], -b.v[1], b.v[2]].map(|v| (v * SPLAT_BOUNCE) >> 12);
            }
            b.left -= DT;
            b.left > 0
        });
    }

    pub fn draw(&self, g: &Game, moving: &mut Vec<Vtx>) {
        let Some(s) = &g.sprites else { return };
        let (right, down) = (g.view.1.row(0) * SPLAT_SIZE, g.view.1.row(1) * SPLAT_SIZE);
        for b in &self.bits {
            let at = Vec3::new(b.pos[0] as f32, (b.pos[1] - SPLAT_SIZE as i32) as f32, b.pos[2] as f32);
            s.quad(SPLAT_SPRITE, at, (right, down), Look::default(), moving);
        }
    }
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
/// the range; each ends in a splat. Returns the characters hit.
pub fn roll_pies(g: &Game, pies: &mut Vec<Pie>, splats: &mut Splats, targets: &[usize]) -> Vec<usize> {
    splats.step(g);
    let mut hits = Vec::new();
    pies.retain_mut(|p| {
        // In quarter steps, so a fast pie cannot pass through.
        for _ in 0..4 {
            for k in 0..2 {
                p.pos[2 * k] += (p.v[k] * DT) >> 8;
            }
            p.travelled += ((p.v[0] as f64).hypot(p.v[1] as f64) as i32 * DT) >> 8;
            let q = collide::query(&g.col, [p.pos[0], p.pos[1] - 1, p.pos[2]]);
            if q.kind == 2 || q.tile.is_none() || p.travelled > PIE_RANGE {
                splats.spawn(g, p.pos);
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
                splats.spawn(g, p.pos);
                return false;
            }
        }
        true
    });
    hits
}

/// The pies flying, their splats and the one in hand (model 885).
pub fn draw_pies(g: &Game, pies: &[Pie], splats: &Splats, holding: bool, moving: &mut Vec<Vtx>) {
    splats.draw(g, moving);
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
