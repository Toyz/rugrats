//! One thing put down onto another, on the port and on the original:
//! what each says rests on what (+0x70, +0x74, 0x8006c7a0), and what GRAB
//! takes after.
//!
//! `stack_check LEVEL A B [AWAY] [HEADING]`
//!
//! Both players are put 700 from thing A (the port's index) facing it,
//! Circle picks it up; then put AWAY (default 400) from thing B facing it
//! along HEADING, Circle puts A down. Then Circle again (GRAB), and the
//! thing each takes is printed. LIST=1 lists the things.
use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::{Game, Held};
use rrt::emu::{BusExt, Cpu};
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

/// The original's object nearest `at`.
fn nearest(p: &mut Play, at: [i32; 3]) -> u32 {
    let objs = objects(p);
    objs.into_iter().map(|o| ((0..3).map(|i| (p.read_i32(o + 0x18 + 4 * i as u32) - at[i]).abs() as i64).sum::<i64>(), o)).min().map_or(0, |(_, o)| o)
}

/// The items of a 0x8001a1c0 list from its head (newest first: the
/// nexts at +4, the items at +8).
fn ring(p: &mut Play, list: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut node = list;
    while node != 0 && out.len() < 16 {
        out.push(p.read_u32(node + 8));
        node = p.read_u32(node + 4);
    }
    out
}

/// Both players `away` from `at` along `heading`, facing it, on the
/// floor there.
fn put(g: &mut Game, p: &mut Play, at: [i32; 3], heading: i32, away: i32) {
    let a = heading as f64 * std::f64::consts::TAU / 4096.0;
    let (x, z) = (at[0] - (a.sin() * away as f64) as i32, at[2] - (a.cos() * away as f64) as i32);
    let floor = rugrats::world::collide::query(&g.col, [x, at[1] - 64, z]).floor;
    let pos = [x, floor + g.player.feet(), z];
    // What is carried moves with the player (the original's +0x18 is
    // otherwise left behind until its draw, stretching the carry box).
    let d = [0, 1, 2].map(|i| pos[i] - g.player.pos[i]);
    let o = p.read_u32(PLAYER + 0x1f0);
    if o != 0 {
        for i in 0..3u32 {
            let v = p.read_i32(o + 0x18 + 4 * i);
            p.write_u32(o + 0x18 + 4 * i, (v + d[i as usize]) as u32);
        }
    }
    if let Some(k) = g.carried() {
        g.things[k].pos += rrt::glam::Vec3::new(d[0] as f32, d[1] as f32, d[2] as f32);
    }
    p.put(pos, heading as i16);
    g.player.pos = pos;
    g.player.heading = heading;
    g.player.target = heading;
    g.player.speed = 0;
    g.set_camera_yaw(heading as f32);
}

fn run(g: &mut Game, p: &mut Play, pads: &mut Pads, script: &[(u16, usize)], a: usize, oa: u32) -> Result<(), String> {
    for &(bits, n) in script {
        for _ in 0..n {
            pads.push_bits(0, bits);
            p.frame(bits)?;
            g.frame(pads);
            if std::env::var_os("TRACE").is_some() {
                let me = p.player();
                println!("  player port {:?} st {:?} act {:?} on {:?} box {:?} | orig {:?} st {} speed {}", g.player.pos, g.player.state, g.player.action, g.player.support.map(|b| g.bodies[b].template), g.player.body_box(), me.pos, me.state, me.speed);
                if oa != 0 {
                    let t = &g.things[a];
                    println!("    A port {:?} {:?} | orig {:?}", [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32], t.state, [0, 4, 8].map(|i| p.read_i32(oa + 0x18 + i)));
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<i32>().unwrap_or(0));
    let level = args.next().unwrap_or(6) as u8;
    let a = args.next().unwrap_or(0) as usize;
    let b = args.next().unwrap_or(1) as usize;
    let away = args.next().filter(|&a| a != 0).unwrap_or(400);
    let heading = args.next().unwrap_or(0) & 4095;
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let mut g = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    if std::env::var_os("LIST").is_some() {
        for (i, t) in g.things.iter().enumerate() {
            let boxes = g.set.models.get(t.shape).map(|m| m.boxes.clone()).unwrap_or_default();
            println!("thing {i}: template {} at {:?} id {} hidden {} state {:?} boxes {boxes:?}", t.template, t.pos, t.id, t.hidden, t.state);
        }
        for (i, gd) in g.goodies.iter().enumerate() {
            println!("goodie {i}: template {} flags {:#x} kind {} at {:?} taken {}", gd.template, g.set.templates[gd.template as usize].flags, gd.kind, gd.pos, gd.taken);
        }
        return Ok(());
    }
    let character = Some(g.character as u32).filter(|&c| c != 1);
    // SETUP=template: each 0x80046570 on an object of that template during
    // the boot, with its first box as it is then.
    let setup = std::env::var("SETUP").ok().and_then(|v| v.parse::<u16>().ok());
    let mut p = Play::boot_with(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, character, cinematics: false }, move |hle| {
        if let Some(want) = setup {
            hle.m.hook(0x8004_6570, move |c| {
                let o = c.arg32(0);
                let t = c.bus.read_u16(o + 0x96).unwrap_or(0);
                if t == want {
                    let l = c.bus.read_u32(o + 0x88).unwrap_or(0);
                    let bp = if l != 0 { c.bus.read_u32(l + 0xc).unwrap_or(0) } else { 0 };
                    let b: Vec<i16> = (0..8).map(|i| c.bus.read_u16(bp + 2 * i).unwrap_or(0) as i16).collect();
                    let at: Vec<i32> = (0..3).map(|i| c.bus.read_u32(o + 0x18 + 4 * i).unwrap_or(0) as i32).collect();
                    println!("     SETUP {o:#x} template {t} at {at:?} +0x88 {l:#x} boxes at {bp:#x} box {b:?} ra {:#x}", c.cpu.reg(31));
                }
                rrt::emu::HookResult::Continue
            });
        }
    })?;
    let ipos = |v: rrt::glam::Vec3| [v.x as i32, v.y as i32, v.z as i32];
    let (pa, pb) = (ipos(g.things[a].pos), ipos(g.things[b].pos));
    // OBJBOUNDS=1: every object's +0x78 box and +0x86 against its
    // template's model's `bound_size` on the port, those that differ.
    if std::env::var_os("OBJBOUNDS").is_some() {
        let (mut same, mut off) = (0, 0);
        for o in objects(&mut p) {
            let t = p.read_i16(o + 0x96) as u16 as usize;
            let b: Vec<i16> = (0..7).map(|i| p.read_i16(o + 0x78 + 2 * i)).collect();
            let size = p.read_i16(o + 0x86);
            // (Its placement's set-up size, found by its place.)
            let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
            let placed = g.set.placements.iter().position(|q| q.pos == at && q.kind == 1 && q.template as usize == t).and_then(|k| g.set.sizes.get(k).copied()).filter(|&s| s > 0);
            let r = placed.map(|s| s as i16).or_else(|| g.set.templates.get(t).and_then(|tp| g.set.models.get(tp.shape as u32)).map(|m| rugrats::world::boxhit::bound_size(&m.boxes) as i16));
            let cube = r.is_some_and(|r| b[0] == -r && b[1] == -r && b[2] == -r && b[4] == r && b[5] == r && b[6] == r && size == r);
            if std::env::var("OBJT").ok().and_then(|v| v.parse::<usize>().ok()) == Some(t) {
                let m = g.set.templates.get(t).and_then(|tp| g.set.models.get(tp.shape as u32));
                let l = p.read_u32(o + 0x88);
                let (n, bp) = if l != 0 { (p.read_i16(l + 0xa) as u16 as u32, p.read_u32(l + 0xc)) } else { (0, 0) };
                let theirs: Vec<Vec<i16>> = (0..n.min(8)).map(|j| (0..8).map(|i| p.read_i16(bp + 16 * j + 2 * i)).collect()).collect();
                println!("  {o:#x} +0x88 {l:#x} boxes {theirs:?}");
                let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
                let place = g.set.placements.iter().position(|q| q.pos == at);
                if let Some(k) = place {
                    let q = g.set.placements[k];
                    let shape = g.set.templates.get(q.template as usize).map(|tp| tp.shape);
                    println!("  {o:#x} port placement {k} template {} shape {shape:?} size {:?}", q.template, g.set.sizes.get(k));
                }
                println!("  {o:#x} flags {:#x} +0x44 {:#x} +0x48 {:#x} +0x50 {:#x} +0x3b {} | placement {place:?} {:?}", p.read_u32(o), p.read_u32(o + 0x44), p.read_u32(o + 0x48), p.read_u32(o + 0x50), p.read_u8(o + 0x3b), place.map(|k| (g.set.placements[k].kind, g.set.placements[k].id, g.set.placements[k].rotation)));
                println!("  {o:#x} template {t} shape {:?} at {:?} angles {:?} +0x78 {b:?} +0x86 {size} | model radius {:?} boxes {:?}", g.set.templates.get(t).map(|tp| tp.shape), [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)), [0, 2, 4].map(|i| p.read_i16(o + 0x24 + i)), m.map(|m| m.radius), m.map(|m| m.boxes.clone()));
            }
            if cube {
                same += 1;
            } else {
                off += 1;
                println!("template {t} class {} +0x78 {b:?} +0x86 {size} | radius {r:?}", p.read_u8(o + 0xa6));
            }
        }
        println!("{same} cubes of bound_size, {off} not");
        return Ok(());
    }
    // BOUNDS=1: each thing's +0x78 box and +0x86 on the original, its
    // model's radius and boxes on the port.
    if std::env::var_os("BOUNDS").is_some() {
        for k in 0..g.things.len() {
            let at = ipos(g.things[k].pos);
            let o = nearest(&mut p, at);
            let b: Vec<i16> = (0..7).map(|i| p.read_i16(o + 0x78 + 2 * i)).collect();
            let m = g.set.models.get(g.things[k].shape);
            println!("thing {k} template {} obj {o:#x} at {:?} +0x78 {b:?} +0x86 {} | radius {:?} boxes {:?}", g.things[k].template, [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)), p.read_i16(o + 0x86), m.map(|m| m.radius), m.map(|m| m.boxes.clone()));
        }
        return Ok(());
    }
    let ob = nearest(&mut p, pb);
    // HOOKS=1: the original's stacking judged (0x8006c7a0) and its answer.
    if std::env::var_os("HOOKS").is_some() {
        p.hle.m.hook(0x8006_c7a0, |c| {
            println!("     STACK {:#x} {:#x} ra {:#x}", c.arg32(0), c.arg32(1), c.cpu.reg(31));
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8004_e018, |c| {
            println!("     STACKED 1");
            let _ = c;
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8004_e030, |c| {
            println!("     STACKED 2");
            let _ = c;
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8006_08b0, |c| {
            let sp = c.cpu.reg(29) as u32;
            let v0 = c.cpu.reg(2) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let from = [w(sp + 24), w(sp + 28), w(sp + 32)];
            let ahead = [w(sp + 40), w(sp + 44), w(sp + 48)];
            let hit = [w(sp + 56), w(sp + 60), w(sp + 64)];
            let what = if v0 > 1 { c.bus.read_u16(v0 + 0x96).unwrap_or(0) as i32 } else { -1 };
            println!("     PUTWAY {v0:#x} template {what} from {from:?} ahead {ahead:?} hit {hit:?}");
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8004_df94, |c| {
            let o = c.arg32(0);
            let fl = c.bus.read_u32(o + 0x90).unwrap_or(0);
            let done = c.bus.read_u8(o + 0x6b).unwrap_or(0);
            let flags = c.bus.read_u32(o).unwrap_or(0);
            let met = if fl != 0 { c.bus.read_u32(fl + 0x98).unwrap_or(0) } else { 0 };
            if std::env::var_os("THINKS").is_some() || done == 255 {
                println!("     THINK {o:#x} flight {fl:#x} +0x6b {done} flags {flags:#x} met {met:#x}");
            }
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8004_dfd0, |c| {
            println!("     DONE {:#x} flags {:#x}", c.cpu.reg(17), c.cpu.reg(3));
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8004_dff0, |c| {
            println!("     MET {:#x}", c.cpu.reg(2));
            rrt::emu::HookResult::Continue
        });
    }
    let mut pads = Pads::default();
    put(&mut g, &mut p, pa, 0, 700);
    run(&mut g, &mut p, &mut pads, &[(0, 6), (1 << input::CIRCLE, 1), (0, 40)], a, 0)?;
    let oa = p.read_u32(PLAYER + 0x1f0);
    println!("carried: port {:?} orig {oa:#x}", g.carried());
    put(&mut g, &mut p, pb, heading, away);
    // WALK=n: Up held n frames first.
    let walk = std::env::var("WALK").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    run(&mut g, &mut p, &mut pads, &[(0, 4), (1 << input::UP, walk), (0, 20), (1 << input::CIRCLE, 1), (0, 90)], a, oa)?;
    let me = p.player();
    println!("player: port {:?} heading {} carried {:?} | orig {:?} heading {} carried {:#x}", g.player.pos, g.player.heading, g.carried(), me.pos, p.read_i16(PLAYER + 0x24), p.read_u32(PLAYER + 0x1f0));
    let (oa_at, ob_at) = ([0, 4, 8].map(|i| p.read_i32(oa + 0x18 + i)), [0, 4, 8].map(|i| p.read_i32(ob + 0x18 + i)));
    println!("A: port at {:?} on {:?} | orig {oa:#x} at {oa_at:?} on {:#x}", ipos(g.things[a].pos), g.things[a].on, p.read_u32(oa + 0x70));
    let (ra, rb) = (p.read_u32(oa + 0x74), p.read_u32(ob + 0x74));
    let (la, lb) = (ring(&mut p, ra), ring(&mut p, rb));
    println!("B: port at {pb:?} load {:?} | orig {ob:#x} at {ob_at:?} load {lb:x?}", g.load(Held::Thing(b)));
    println!("A load: port {:?} | orig {la:x?}", g.load(Held::Thing(a)));
    run(&mut g, &mut p, &mut pads, &[(1 << input::CIRCLE, 1), (0, 40)], a, 0)?;
    println!("GRAB: port {:?} orig {:#x}", g.carried(), p.read_u32(PLAYER + 0x1f0));
    Ok(())
}
