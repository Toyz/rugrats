//! A thing picked up and thrown, on the port and on the original: the
//! thing's position each frame, side by side.
//!
//! `throw_check LEVEL THING [HEADING] [AWAY]`
//!
//! Both players are put AWAY (default 700) from thing THING (the port's
//! index) facing it, Circle picks it up, Triangle throws it, and the
//! thing's position (the original's object from player +0x1f0, +0x18) is
//! printed each frame.
use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rrt::emu::BusExt;
use rugrats_hle::play::{Boot, PLAYER, Play};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<i32>().unwrap_or(0));
    let level = args.next().unwrap_or(1) as u8;
    let k = args.next().unwrap_or(8) as usize;
    let turn = args.next();
    let away = args.next().filter(|&a| a != 0).unwrap_or(700);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let mut g = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
    let t = g.things[k].pos;
    let heading = turn.unwrap_or(0) & 4095;
    let a = heading as f64 * std::f64::consts::TAU / 4096.0;
    let floor = rugrats::world::collide::query(&g.col, [t.x as i32, t.y as i32 - 64, t.z as i32]).floor;
    let pos = [t.x as i32 - (a.sin() * away as f64) as i32, floor + g.player.feet(), t.z as i32 - (a.cos() * away as f64) as i32];
    p.put(pos, heading as i16);
    g.player.pos = pos;
    g.player.heading = heading;
    g.player.target = heading;
    g.player.speed = 0;
    g.set_camera_yaw(heading as f32);
    if std::env::var_os("HOOKS").is_some() {
        p.hle.m.hook(0x8006_1f88, |c| {
            let (o, pl) = (c.arg32(0), c.arg32(2));
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let op = [w(o + 0x18), w(o + 0x1c), w(o + 0x20)];
            let pp = [w(pl + 0x18), w(pl + 0x1c), w(pl + 0x20)];
            println!("     DESC obj {o:#x} at {op:?} player at {pp:?}");
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8008_864c, |c| {
            let d = c.arg32(0);
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let v: Vec<i32> = (0..16).map(|k| w(d + 4 * k)).collect();
            println!("     LAUNCH desc {v:?}");
            rrt::emu::HookResult::Continue
        });
    }
    let script = [(0u16, 6usize), (1 << input::CIRCLE, 1), (0, 40), (1 << input::TRIANGLE, 1), (0, 80)];
    let mut held: std::collections::VecDeque<u16> = std::iter::repeat_n(0, 1).collect();
    let mut pads = Pads::default();
    let mut obj = 0u32;
    let mut f = 0;
    for (bits, n) in script {
        for _ in 0..n {
            held.push_back(bits);
            pads.push_bits(0, held.pop_front().unwrap_or(0));
            g.frame(&pads);
            p.frame(bits)?;
            let o = p.read_u32(PLAYER + 0x1f0);
            if o != 0 {
                obj = o;
            }
            let opos = if obj != 0 { Some([0, 4, 8].map(|k| p.read_i32(obj + 0x18 + k))) } else { None };
            let tp = g.things[k].pos;
            let port = [tp.x as i32, tp.y as i32, tp.z as i32];
            let me = p.player();
            let hand = |p: &mut Play, slot: u32| {
                let h = p.read_u32(PLAYER + 0x154);
                let part = p.read_u8(h + 0x14 + slot) as u32;
                let m = p.read_u32(PLAYER + 0x158);
                [0, 4, 8].map(|k| p.read_i32(m + 32 * part + 20 + k))
            };
            let (oh15, oh12) = (hand(&mut p, 15), hand(&mut p, 12));
            let parts = g.player.parts(&g.anims, &g.trig);
            let ph = |slot| g.player.slot_part(slot).and_then(|i| parts.get(i)).map(|x| [x.1.x as i32, x.1.y as i32, x.1.z as i32]);
            if std::env::var_os("TRACKS").is_some() {
                let raw: Vec<String> = (0..4u32)
                    .map(|k| (0..6u32).map(|i| format!("{:04x}", p.read_i16(PLAYER + 0xa8 + 12 * k + 2 * i) as u16)).collect::<Vec<_>>().join(" "))
                    .collect();
                println!("     tracks port {} | orig {}", g.player.animator.describe(), raw.join(" | "));
            }
            if std::env::var_os("BOUND").is_some() && obj != 0 {
                let bound: Vec<i16> = (0..8).map(|k| p.read_i16(obj + 0x78 + 2 * k)).collect();
                let angles: Vec<i16> = (0..3).map(|k| p.read_i16(obj + 0x24 + 2 * k)).collect();
                let fl = p.read_u32(obj + 0x90);
                let fpos = if fl != 0 { Some([0, 4, 8].map(|k| p.read_i32(fl + 0x10 + k))) } else { None };
                let fv = if fl != 0 { Some([0, 4, 8].map(|k| p.read_i32(fl + 0x3c + k))) } else { None };
                println!("     bound {bound:?} angles {angles:?} flight pos {fpos:?} v {fv:?} radius {} flags {:#x}", if fl != 0 { p.read_i32(fl + 0x1c) } else { 0 }, if fl != 0 { p.read_u32(fl + 8) } else { 0 });
            }
            if std::env::var_os("QUATS").is_some() {
                let h = p.read_u32(PLAYER + 0x154);
                let n = p.read_i16(h) as u32;
                let oq: Vec<[i32; 4]> = (0..n).map(|i| [0, 4, 8, 12].map(|k| p.read_i32(h + 0x34 + 40 * i + 4 + k))).collect();
                let pq = g.player.animator.turns(&g.player.hierarchy);
                println!("     quats orig {:?}", oq);
                println!("     quats port {:?}", pq.iter().map(|q| q.unwrap_or([0; 4])).collect::<Vec<_>>());
            }
            if std::env::var_os("HANDS").is_some() {
                println!("     hand15 port {:?} orig {oh15:?} | hand12 port {:?} orig {oh12:?}", ph(15), ph(12));
            }
            let fl = if obj != 0 { p.read_u32(obj + 0x90) } else { 0 };
            println!(
                "{f:3}: port {port:?} carried {:?} me {:?} | orig {opos:?} flight {fl:#x} me {:?} s{} {}",
                g.carried(),
                g.player.pos,
                me.pos,
                me.state,
                opos.map_or(String::new(), |o| format!("d {:?}", [0, 1, 2].map(|i| port[i] - o[i])))
            );
            f += 1;
        }
    }
    Ok(())
}
