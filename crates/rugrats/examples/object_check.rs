//! Holds `actors::ground::objects` to the original's object probe
//! (0x80063d3c), called on the live player in the oracle.
//!
//! `object_check [LEVEL..] [--n N]`
//!
//! For each level (default 1 and 18) the port's objects seed the spots: N
//! spots each beside one of them, a heading and a speed each. There the
//! original's player is put, its nearby list rebuilt (0x80062ed4), the
//! list read back as the port's solids (position +0x18, angles +0x24, the
//! bound +0x78, the model's boxes or the hierarchy's), and 0x80063d3c
//! called for the middle probe's offset. The object met (+0xf4, by its
//! place in the list), its box (+0xfc) and the point and normal last
//! written (+0x104, +0x114) are compared with the port's.
use std::path::Path;
use std::rc::Rc;

use rugrats::actors::ground::{self, Body, Near};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, PLAYER, Play};

#[path = "common/oracle.rs"]
mod oracle;

const OBJECTS: u32 = 0x8006_3d3c;
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
        let mut game = Game::open_from(&disc, Level(level), None, &Progress::default())?;
        game.frame(&Pads::default());
        let seeds: Vec<[i32; 3]> = game.bodies.iter().map(|b| [b.pos.x as i32, b.max_y, b.pos.z as i32]).collect();
        let (lo, hi) = game.player.body_box();
        let body = Body { lo, hi };
        let feet = game.player.feet();
        let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
        let mut rng = Rng(0x2545_f491_4f6c_dd1d ^ level as u64);
        let (mut same, mut bad, mut met) = (0, 0, 0);
        let sp = p.hle.m.cpu.r[29];
        let buf = sp.wrapping_sub(0x2000) & !15;
        for _ in 0..n {
            let mut spot = None;
            let mut seed = [0; 3];
            for _ in 0..60 {
                let s = seeds[rng.next() as usize % seeds.len().max(1)];
                seed = s;
                let x = s[0] + rng.range(-500, 500);
                let z = s[2] + rng.range(-500, 500);
                let q = rugrats::world::collide::query(&game.col, [x, s[1] - 64, z]);
                if q.kind == 0 && q.tile.is_some() {
                    spot = Some([x, q.floor, z]);
                    break;
                }
            }
            let Some([x, floor, z]) = spot else { continue };
            let pos = [x, floor + feet, z];
            // Mostly toward the seed's object.
            let toward = ((((seed[0] - x) as f64).atan2((seed[2] - z) as f64)) * 4096.0 / std::f64::consts::TAU) as i32;
            let heading = if rng.next().is_multiple_of(4) { rng.range(0, 4095) } else { (toward + rng.range(-300, 300)).rem_euclid(4096) };
            let speed = if rng.next().is_multiple_of(2) { 17 } else { 8 };
            let b = PLAYER;
            for (k, v) in pos.iter().enumerate() {
                p.write_u32(b + 0x18 + 4 * k as u32, *v as u32);
            }
            p.write_u32(b + 0x148, floor as u32);
            oracle::rebuild(&mut p)?;
            let list = oracle::nearby(&mut p);
            // The middle probe's offset, as `ground::probe` makes it.
            let reach = lo[2].abs().max(hi[2].abs()) + speed * p.read_i32(0x800c_3f1c);
            let m = trig.heading_matrix(trig.ratan2(trig.sin(heading), trig.cos(heading)));
            let off = trig.apply(&m, [0, 0, reach]);
            for k in 0..3u32 {
                p.write_u32(buf + 4 * k, off[k as usize] as u32);
                p.write_u32(b + 0x104 + 4 * k, 0x5555);
                p.write_u32(b + 0x114 + 4 * k, 0x5555);
            }
            p.write_u32(b + 0xf8, 0);
            let gp = std::mem::replace(&mut p.hle.m.cpu.r[28], GP);
            let r = p.hle.call_nested(OBJECTS, &[b, buf]);
            p.hle.m.cpu.r[28] = gp;
            r.map_err(|e| format!("{e:?}"))?;
            let hit_obj = p.read_u32(b + 0xf4);
            let hit_box = p.read_u8(b + 0xfc);
            let point = [0, 4, 8].map(|k| p.read_i32(b + 0x104 + k));
            let normal = [0, 4, 8].map(|k| p.read_i32(b + 0x114 + k));
            let orig_hit = (hit_obj != 0).then(|| (list.iter().position(|(o, ..)| *o == hit_obj).unwrap_or(usize::MAX), hit_box));
            let near: Vec<Near> = list.iter().map(|(_, s, skip)| Near { solid: s, skip: *skip }).collect();
            let mut port = ground::Met { hit: None, point: [0x5555; 3], index: 0, normal: [0x5555; 3] };
            ground::objects(&game.col, &trig, pos, body, None, off, &near, &mut port);
            let ok = port.hit == orig_hit && port.point == point && port.normal == normal;
            met += orig_hit.is_some() as u32;
            if ok {
                same += 1;
            } else {
                bad += 1;
                if bad <= 12 {
                    println!(
                        "level {level}: at {pos:?} heading {heading} off {off:?} ({} near): original {orig_hit:?} {point:?} {normal:?}; port {port:?}",
                        list.len()
                    );
                }
            }
        }
        println!("level {level}: {same} the same ({met} met), {bad} different");
        bad_total += bad;
    }
    if bad_total > 0 {
        return Err(format!("{bad_total} different"));
    }
    Ok(())
}
