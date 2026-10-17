//! The other characters on the port and on the original, left alone: each
//! port NPC matched to the original's object at its place (the rooms'
//! lists, 0x800c4a8c), its animation given the original's, then the two
//! played on with no input and their places, headings and animation tracks
//! (+0xa8) printed side by side.
//!
//! `npc_check LEVEL [FRAMES]`
use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rrt::emu::{BusExt, Cpu};
use rugrats_hle::play::{Boot, Play};

/// The original's objects in the rooms' lists.
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

fn tracks(p: &mut Play, o: u32) -> [[i16; 6]; 4] {
    std::array::from_fn(|k| std::array::from_fn(|i| p.read_i16(o + 0xa8 + 12 * k as u32 + 2 * i as u32)))
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let level: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let frames: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(60);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let mut g = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    let character = Some(g.character as u32).filter(|&c| c != 1);
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, character, cinematics: std::env::var_os("CINES").is_some() })?;
    // SETTLE=n: the original run on n frames first (with CINES, past the
    // level's opening cinematic).
    for _ in 0..std::env::var("SETTLE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0) {
        p.frame(0)?;
    }
    let objs = objects(&mut p);
    // POSTS=1: each post, the original's record (D_800c4b9c, 28 bytes:
    // +4 place, +0x10 next, +0x14 flags) beside the port's.
    if std::env::var_os("POSTS").is_some() {
        let base = p.read_u32(0x800c_4b9c);
        for (i, post) in g.posts.iter().enumerate() {
            let r = base + 28 * i as u32;
            let at = [4u32, 8, 12].map(|o| p.read_i32(r + o));
            let next: Vec<u8> = (0..4).map(|o| p.read_u8(r + 0x10 + o)).collect();
            let same = at == post.pos;
            println!("post {}: {} orig {at:?} {next:?} {:#x} | port {:?} {:?} {:#x}", i + 1, if same { "same" } else { "DIFF" }, p.read_u32(r + 0x14), post.pos, post.next, post.flags);
        }
    }
    // DUMP=1: the original's bodies: id (+0x38), character (+0x15c),
    // place, heading, mode (+0x699), post (+0x695), speed, state; and the
    // player.
    if std::env::var_os("DUMP").is_some() {
        for &o in &objs {
            if p.read_u32(o + 0x154) != 0 {
                println!(
                    "body {o:#x} id {} char {} at {:?} h {} mode {} post {} speed {} state {}",
                    p.read_i16(o + 0x38) as u16,
                    p.read_i16(o + 0x15c),
                    [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)),
                    p.read_i16(o + 0x24),
                    p.read_u8(o + 0x699),
                    p.read_u8(o + 0x695),
                    p.read_i32(o + 0x184),
                    p.read_u8(o + 0x18c)
                );
            }
        }
        let me = p.player();
        println!("player at {:?} h {} | port player at {:?} h {}", me.pos, me.heading, g.player.pos, g.player.heading);
    }
    // Each NPC's object: the one of its template at its place, or nearest.
    let mut pairs = Vec::new();
    let mut bodies: Vec<(u32, [i32; 3])> = Vec::new();
    // (The player's own body, 0x800e2b48, is no character: it has the
    // id 1492 as Tommy has.)
    for &o in &objs {
        if p.read_u32(o + 0x154) != 0 && o != 0x800e_2b48 {
            let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
            bodies.push((o, at));
        }
    }
    for (k, n) in g.npcs.iter().enumerate() {
        // Its own id (+0x38) if a body has it, else the nearest.
        let by_id: Vec<(u32, [i32; 3])> = bodies.iter().copied().filter(|&(o, _)| p.read_i16(o + 0x38) as u16 == n.id && n.id != 0).collect();
        let pool = if by_id.is_empty() { &bodies } else { &by_id };
        let best = pool.iter().map(|&(o, at)| ((0..3).map(|i| (at[i] - n.pos[i]).abs() as i64).sum::<i64>(), o)).min();
        if let Some((d, o)) = best {
            println!("npc {k} id {} at {:?} <-> {o:#x} (off {d})", n.id, n.pos);
            pairs.push((k, o));
        }
    }
    // The port's animators given the original's.
    for &(k, o) in &pairs {
        let t = tracks(&mut p, o);
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
        n.animator.mark_asked(n.hold.unwrap_or(0));
        println!("    npc {k}: tracks {t:?} hold {:?} mode {:?}", n.hold, n.mode);
        n.pos = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
        n.heading = p.read_i16(o + 0x24) as i32 & 4095;
    }
    // PUT=x,y,z: both players put there first (heading 0).
    if let Ok(at) = std::env::var("PUT") {
        let v: Vec<i32> = at.split(',').filter_map(|s| s.parse().ok()).collect();
        if let [x, y, z] = v[..] {
            p.put([x, y, z], 0);
            g.player.pos = [x, y, z];
            g.player.heading = 0;
            g.player.target = 0;
        }
    }
    // CASTS=1: the original characters' look-ahead casts (0x80082b60):
    // who cast, from, to, and what 0x80090554 met.
    if std::env::var_os("CASTS").is_some() {
        p.hle.m.hook(0x8008_2bc4, |c| {
            let met = c.cpu.reg(2) as u32;
            let s0 = c.cpu.reg(16) as u32;
            let s1 = c.cpu.reg(17) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let seg = [w(s0), w(s0 + 4), w(s0 + 8), w(s0 + 16), w(s0 + 20), w(s0 + 24)];
            let tpl = if met > 1 { c.bus.read_u16(met + 0x96).unwrap_or(0) } else { 0 };
            println!("     CAST {s1:#x} seg {seg:?} met {met:#x} template {tpl}");
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8008_22dc, |c| {
            println!("     DETOUR {:#x} -> {} heading {}", c.cpu.reg(18) as u32, c.cpu.reg(2) as i32, c.bus.read_u16(c.cpu.reg(18) as u32 + 0x24).unwrap_or(0) as i16);
            rrt::emu::HookResult::Continue
        });
    }
    // MOVES=k: the original's object k's place, heading, speed (+0x184)
    // and way (+0x178) each frame beside the port's.
    let moves: Option<usize> = std::env::var("MOVES").ok().and_then(|s| s.parse().ok());
    // CALLS=1 with MOVES: the original's update (0x80084084) and body
    // (0x8005a068) calls for that object.
    if std::env::var_os("CALLS").is_some()
        && let Some(m) = moves
        && let Some(&(_, o)) = pairs.iter().find(|(k, _)| *k == m)
    {
        for a in [0x8008_4084u32, 0x8005_a068, 0x8005_6834, 0x8005_7214, 0x8005_72a8] {
            p.hle.m.hook(a, move |c| {
                if c.arg32(0) == o {
                    println!(
                        "     CALL {a:#x} a1 {} a2 {} ra {:#x}",
                        c.cpu.reg(5) as i32,
                        c.cpu.reg(6) as i32,
                        c.cpu.reg(31) as u32
                    );
                }
                rrt::emu::HookResult::Continue
            });
        }
    }
    // SYNCAT=f: each paired NPC given the original's place and heading
    // again before frame f.
    let sync_at: Option<usize> = std::env::var("SYNCAT").ok().and_then(|s| s.parse().ok());
    let pads = Pads::default();
    let mut off = 0;
    // By NPC: frames off, and whether each side's tracks ever changed.
    let mut each: Vec<(usize, bool, bool)> = vec![(0, false, false); pairs.len()];
    let start_orig: Vec<[[i16; 6]; 4]> = pairs.iter().map(|&(_, o)| tracks(&mut p, o)).collect();
    let start_port: Vec<[[i16; 6]; 4]> = pairs.iter().map(|&(k, _)| g.npcs[k].animator.record()).collect();
    // (A frame the original took three VSyncs over, dt 15 (0x800c3f1c),
    // the port, at dt 10, cannot follow: its time (0x800c3e98) stands
    // still one frame and goes 15 the next. Those frames are not counted,
    // and each pair is given the original's state again after them.)
    let mut lagged = false;
    let mut clock = p.read_i32(0x800c_3e98);
    for f in 0..frames {
        if sync_at.is_some_and(|s| s == f || s < f && lagged) {
            for &(k, o) in &pairs {
                let t = tracks(&mut p, o);
                let speed = p.read_i16(o + 0xe4) as i32;
                let after = (p.read_u8(o + 0xe6), p.read_u8(o + 0xe7));
                let fidget = (p.read_i16(o + 0xf0), p.read_i16(o + 0xf2) as i32);
                let (cur, prev) = (p.read_u8(o + 0x695), p.read_u8(o + 0x694));
                let n = &mut g.npcs[k];
                n.pos = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
                n.heading = p.read_i16(o + 0x24) as i32 & 4095;
                n.animator.load_record(t, speed, after, fidget, vec![None; 32], &g.anims);
                // (Its after state, +0xe6, playing is standing: the idle asked.)
                if let Some(a) = rugrats::actors::animator::logical_of(t[0][0]).filter(|&a| a != after.0 as i16) {
                    n.animator.mark_asked(a);
                }
                n.sync_body(p.read_u8(o + 0x18c), p.read_i32(o + 0x184));
                if let rugrats::actors::npc::Mode::Posts { .. } = n.mode {
                    n.mode = rugrats::actors::npc::Mode::Posts { prev, cur };
                }
            }
        }
        g.frame(&pads);
        p.frame(0)?;
        let dt = p.read_i32(0x800c_3e98) - clock;
        clock += dt;
        lagged = dt != 10;
        if lagged {
            println!("{f:3} LAG dt {dt}");
            continue;
        }
        if let Some(m) = moves
            && let Some(&(k, o)) = pairs.iter().find(|(k, _)| *k == m)
        {
            let n = &g.npcs[k];
            let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
            let way = [0, 4, 8].map(|i| p.read_i32(o + 0x178 + i));
            println!(
                "{f:3} MOVE orig {at:?} h {} to {} mode {} wait {} ev {:#x} track0 {:?} speed {} way {way:?} v {:?} state {} flags {:#x} held {} posts {}<{} | port {:?} h {} speed {} {:?} {:?} {:?}",
                p.read_i16(o + 0x24),
                p.read_i16(o + 0x2c),
                p.read_u8(o + 0x699),
                p.read_i32(o + 0x69c),
                p.read_u32(o + 0xe0),
                tracks(&mut p, o)[0],
                p.read_i32(o + 0x184),
                [0, 2, 4].map(|i| p.read_i16(o + 0x170 + i)),
                p.read_u8(o + 0x18c),
                p.read_u32(o + 0x188),
                p.read_i16(o + 0x1b0),
                p.read_u8(o + 0x695),
                p.read_u8(o + 0x694),
                n.pos,
                n.heading,
                n.speed,
                n.bite,
                n.animator.record()[0],
                n.mode
            );
        }
        for &(k, o) in &pairs {
            let n = &g.npcs[k];
            let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
            let h = p.read_i16(o + 0x24) as i32 & 4095;
            let t = tracks(&mut p, o);
            let mine = n.animator.record();
            let same = t.iter().zip(&mine).all(|(a, b)| a[..4] == b[..4] && (a[5] & 0xff) == (b[5] & 0xff));
            let place = at[0] == n.pos[0] && at[2] == n.pos[2] && h == n.heading;
            let i = pairs.iter().position(|&(kk, _)| kk == k).unwrap();
            each[i].1 |= t != start_orig[i];
            each[i].2 |= mine != start_port[i];
            if !same || !place {
                each[i].0 += 1;
                off += 1;
                if off <= 40 {
                    println!("{f:3} npc {k}: port {:?} h {} {} | orig {at:?} h {h} {:?}", n.pos, n.heading, n.animator.describe(), t);
                }
            }
        }
    }
    for (i, &(k, _)) in pairs.iter().enumerate() {
        let (n_off, o_moved, p_moved) = each[i];
        println!("npc {k} id {} char {} trigger {} mode {:?}: {n_off} off, original {}, port {}", g.npcs[k].id, g.npcs[k].character, g.npcs[k].trigger, g.npcs[k].mode, if o_moved { "animated" } else { "still" }, if p_moved { "animated" } else { "still" });
    }
    println!("{off} frames off");
    Ok(())
}
