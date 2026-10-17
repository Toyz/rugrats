//! Calls the original ground probe (0x80064a34) on the player at a spot,
//! and the port's, and prints both.
//!
//! `ground_at LEVEL CHAR X Y Z HEADING SPEED [MODE]`
use std::path::Path;
use std::rc::Rc;

use rrt::emu::BusExt;
use rugrats::actors::ground::{self, Body};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, PLAYER, Play};

fn main() -> Result<(), String> {
    let a: Vec<i32> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let [level, ch, x, y, z, heading, speed] = a[..7] else { return Err("args".into()) };
    let mode = a.get(7).copied().unwrap_or(1) as u32;
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let trig = Trig::load(&disc)?;
    let game = Game::open_from(&disc, Level(level as u8), None, &Progress::default())?;
    let character = (ch > 0).then_some(ch as u32);
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level: level as u8, character, ..Default::default() })?;
    let b = PLAYER;
    let h = p.read_u32(b + 0x154);
    let bx: Vec<i32> = (0..8).map(|k| p.read_i16(h + 4 + 2 * k) as i32).collect();
    let limit_ix = p.read_u8(b + 0x15e);
    let limit = p.read_i16(0x800c_430c + 2 * limit_ix as u32) as i32;
    let floor = rugrats::world::collide::query(&game.col, [x, y, z]).floor;
    println!("body box {bx:?} 15c {} 15e {limit_ix} limit {limit} floor {floor}", p.read_i16(b + 0x15c));
    let way = [trig.sin(heading), 0, trig.cos(heading)];
    for (k, v) in [x, y, z].iter().enumerate() {
        p.write_u32(b + 0x18 + 4 * k as u32, *v as u32);
    }
    p.write_u32(b + 0x148, floor as u32);
    for (k, v) in way.iter().enumerate() {
        p.write_u32(b + 0x178 + 4 * k as u32, *v as u32);
    }
    p.write_u32(b + 0x184, speed as u32);
    p.write_u32(b + 0xf4, 0);
    p.write_u32(b + 0xf8, 0);
    let gp = std::mem::replace(&mut p.hle.m.cpu.r[28], 0x800c_3e80);
    let r = p.hle.call_nested(0x8006_4a34, &[b, mode]);
    p.hle.m.cpu.r[28] = gp;
    let rh = |p: &mut Play, a: u32| p.hle.m.bus.read_u16(a).unwrap_or(0) as i16 as i32;
    println!(
        "original: {r:?} delta {} normal {:?} slant {} f4 {:#x} pos now {:?}",
        p.read_i32(b + 0x144),
        [rh(&mut p, b + 0x114), rh(&mut p, b + 0x118), rh(&mut p, b + 0x11c)],
        rh(&mut p, b + 0x130),
        p.read_u32(b + 0xf4),
        p.player().pos
    );
    let body = Body { lo: [bx[0], bx[1], bx[2]], hi: [bx[4], bx[5], bx[6]] };
    println!("port: {:?}", ground::probe(&game.col, &trig, [x, y, z], floor, way, speed * 10, body, limit));
    Ok(())
}
