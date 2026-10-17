//! Holds the ground probes to the original's with the player standing on a
//! thing (+0xf8, +0xfd), and the run's probe of its own step
//! (0x800645d8, mode 2 of 0x80064a34).
//!
//! `below_check [LEVEL..] [--n N]`
//!
//! For each level (default 6 and 18) and N spots: a thing near one of the
//! port's objects is picked from the original's nearby list (a model turned
//! about y only), the player put on its box's top within its middle, the
//! list rebuilt, +0xf8 and +0xfd set to it, and 0x80064a34 called in mode
//! 1 or 2 (alternately) with a heading and a speed. The return, the record
//! (+0x144, +0x114, +0x124, +0x130, +0xf4, +0xfc) and, in mode 2, the
//! position's height, the floor (+0x148), the thing stood on, the eased
//! step (+0x188 flag 0x80, +0x14c) and a fall (state 4) are compared.
use std::path::Path;
use std::rc::Rc;

use rugrats::actors::ground::{Ahead, Body, Ground, Near, Probed, Probes};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::boxhit::Shape;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, PLAYER, Play};

#[path = "common/oracle.rs"]
mod oracle;

const PROBE: u32 = 0x8006_4a34;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u32) as i32
    }
}

fn put(p: &mut Play, pos: [i32; 3], floor: i32) {
    for (k, v) in pos.iter().enumerate() {
        p.write_u32(PLAYER + 0x18 + 4 * k as u32, *v as u32);
    }
    p.write_u32(PLAYER + 0x148, floor as u32);
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut levels = Vec::new();
    let mut n = 300;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--n" => n = args.next().and_then(|s| s.parse().ok()).unwrap_or(n),
            l => levels.push(l.parse::<u8>().map_err(|e| e.to_string())?),
        }
    }
    if levels.is_empty() {
        levels = vec![6, 18];
    }
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let trig = Trig::load(&disc)?;
    let mut bad_total = 0;
    for level in levels {
        let mut game = Game::open_from(&disc, Level(level), None, &Progress::default())?;
        game.frame(&Pads::default());
        let seeds: Vec<[i32; 3]> = game.bodies.iter().map(|b| [b.pos.x as i32, b.max_y, b.pos.z as i32]).collect();
        let (lo, hi) = game.player.body_box();
        let body = Body { lo, hi };
        let feet = game.player.feet();
        let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ level as u64);
        let (mut same, mut bad, mut tried) = (0, 0, 0);
        let b = PLAYER;
        for i in 0..n {
            // A thing to stand on: near a seed, a model turned about y only.
            let s = seeds[rng.next() as usize % seeds.len().max(1)];
            put(&mut p, [s[0], s[1] - 600, s[2]], s[1]);
            oracle::rebuild(&mut p)?;
            let list = oracle::nearby(&mut p);
            let things: Vec<usize> = (0..list.len())
                .filter(|&k| {
                    let (_, sol, skip) = &list[k];
                    !skip && sol.angles[1] == 0 && sol.angles[2] == 0 && matches!(&sol.shape, Shape::Model(bx) if !bx.is_empty())
                })
                .collect();
            if things.is_empty() {
                continue;
            }
            let (addr, solid, _) = list[things[rng.next() as usize % things.len()]].clone();
            let Shape::Model(boxes) = &solid.shape else { continue };
            let j = rng.next() as usize % boxes.len();
            let (blo, bhi) = boxes[j];
            let at = |a: i32, b: i32, u: i32| a + (b - a) * u / 100;
            let local = [at(blo[0], bhi[0], rng.range(25, 75)), blo[1], at(blo[2], bhi[2], rng.range(25, 75))];
            let w = trig.turn_by(solid.angles, local);
            let floor = solid.pos[1] + w[1];
            let pos = [solid.pos[0] + w[0], floor + feet, solid.pos[2] + w[2]];
            let q = rugrats::world::collide::query(&game.col, [pos[0], floor - 64, pos[2]]);
            if q.kind != 0 || q.tile.is_none() || q.floor < floor {
                continue;
            }
            put(&mut p, pos, floor);
            oracle::rebuild(&mut p)?;
            let list = oracle::nearby(&mut p);
            let Some(k) = list.iter().position(|(o, ..)| *o == addr) else { continue };
            tried += 1;
            let heading = rng.range(0, 4095);
            let speed = if rng.next().is_multiple_of(2) { 17 } else { 8 };
            let way = [trig.sin(heading), 0, trig.cos(heading)];
            let mode = 1 + (i % 2) as u32;
            // The original.
            for (o, v) in way.iter().enumerate() {
                p.write_u32(b + 0x178 + 4 * o as u32, *v as u32);
            }
            p.write_u32(b + 0x184, speed as u32);
            p.write_u32(b + 0xf4, 0);
            p.write_u32(b + 0xf8, addr);
            p.hle.m.bus.load_ram(b + 0xfd, &[j as u8]);
            for o in 0..7u32 {
                p.write_u32(b + 0x104 + 4 * o, 0);
            }
            p.write_u32(b + 0x130, 0x7777);
            let flags = p.read_u32(b);
            p.write_u32(b, flags & !0x80);
            p.hle.m.bus.load_ram(b + 0x18c, &[2]);
            let dt = p.read_i32(0x800c_3f1c);
            let r = oracle::call(&mut p, PROBE, &[b, mode])? as i32;
            let rh = |p: &mut Play, a: u32| p.read_i16(a) as i32;
            let delta = p.read_i32(b + 0x144);
            let normal = [rh(&mut p, b + 0x114), rh(&mut p, b + 0x118), rh(&mut p, b + 0x11c)];
            let tangent = [rh(&mut p, b + 0x124), rh(&mut p, b + 0x128), rh(&mut p, b + 0x12c)];
            let slant = rh(&mut p, b + 0x130);
            let index_of = |p: &mut Play, at: u32| {
                let o = p.read_u32(at);
                (o != 0).then(|| list.iter().position(|(a, ..)| *a == o).unwrap_or(usize::MAX))
            };
            let hit = index_of(&mut p, b + 0xf4).map(|o| (o, p.read_u8(b + 0xfc)));
            let below = index_of(&mut p, b + 0xf8).map(|o| (o, p.read_u8(b + 0xfd)));
            let state = p.read_u8(b + 0x18c);
            let (oy, ofloor) = (p.read_i32(b + 0x1c), p.read_i32(b + 0x148));
            let ostep = (p.read_u32(b) & 0x80 != 0).then(|| (p.read_i32(b + 0x150), p.read_i32(b + 0x14c)));
            // The port.
            let near: Vec<Near> = list.iter().map(|(_, s, skip)| Near { solid: s, skip: *skip }).collect();
            let angles = [p.read_i16(b + 0x24), p.read_i16(b + 0x26), p.read_i16(b + 0x28)];
            let g = Ground { col: &game.col, trig: &trig, near: &near, body, limit: 256, angles, reflect: flags & 0x10_0000 != 0, low: p.read_i16(b + 0x68) as i32 };
            let mut st = Probes { below: Some((k, j as u8)), ..Probes::default() };
            let (mut py, mut pfloor, mut pstep, mut fell) = (pos[1], floor, None, false);
            if mode == 2 {
                match g.ahead(pos, floor, way, speed * dt, false, &mut st) {
                    Ahead::Fall => fell = true,
                    Ahead::Followed(d) => {
                        py += d;
                        pfloor += d;
                    }
                    Ahead::Step { from, to } => pstep = Some((from, to)),
                    Ahead::Kept => {}
                }
            }
            let port = if fell { None } else { Some(g.probe([pos[0], py, pos[2]], pfloor, way, speed * dt, &mut st)) };
            let record_ok = match port {
                None => r == 0 && state == 4,
                Some(Probed::Clear(rec)) => r == 0 && rec.delta == delta && rec.normal == normal && rec.hit == hit,
                Some(Probed::Held { record, slide: Some((t, a)) }) => {
                    r == 1 && record.delta == delta && record.normal == normal && t == tangent && a == slant && record.hit == hit
                }
                Some(Probed::Held { record, slide: None }) => r == 1 && record.delta == delta && slant == 0 && record.hit == hit,
            };
            let ok = record_ok && (fell || (oy == py && ofloor == pfloor && ostep == pstep && st.below == below));
            if ok {
                same += 1;
            } else {
                bad += 1;
                if bad <= 12 {
                    println!(
                        "level {level} mode {mode}: on {k} box {j} at {pos:?} heading {heading} speed {speed}: original {r} state {state} y {oy} floor {ofloor} step {ostep:?} below {below:?} delta {delta} normal {normal:?} tangent {tangent:?} slant {slant} hit {hit:?}; port y {py} floor {pfloor} step {pstep:?} below {:?} {port:?}",
                        st.below
                    );
                }
            }
        }
        println!("level {level}: {same} the same, {bad} different (of {tried} stood on a thing)");
        bad_total += bad;
    }
    if bad_total > 0 {
        return Err(format!("{bad_total} different"));
    }
    Ok(())
}
