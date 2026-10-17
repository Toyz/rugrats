//! Holds `actors::ground::Ground::air` to the original's air test
//! (0x80066014), called on the live player in the oracle.
//!
//! `air_check [LEVEL..] [--n N]`
//!
//! For each level (default 1 and 18) and N spots in the air (beside the
//! port's objects or about the start, up to 1500 over the floor), a move
//! (+0xd8: across up to 300 each way, up or down up to 400; one in six
//! straight up or down) is written with the position, the nearby list
//! rebuilt and the record cleared, and 0x80066014(player) called. Its
//! answer, the floor (+0x148), the object met (+0xf4, +0xfc), the point
//! and normal (+0x104, +0x114), the tangent and slant (+0x124, +0x130)
//! where answered, and the thing stood on (+0xf8, +0xfd) are compared.
use std::path::Path;
use std::rc::Rc;

use rugrats::actors::ground::{Air, Body, Ground, Near, Probes};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, PLAYER, Play};

#[path = "common/oracle.rs"]
mod oracle;

const AIR: u32 = 0x8006_6014;

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

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut levels = Vec::new();
    let mut n = 400;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--n" => n = args.next().and_then(|s| s.parse().ok()).unwrap_or(n),
            l => levels.push(l.parse::<u8>().map_err(|e| e.to_string())?),
        }
    }
    if levels.is_empty() {
        levels = vec![1, 18];
    }
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let trig = Trig::load(&disc)?;
    let mut bad_total = 0;
    for level in levels {
        let mut game = Game::open_from(&disc, Level(level), None, &Progress::default())?;
        game.frame(&Pads::default());
        let seeds: Vec<[i32; 3]> = game.bodies.iter().map(|b| [b.pos.x as i32, b.max_y, b.pos.z as i32]).collect();
        let feet = game.player.feet();
        let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
        let b = PLAYER;
        // The original's own body (the character it runs as).
        let h = p.read_u32(b + 0x154);
        let v3 = |p: &mut Play, a: u32| [0, 1, 2].map(|k| p.read_i16(a + 2 * k) as i32);
        let body = Body { lo: v3(&mut p, h + 4), hi: v3(&mut p, h + 12) };
        let start = p.player().pos;
        let (low, plane) = (p.read_i16(b + 0x68) as i32, p.read_i32(b + 0x13c));
        println!("level {level}: +0x68 {low} +0x13c {plane}");
        let mut rng = Rng(0x51ed_270b_27f1_9d43 ^ level as u64);
        let (mut same, mut bad) = (0, 0);
        let mut seen = [0u32; 4];
        for i in 0..n {
            let mut spot = None;
            for _ in 0..60 {
                let (x, y, z) = if i % 2 == 1 && !seeds.is_empty() {
                    let s = seeds[rng.next() as usize % seeds.len()];
                    (s[0] + rng.range(-700, 700), s[1] - 64, s[2] + rng.range(-700, 700))
                } else {
                    (start[0] + rng.range(-4000, 4000), start[1] - 600, start[2] + rng.range(-4000, 4000))
                };
                let q = rugrats::world::collide::query(&game.col, [x, y, z]);
                if q.kind == 0 && q.tile.is_some() {
                    spot = Some([x, q.floor, z]);
                    break;
                }
            }
            let Some([x, floor, z]) = spot else { continue };
            let up = rng.range(0, 1500);
            let pos = [x, floor + feet - up, z];
            let d = if rng.next().is_multiple_of(6) {
                [0, rng.range(-400, 400), 0]
            } else {
                [rng.range(-300, 300), rng.range(-400, 400), rng.range(-300, 300)]
            };
            for (k, v) in pos.iter().enumerate() {
                p.write_u32(b + 0x18 + 4 * k as u32, *v as u32);
            }
            p.write_u32(b + 0x148, floor as u32);
            oracle::rebuild(&mut p)?;
            let list = oracle::nearby(&mut p);
            for k in 0..3 {
                p.write_i16(b + 0xd8 + 2 * k as u32, d[k] as i16);
            }
            p.write_u32(b + 0xf4, 0);
            p.write_u32(b + 0xf8, 0);
            p.hle.m.bus.load_ram(b + 0xfd, &[255]);
            // The record cleared (+0x104 .. +0x133), the plane (+0x13c) kept.
            for k in 0..12u32 {
                p.write_u32(b + 0x104 + 4 * k, 0);
            }
            let flags = p.read_u32(b);
            let r = oracle::call(&mut p, AIR, &[b])? as i32;
            let rd = |p: &mut Play, a: u32| [0, 4, 8].map(|k| p.read_i32(a + k));
            let point = rd(&mut p, b + 0x104);
            let normal = rd(&mut p, b + 0x114);
            let tangent = rd(&mut p, b + 0x124);
            let slant = p.read_i32(b + 0x130);
            let ofloor = p.read_i32(b + 0x148);
            let index_of = |p: &mut Play, at: u32| {
                let o = p.read_u32(at);
                (o != 0).then(|| list.iter().position(|(a, ..)| *a == o).unwrap_or(usize::MAX))
            };
            let hit = index_of(&mut p, b + 0xf4).map(|o| (o, p.read_u8(b + 0xfc)));
            let below = index_of(&mut p, b + 0xf8).map(|o| (o, p.read_u8(b + 0xfd)));
            let near: Vec<Near> = list.iter().map(|(_, s, skip)| Near { solid: s, skip: *skip }).collect();
            let angles = [p.read_i16(b + 0x24), p.read_i16(b + 0x26), p.read_i16(b + 0x28)];
            let g = Ground { col: &game.col, trig: &trig, near: &near, body, limit: 256, angles, reflect: flags & 0x10_0000 != 0, low };
            let mut st = Probes::default();
            let mut pfloor = floor;
            let port = g.air(pos, d, plane, &mut pfloor, &mut st);
            let pr = match port {
                Air::Clear => 0,
                Air::Wall { .. } => 1,
                Air::Land => 2,
                Air::Slide(_) => 3,
            };
            seen[pr as usize] += 1;
            let turned_ok = match port {
                Air::Wall { turned: Some((t, a)) } => t == tangent && a == slant,
                Air::Slide(t) => t == tangent,
                _ => true,
            };
            let ok = r == pr && turned_ok && pfloor == ofloor && st.met.hit == hit && st.met.point == point && st.met.normal == normal && st.below == below;
            if ok {
                same += 1;
            } else {
                bad += 1;
                if bad <= 12 {
                    println!(
                        "level {level}: at {pos:?} d {d:?} ({} near): original {r} floor {ofloor} hit {hit:?} point {point:?} normal {normal:?} tangent {tangent:?} slant {slant} below {below:?}; port {port:?} floor {pfloor} {:?} below {:?}",
                        list.len(),
                        st.met,
                        st.below
                    );
                }
            }
        }
        println!("level {level}: {same} the same, {bad} different (port's answers clear/wall/land/slide {seen:?})");
        bad_total += bad;
    }
    if bad_total > 0 {
        return Err(format!("{bad_total} different"));
    }
    Ok(())
}
