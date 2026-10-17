//! Holds `actors::ground::probe` to the original ground probe (0x80064a34),
//! called on the live player in the oracle with the same position, floor,
//! way and speed, objects out of it.
//!
//! `ground_check [LEVEL..] [--n N]`
//!
//! For each level (default 1 and 18) the original is booted into it
//! (`rugrats_hle::play`), then for N spots in the open about the player's
//! start, with a heading and a speed each, the player's +0x18 (position),
//! +0x148 (floor), +0x178 (way) and +0x184 (speed) are written, +0xf4 and
//! +0xf8 (the objects met) cleared, and 0x80064a34(player, 1) called. Its
//! return, +0x144 (the floor delta), +0x114 (the wall's normal), +0x124
//! (the tangent) and +0x130 (the slant) are compared with the port's.
use std::path::Path;
use std::rc::Rc;

use rrt::emu::BusExt;
use rugrats::actors::ground::{self, Body, Probed};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, PLAYER, Play};

const PROBE: u32 = 0x8006_4a34;
/// The game's gp.
const GP: u32 = 0x800c_3e80;

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
        let game = Game::open_from(&disc, Level(level), None, &Progress::default())?;
        let (lo, hi) = game.player.body_box();
        let body = Body { lo, hi };
        let feet = game.player.feet();
        let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
        let start = p.player().pos;
        // The angles and vectors the original turns (0x80075ee8).
        let turns = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
        {
            let turns = turns.clone();
            p.hle.m.hook(0x8007_5ee8, move |c| {
                use rrt::emu::Cpu;
                let (a0, a1) = (c.cpu.reg(4) as u32, c.cpu.reg(5) as u32);
                let ang: Vec<i16> = (0..3).map(|k| c.bus.read_u16(a0 + 2 * k).unwrap_or(0) as i16).collect();
                let v: Vec<i32> = (0..3).map(|k| c.bus.read_u32(a1 + 4 * k).unwrap_or(0) as i32).collect();
                turns.borrow_mut().push(format!("{ang:?} {v:?}"));
                rrt::emu::HookResult::Continue
            });
        }
        // The middle probe's offset as the original hands it to the slide
        // (0x800650e8: a1 = sp + 304).
        let seen = std::rc::Rc::new(std::cell::Cell::new([0i32; 3]));
        {
            let seen = seen.clone();
            p.hle.m.hook(0x8006_50e8, move |c| {
                use rrt::emu::Cpu;
                let sp = c.cpu.reg(29) as u32;
                seen.set([0, 4, 8].map(|k| c.bus.read_u32(sp + 304 + k).unwrap_or(0) as i32));
                rrt::emu::HookResult::Continue
            });
        }
        let mut rng = Rng(0x2545_f491_4f6c_dd1d ^ level as u64);
        let (mut same, mut bad) = (0, 0);
        for _ in 0..n {
            // A spot in the open: its floor under it.
            let mut spot = None;
            for _ in 0..60 {
                let x = start[0] + rng.range(-4000, 4000);
                let z = start[2] + rng.range(-4000, 4000);
                let q = rugrats::world::collide::query(&game.col, [x, start[1] - 600, z]);
                if q.kind == 0 && q.tile.is_some() {
                    spot = Some([x, q.floor, z]);
                    break;
                }
            }
            let Some([x, floor, z]) = spot else { continue };
            let pos = [x, floor + feet, z];
            let heading = rng.range(0, 4095);
            let speed = if rng.next().is_multiple_of(2) { 17 } else { 8 };
            let way = [trig.sin(heading), 0, trig.cos(heading)];
            let port = ground::probe(&game.col, &trig, pos, floor, way, speed * 10, body, 256);
            // The original, on the same player.
            let b = PLAYER;
            for (k, v) in pos.iter().enumerate() {
                p.write_u32(b + 0x18 + 4 * k as u32, *v as u32);
            }
            p.write_u32(b + 0x148, floor as u32);
            for (k, v) in way.iter().enumerate() {
                p.write_u32(b + 0x178 + 4 * k as u32, *v as u32);
            }
            p.write_u32(b + 0x184, speed as u32);
            p.write_u32(b + 0xf4, 0);
            p.write_u32(b + 0xf8, 0);
            p.write_u32(b + 0x130, 0x7777);
            turns.borrow_mut().clear();
            let gp = std::mem::replace(&mut p.hle.m.cpu.r[28], GP);
            let r = p.hle.call_nested(PROBE, &[b, 1]);
            p.hle.m.cpu.r[28] = gp;
            let r = match r {
                Ok(r) => r as i32,
                Err(e) => {
                    println!("level {level}: trap at {pos:?} heading {heading}: {e:?}");
                    bad += 1;
                    continue;
                }
            };
            let rd = |p: &mut Play, a: u32| p.hle.m.bus.read_u32(a).unwrap_or(0) as i32;
            let rh = |p: &mut Play, a: u32| p.hle.m.bus.read_u16(a).unwrap_or(0) as i16 as i32;
            let delta = rd(&mut p, b + 0x144);
            let normal = [rh(&mut p, b + 0x114), rh(&mut p, b + 0x118), rh(&mut p, b + 0x11c)];
            let tangent = [rh(&mut p, b + 0x124), rh(&mut p, b + 0x128), rh(&mut p, b + 0x12c)];
            let slant = rh(&mut p, b + 0x130);
            let ok = match port {
                Probed::Clear(rec) => r == 0 && rec.delta == delta && rec.normal == normal,
                Probed::Held { record, slide: Some((t, a)) } => {
                    r == 1 && record.delta == delta && record.normal == normal && t == tangent && a == slant
                }
                Probed::Held { record, slide: None } => r == 1 && record.delta == delta && slant == 0,
            };
            if ok {
                same += 1;
            } else {
                bad += 1;
                if bad <= 12
                    && let Probed::Held { record, slide: Some(_) } = port
                {
                    // The original's tangent routine on the port's own way
                    // back and normal.
                    let reach = body.lo[2].abs().max(body.hi[2].abs()) + speed * 10;
                    let ang = trig.ratan2(way[0], way[2]);
                    let off = trig.turn(ang, [0, 0, reach]);
                    let back = [-off[0], 0, -off[2]];
                    let n = [record.normal[0], 0, record.normal[2]];
                    let sp = p.hle.m.cpu.r[29];
                    let buf = sp.wrapping_sub(0x1000) & !15;
                    for k in 0..3 {
                        p.hle.m.bus.write_u32(buf + 4 * k, back[k as usize] as u32).ok();
                        p.hle.m.bus.write_u32(buf + 16 + 4 * k, n[k as usize] as u32).ok();
                    }
                    let gp = std::mem::replace(&mut p.hle.m.cpu.r[28], GP);
                    let _ = p.hle.call_nested(0x8007_63d0, &[buf, buf + 16, buf + 32]);
                    p.hle.m.cpu.r[28] = gp;
                    let o: Vec<i32> = (0..4).map(|k| p.hle.m.bus.read_u32(buf + 32 + 4 * k).unwrap_or(0) as i32).collect();
                    println!("  original's offset {:?}, port's {off:?}; angles {ang:?}; turns {:?}", seen.get(), turns.borrow());
                    println!("  original's 0x800763d0 on the port's back {back:?} and normal {n:?}: {o:?}; port {:?}", ground::tangent(back, n, &trig));
                }
                if bad <= 12 {
                    println!(
                        "level {level}: at {pos:?} heading {heading} speed {speed}: original {r} delta {delta} normal {normal:?} tangent {tangent:?} slant {slant}; port {port:?}"
                    );
                }
            }
        }
        println!("level {level}: {same} the same, {bad} different");
        bad_total += bad;
    }
    if bad_total > 0 {
        return Err(format!("{bad_total} different"));
    }
    Ok(())
}
