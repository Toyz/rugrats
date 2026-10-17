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
use rrt::emu::Cpu;
use rugrats_hle::play::{Boot, PLAYER, Play};

/// The original's objects in the rooms' lists (0x800c4a8c).
fn objects(p: &mut Play) -> Vec<u32> {
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    let mut out = Vec::new();
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                out.push(p.read_u32(node + 8));
                node = p.read_u32(node + 4);
            }
        }
    }
    out
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<i32>().unwrap_or(0));
    let level = args.next().unwrap_or(1) as u8;
    let k = args.next().unwrap_or(8) as usize;
    let turn = args.next();
    let away = args.next().filter(|&a| a != 0).unwrap_or(700);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let mut g = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    // The level's own player (level 6 and 10 Chuckie, 4 and 11 Spike, 5
    // Phil or Lil, 7 Reptar, 17 Angelica).
    let character = Some(g.character as u32).filter(|&c| c != 1);
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, character, cinematics: std::env::var_os("CINES").is_some() })?;
    if let Some(n) = std::env::var("SETTLE").ok().and_then(|s| s.parse::<usize>().ok()) {
        for _ in 0..n {
            p.frame(0)?;
        }
        let me = p.player();
        println!("oracle settled: player at {:?}", me.pos);
    }
    if std::env::var_os("LIST").is_some() {
        for (i, t) in g.things.iter().enumerate() {
            println!("thing {i}: template {} at {:?}", t.template, t.pos);
        }
        for n in &g.npcs {
            println!("npc {}: at {:?}", n.id, n.pos);
        }
    }
    let t = g.things[k].pos;
    let heading = turn.unwrap_or(0) & 4095;
    let a = heading as f64 * std::f64::consts::TAU / 4096.0;
    let (x, z) = (t.x as i32 - (a.sin() * away as f64) as i32, t.z as i32 - (a.cos() * away as f64) as i32);
    let floor = rugrats::world::collide::query(&g.col, [x, t.y as i32 - 64, z]).floor;
    let pos = [x, floor + g.player.feet(), z];
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
    // MEETS=1: the roll's meeting (0x80086668 in 0x80086ef0): what, the
    // segment, the point and the normal.
    if std::env::var_os("MEETS").is_some() {
        p.hle.m.hook(0x8008_718c, |c| {
            let v0 = c.cpu.reg(2) as u32;
            let sp = c.cpu.reg(29) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let seg = [w(sp + 0x70), w(sp + 0x74), w(sp + 0x78), w(sp + 0x80), w(sp + 0x84), w(sp + 0x88)];
            let point = [w(sp + 0x50), w(sp + 0x54), w(sp + 0x58)];
            let normal = [w(sp + 0x40), w(sp + 0x44), w(sp + 0x48)];
            println!("     MEET orig {v0:#x} seg {seg:?} point {point:?} normal {normal:?}");
            rrt::emu::HookResult::Continue
        });
    }
    if std::env::var_os("MEETS").is_some() {
        p.hle.m.hook(0x8008_6a20, |c| {
            println!("     SLOPE orig ny {} dt {}", c.arg32(0) as i16, c.arg32(1) as i32);
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8008_6dd8, |c| {
            let push = c.cpu.reg(2) as i32;
            let sp = c.cpu.reg(29) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            println!("     SLOPE orig push {push} u [{}, {}, {}] n [{}, {}, {}] s0 {} s3 {}", w(sp + 0x10), w(sp + 0x14), w(sp + 0x18), w(sp + 0x20), w(sp + 0x24), w(sp + 0x28), c.cpu.reg(16) as i32, c.cpu.reg(19) as i32);
            rrt::emu::HookResult::Continue
        });
    }
    // PUTDOWN=1: Circle again to put it down instead of the throw.
    let second = if std::env::var_os("PUTDOWN").is_some() { input::CIRCLE } else { input::TRIANGLE };
    if let Ok(list) = std::env::var("HOOKFNS") {
        for a in list.split(',').filter_map(|a| u32::from_str_radix(a.trim_start_matches("0x"), 16).ok()) {
            p.hle.m.hook(a, move |c| {
                println!("     CALL {a:#010x} a0 {:#x} a1 {:#x}", c.arg32(0), c.arg32(1));
                rrt::emu::HookResult::Continue
            });
        }
    }
    // SYNCNPC=1: each NPC given its original object's place and heading
    // after every frame (the nearest body at the start).
    let synced: Vec<(usize, u32)> = if std::env::var_os("SYNCNPC").is_some() {
        let objs = objects(&mut p);
        let mut bodies: Vec<(u32, [i32; 3])> = Vec::new();
        for o in objs {
            if p.read_u32(o + 0x154) != 0 {
                bodies.push((o, [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i))));
            }
        }
        g.npcs
            .iter()
            .enumerate()
            .filter_map(|(k, n)| {
                bodies
                    .iter()
                    .map(|&(o, at)| ((0..3).map(|i| (at[i] - n.pos[i]).abs() as i64).sum::<i64>(), o))
                    .min()
                    .filter(|&(d, _)| d < 200)
                    .map(|(_, o)| (k, o))
            })
            .collect()
    } else {
        Vec::new()
    };
    let script = [(0u16, 6usize), (1 << input::CIRCLE, 1), (0, 40), (1 << second, 1), (0, 80)];
    let mut held: std::collections::VecDeque<u16> = std::iter::repeat_n(0, 1).collect();
    let mut pads = Pads::default();
    let mut obj = 0u32;
    let mut f = 0;
    for (bits, n) in script {
        for _ in 0..n {
            held.push_back(bits);
            pads.push_bits(0, held.pop_front().unwrap_or(0));
            // The synced characters: their animation as the original's
            // stood (as last drawn), then their place after its frame, held
            // there through the port's.
            for &(k, o) in &synced {
                let t: [[i16; 6]; 4] = std::array::from_fn(|j| std::array::from_fn(|i| p.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32)));
                let speed = p.read_i16(o + 0xe4) as i32;
                let after = (p.read_u8(o + 0xe6), p.read_u8(o + 0xe7));
                let fidget = (p.read_i16(o + 0xf0), p.read_i16(o + 0xf2) as i32);
                let hier = p.read_u32(o + 0x154);
                let parts = p.read_i16(hier) as u32;
                let mut pose = vec![None; 32];
                for (channel, slot) in pose.iter_mut().enumerate() {
                    let part = p.read_u8(hier + 0x14 + channel as u32) as u32;
                    if part < parts {
                        let q = [0, 4, 8, 12].map(|c| p.read_i32(hier + 0x34 + 40 * part + 4 + c));
                        if q != [0; 4] {
                            *slot = Some(q);
                        }
                    }
                }
                let n = &mut g.npcs[k];
                n.animator.load_record(t, speed, after, fidget, pose, &g.anims);
                n.drawn = None;
            }
            p.frame(bits)?;
            for &(k, o) in &synced {
                let n = &mut g.npcs[k];
                n.pos = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
                n.heading = p.read_i16(o + 0x24) as i32 & 4095;
                n.mode = rugrats::actors::npc::Mode::Idle;
            }
            g.frame(&pads);
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
                if fl != 0 {
                    let hit = p.read_u32(fl + 0x98);
                    if hit != 0 {
                        println!("     last hit {hit:#x} template {} at {:?} class {}", p.read_i16(hit + 0x96), [0, 4, 8].map(|k| p.read_i32(hit + 0x18 + k)), p.read_u8(hit + 0xa6));
                    }
                }
            }
            if std::env::var_os("AIM").is_some() {
                let t = p.read_u32(PLAYER + 0x1f4);
                let theirs = (t != 0).then(|| [0, 4, 8].map(|k| p.read_i32(t + 0x18 + k)));
                let mine = g.aim_target.map(|k| g.npcs[k].pos);
                println!("     aim port {mine:?} orig {theirs:?}");
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
