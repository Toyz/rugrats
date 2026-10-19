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

/// An object's animation as the original keeps it, for
/// `Animator::load_record`: the tracks, the speed (+0xe4), the states after
/// (+0xe6, +0xe7), the fidget (+0xf0, +0xf2) and each channel's kept
/// quaternion (its part's +4).
#[allow(clippy::type_complexity)]
fn record(p: &mut Play, o: u32) -> ([[i16; 6]; 4], i32, (u8, u8), (i16, i32), Vec<Option<[i32; 4]>>) {
    let t = tracks(p, o);
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
    (t, speed, after, fidget, pose)
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
    // FAST=1: a console fast enough that the original never takes a third
    // blank over a frame (the oracle's budget, `Hle::frame_steps`, is
    // instructions): its time goes 10 every frame, as the port's.
    if std::env::var_os("FAST").is_some() {
        p.hle.frame_steps = std::env::var("FAST").ok().and_then(|s| s.parse().ok()).unwrap_or(1_200_000);
    }
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
        // (By place only an object not paired yet and near: a character
        // the original has not made pairs with none.)
        let near: Vec<(u32, [i32; 3])> = bodies.iter().copied().filter(|&(o, _)| !pairs.iter().any(|&(_, q)| q == o)).collect();
        let pool = if by_id.is_empty() { &near } else { &by_id };
        let best = pool.iter().map(|&(o, at)| ((0..3).map(|i| (at[i] - n.pos[i]).abs() as i64).sum::<i64>(), o)).min();
        match best {
            Some((d, o)) if !by_id.is_empty() || d <= 4000 => {
                println!("npc {k} id {} at {:?} <-> {o:#x} (off {d})", n.id, n.pos);
                pairs.push((k, o));
            }
            _ => println!("npc {k} id {} at {:?}: no object of the original's", n.id, n.pos),
        }
    }
    // The port's animators given the original's.
    for &(k, o) in &pairs {
        let (t, speed, after, fidget, pose) = record(&mut p, o);
        let n = &mut g.npcs[k];
        n.animator.load_record(t, speed, after, fidget, pose, &g.anims);
        n.animator.mark_asked(n.hold.unwrap_or(0));
        println!("    npc {k}: tracks {t:?} speed {speed} after {after:?} fidget {fidget:?} hold {:?} mode {:?}", n.hold, n.mode);
        if std::env::var_os("COLUMNS").is_some() {
            // The original's character (+0x15c) and its state 0 column for
            // each track (+0xec's entry into +0xe8, then D_800c4594).
            let ec = p.read_u32(o + 0xec);
            let e8 = p.read_u32(o + 0xe8);
            let entry = p.read_i16(ec) as u32;
            let cols: Vec<i16> = (0..4).map(|j| p.read_i16(e8 + entry * 8 + 2 * j)).collect();
            let table = p.read_u32(0x800c_4594);
            let spans: Vec<(i16, i16)> = cols.iter().map(|&c| (p.read_i16(table + 4 * c as u32), p.read_i16(table + 4 * c as u32 + 2))).collect();
            println!("    COLUMNS char {} entry {entry} cols {cols:?} spans {spans:?}", p.read_i16(o + 0x15c));
        }
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
        // The character probe's kind (0x80064444 back in 0x80064838):
        // kind, limit, the record's delta and normal (+0xf4 + 0x50, +0x20).
        p.hle.m.hook(0x8006_49a0, move |c| {
            if c.cpu.reg(19) as u32 == o {
                let mut r = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
                let rec = o + 0xf4;
                println!(
                    "     PROBE kind {} limit {} delta {} normal {:?} way {:?} touched {:#x} template {}",
                    c.cpu.reg(2) as i32,
                    c.cpu.reg(17) as i32,
                    r(rec + 0x50),
                    [r(rec + 0x20), r(rec + 0x24), r(rec + 0x28)],
                    [r(o + 0x178), r(o + 0x17c), r(o + 0x180)],
                    r(o + 0xf4) as u32,
                    { let t = r(o + 0xf4) as u32; if t != 0 { r(t + 0x94) >> 16 } else { 0 } }
                );
            }
            rrt::emu::HookResult::Continue
        });
        p.hle.m.hook(0x8006_4970, move |c| {
            if c.cpu.reg(19) as u32 == o {
                let sp = c.cpu.reg(29) as u32;
                let mut r = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
                println!("     ZZPOINT reach {} off {:?} point {:?}", c.cpu.reg(20) as i32, [r(sp + 16), r(sp + 20), r(sp + 24)], [r(sp + 32), r(sp + 36), r(sp + 40)]);
            }
            rrt::emu::HookResult::Continue
        });
        // After it, each of the body's last calls, with the place.
        for a in [0x8005_bc80u32, 0x8006_27a0, 0x8006_23a0, 0x8008_3634, 0x8006_af4c, 0x8006_31a4, 0x8004_a318] {
            p.hle.m.hook(a, move |c| {
                if c.cpu.reg(4) as u32 == o {
                    let at = [0, 4, 8].map(|i| c.bus.read_u32(o + 0x18 + i).unwrap_or(0) as i32);
                    println!("     TAIL {a:#x} at {at:?}");
                }
                rrt::emu::HookResult::Continue
            });
        }
        // The body's end (0x8005cb48): the step's sum into the place.
        p.hle.m.hook(0x8005_cb48, move |c| {
            if c.cpu.reg(4) as u32 == o {
                let mut r16 = |a: u32| c.bus.read_u16(a).unwrap_or(0) as i16;
                let d = [r16(o + 0xd8), r16(o + 0xda), r16(o + 0xdc)];
                let f = c.bus.read_u32(o + 0x188).unwrap_or(0);
                let y = c.bus.read_u32(o + 0x1c).unwrap_or(0) as i32;
                println!("     APPLY d {d:?} 188 {f:#x} y {y} ra {:#x}", c.cpu.reg(31));
            }
            rrt::emu::HookResult::Continue
        });
        // The root's step (0x80055904: the track, the object, the time
        // before; +0xde the first frame's gap).
        p.hle.m.hook(0x8005_5904, move |c| {
            if c.cpu.reg(5) as u32 == o {
                let t = c.cpu.reg(4) as u32;
                let mut r16 = |a: u32| c.bus.read_u16(a).unwrap_or(0) as i16;
                let tr = [r16(t), r16(t + 2), r16(t + 4), r16(t + 6), r16(t + 10)];
                let de = r16(o + 0xde);
                let y = c.bus.read_u32(o + 0x1c).unwrap_or(0) as i32;
                let floor = c.bus.read_u32(o + 0x148).unwrap_or(0) as i32;
                let h = c.bus.read_u32(o + 0x154).unwrap_or(0);
                let feet = c.bus.read_u16(h + 0x4c).unwrap_or(0) as i16;
                let flags = c.bus.read_u32(o).unwrap_or(0);
                println!("     ROOT track {tr:?} before {} de {de} y {y} floor {floor} feet {feet} flags {flags:#x}", c.cpu.reg(6) as i16);
            }
            rrt::emu::HookResult::Continue
        });
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
    // WATCH=addr: each call of that function in the original, as it comes.
    if let Some(a) = std::env::var("WATCH").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
        p.hle.m.hook(a, move |c| {
            println!("     WATCH {a:#x} a0 {:#x} ra {:#x}", c.cpu.reg(4) as u32, c.cpu.reg(31) as u32);
            rrt::emu::HookResult::Continue
        });
    }
    let pads = Pads::default();
    if std::env::var_os("TEMPLATES").is_some() {
        let base = p.read_u32(0x800c_4300);
        let mut n = 0;
        for (k, t) in g.set.templates.iter().enumerate() {
            let orig: Vec<u8> = (0..56).map(|i| p.read_u8(base + 56 * k as u32 + i)).collect();
            let mine: Vec<u8> = t.raw.to_vec();
            if orig[..mine.len().min(56)] != mine[..mine.len().min(56)] {
                n += 1;
                let at: Vec<usize> = (0..mine.len().min(56)).filter(|&i| orig[i] != mine[i]).collect();
                if n <= 20 {
                    println!("template {k}: bytes {at:?} orig {:?} port {:?}", at.iter().map(|&i| orig[i]).collect::<Vec<_>>(), at.iter().map(|&i| mine[i]).collect::<Vec<_>>());
                }
            }
        }
        println!("templates differ {n} of {} (table {base:#x})", g.set.templates.len());
    }
    if std::env::var_os("FRAMES").is_some() {
        let table = p.read_u32(0x800c_4598);
        let mut diffs = 0;
        for k in 0..g.anims.frames.len() as u32 {
            let w: Vec<u16> = (0..2).map(|i| p.read_i16(table + 8 * k + 2 * i) as u16).collect();
            let f = &g.anims.frames[k as usize];
            if w[0] != f.events as u16 || w[1] != f.duration as u16 {
                diffs += 1;
                if diffs <= 30 {
                    println!("frame {k}: orig {w:?} port {} {}", f.events, f.duration);
                }
            }
        }
        println!("frames {} differ {diffs}", g.anims.frames.len());
    }
    // SOUNDS=1: the sounds each side keys on (0x800a99a8: program, tone and
    // the note at sp + 0x10), the port's effects through the original's
    // table (0x800b84fc, 3 bytes a sound); one a frame later in the
    // original still counts (the trigger pass runs after its frame's end).
    let keyed: Rc<std::cell::RefCell<Vec<[u8; 3]>>> = Rc::default();
    let sounds = std::env::var_os("SOUNDS").is_some();
    if sounds {
        let k = keyed.clone();
        p.hle.m.hook(0x800a_99a8, move |c| {
            let note = c.bus.read_u8(c.cpu.reg(29) as u32 + 0x10).unwrap_or(0);
            if std::env::var_os("SOUNDRA").is_some() {
                println!("     KEYON ra {:#x} args {:?} note {note}", c.cpu.reg(31), [4, 5, 6, 7].map(|r| c.cpu.reg(r) as i32));
            }
            k.borrow_mut().push([c.cpu.reg(6) as u8, c.cpu.reg(7) as u8, note]);
            rrt::emu::HookResult::Continue
        });
    }
    // HOOKRA=addr: each call of addr, its caller and its a0's template
    // (+0x96) and place.
    if let Some(at) = std::env::var("HOOKRA").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
        p.hle.m.hook(at, move |c| {
            let o = c.cpu.reg(4) as u32;
            let t = c.bus.read_u16(o + 0x96).unwrap_or(0);
            let pos = [0, 4, 8].map(|i| c.bus.read_u32(o + 0x18 + i).unwrap_or(0) as i32);
            println!("     HOOK {at:#x} ra {:#x} a0 {o:#x} template {t} at {pos:?} a1 {:#x}", c.cpu.reg(31), c.cpu.reg(5));
            rrt::emu::HookResult::Continue
        });
    }
    if sounds && std::env::var_os("SOUNDRA").is_some() {
        for (at, name) in [(0x8006_f26cu32, "PLAY"), (0x8006_f644, "PLAYAT")] {
            p.hle.m.hook(at, move |c| {
                println!("     {name} {} ra {:#x}", c.cpu.reg(4) as i32, c.cpu.reg(31));
                rrt::emu::HookResult::Continue
            });
        }
    }
    // FLIGHTS=1: each launch (0x8008864c's record) on both sides, compared
    // frame by frame as the sounds are: flags, place, radius, life,
    // direction, speed, sprite (its TEXPOS entry), half size (neither for
    // the model kinds 2 and 4, whose callers leave them unset), damage, and
    // the sound (+0x64) and mark (+0x46, +0x68) where it meets something.
    type Rec = (u32, [i32; 3], i32, i32, [i32; 3], i32, i32, [i16; 2], i16, (u8, u16, u16));
    let model_kind = |flags: u32| matches!(flags & 15, 2 | 4);
    let flown: Rc<std::cell::RefCell<Vec<Rec>>> = Rc::default();
    let flights = std::env::var_os("FLIGHTS").is_some();
    if flights {
        let k = flown.clone();
        p.hle.m.hook(0x8008_864c, move |c| {
            let r = c.cpu.reg(4) as u32;
            let mut rd = |a: u32| c.bus.read_u32(r + a).unwrap_or(0) as i32;
            let flags = rd(0) as u32;
            let at = [rd(4), rd(8), rd(0xc)];
            let radius = rd(0x10);
            let life = rd(0x28);
            let dir = [rd(0x2c), rd(0x30), rd(0x34)];
            let speed = rd(0x38);
            let sprite_at = rd(0x58) as u32;
            let size_w = rd(0x5c);
            let damage = (rd(0x68) >> 16) as i16;
            let met = (rd(0x64) as u8, (rd(0x44) >> 16) as u16, rd(0x68) as u16);
            let texpos = c.bus.read_u32(0x800c_4484).unwrap_or(0);
            let sprite = if sprite_at >= texpos && texpos != 0 && !model_kind(flags) { ((sprite_at - texpos) / 12) as i32 } else { -1 };
            let size = if model_kind(flags) { [0; 2] } else { [size_w as i16, (size_w >> 16) as i16] };
            k.borrow_mut().push((flags, at, radius, life, dir, speed, sprite, size, damage, met));
            rrt::emu::HookResult::Continue
        });
    }
    let mut flown_owed: Vec<(usize, Rec)> = Vec::new();
    let (mut flight_off, mut flight_same) = (0, 0);
    let table = disc.exe_bytes(0x800b_84fc, 3 * 180)?;
    let mut owed: Vec<(usize, [u8; 3])> = Vec::new();
    let (mut sound_off, mut sound_same) = (0, 0);
    let mut off = 0;
    // By NPC: frames off, and whether each side's tracks ever changed.
    let mut each: Vec<(usize, bool, bool)> = vec![(0, false, false); pairs.len()];
    let start_orig: Vec<[[i16; 6]; 4]> = pairs.iter().map(|&(_, o)| tracks(&mut p, o)).collect();
    let start_port: Vec<[[i16; 6]; 4]> = pairs.iter().map(|&(k, _)| g.npcs[k].animator.record()).collect();
    // (A frame the original took three VSyncs over, dt 15 (0x800c3f1c),
    // the port, at dt 10, cannot follow: its time (0x800c3e98) stands
    // still one frame and goes 15 the next. Those frames are not counted,
    // and each pair is given the original's state again after them;
    // FAST keeps the original from taking them.)
    let mut lagged = false;
    let mut clock = p.read_i32(0x800c_3e98);
    for f in 0..frames {
        if sync_at.is_some_and(|s| s == f || s < f && lagged) {
            // (The player's health too: a bite may end it.)
            g.player.health = p.read_i16(0x800e_2b48 + 0x1a4) as i32;
            // (And the game's rand, 0x800c3f94: the footsteps' notes and
            // the characters' choices draw on it.)
            g.rand.state = p.read_u32(0x800c_3f94);
            // (And each door where the original's is in its cycle: the
            // object of its template placed where it is.)
            if sync_at == Some(f) {
                for d in &mut g.doors {
                    let start = d.start();
                    let Some(&o) = objs.iter().find(|&&o| p.read_i16(o + 0x96) as u16 == d.template && [0, 4, 8].map(|i| p.read_i32(o + 0xc0 + i)) == start) else { continue };
                    let offset = [0, 2, 4].map(|i| p.read_i16(o + 0xa8 + i) as i32);
                    d.sync(offset, p.read_u32(o + 0x3c) & 2 != 0, p.read_u32(o + 0xcc) != 0, p.read_i16(o + 0xd2) as i32);
                }
            }
            // (And the class 10 objects' clocks, +0x64 against the VSync
            // count, and whether each went last think, +0x98's +0x00; the
            // direction seed, 0x800c445c, the callbacks' seeds and the shared
            // sound count, 0x800c4ba0.)
            if sync_at == Some(f) {
                let now = p.read_u32(0x800b_cb60);
                g.vcount = now;
                // (The update list's order, 0x800c43b8 from its head, +0x10 the
                // object: the newest scheduled first.)
                let mut listed = Vec::new();
                let mut node = p.read_u32(0x800c_43b8);
                while node != 0 && listed.len() < 1000 {
                    listed.push(p.read_u32(node + 0x10));
                    node = p.read_u32(node);
                }
                let placements = g.set.placements.clone();
                let mut scheduled = Vec::new();
                for e in &mut g.emitters {
                    let place = placements[e.placement];
                    let Some(&o) = objs.iter().find(|&&o| p.read_i16(o + 0x96) as u16 as u32 == place.template && [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)) == place.pos) else { continue };
                    e.last = p.read_u32(o + 0x64);
                    let rec = p.read_u32(o + 0x98);
                    e.gone = p.read_i16(rec) != 0;
                    scheduled.push((e.placement, p.read_u32(o + 0x9c) != 0));
                    e.scheduled = listed.iter().position(|&l| l == o).map_or(0, |i| (listed.len() - i) as u64);
                }
                for (k, on) in scheduled {
                    g.trigger_set(rugrats::world::game::Trigger::Placement(k), on);
                }
                g.flights.seeds.direction = p.read_u32(0x800c_445c);
                for (k, at) in [0x800c_41a8u32, 0x800c_41ac, 0x800c_41b0, 0x800c_41b4, 0x800c_41bc, 0x800c_41c0, 0x800c_41c4, 0x800c_41c8, 0x800c_41d8].into_iter().enumerate() {
                    g.flights.seeds.emitters[k] = p.read_u32(at);
                }
                g.shelf_sound = p.read_i32(0x800c_4ba0);
            }
            // (And the player's place: the characters aim at it.)
            g.player.pos = [0, 4, 8].map(|i| p.read_i32(0x800e_2b48 + 0x18 + i));
            // (And the trigger pass's place in its list, 0x800c4c20: the
            // entry after the node it stopped at.)
            if sync_at == Some(f) {
                let cursor = p.read_u32(0x800c_4c20);
                let mut node = p.read_u32(0x800c_4c24);
                let mut i = 0;
                while node != 0 && node != cursor && i < 400 {
                    node = p.read_u32(node + 4);
                    i += 1;
                }
                g.trigger_next = if node == cursor && node != 0 { i + 1 } else { 0 };
            }
            for &(k, o) in &pairs {
                let t = tracks(&mut p, o);
                let speed = p.read_i16(o + 0xe4) as i32;
                let after = (p.read_u8(o + 0xe6), p.read_u8(o + 0xe7));
                let fidget = (p.read_i16(o + 0xf0), p.read_i16(o + 0xf2) as i32);
                let (cur, prev) = (p.read_u8(o + 0x695), p.read_u8(o + 0x694));
                let n = &mut g.npcs[k];
                let own_y = n.pos[1];
                n.pos = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
                n.heading = p.read_i16(o + 0x24) as i32 & 4095;
                // (Scheduled as the original's is: on the update list,
                // +0x9c, which 0x80049e18 and a level's start put it on.)
                n.scheduled = p.read_u32(o + 0x9c) != 0;
                // (A bite's wait, +0x69c, and the events it waits on, +0xe0:
                // kept while it waits.)
                n.wait_left = p.read_i32(o + 0x69c);
                // (Idle, facing the player and going home: +0x688 bits 64
                // and 2.)
                let f688 = p.read_u32(o + 0x688);
                n.faces = f688 & 64 != 0;
                n.idle_home = f688 & 2 != 0;
                n.ignores_player = p.read_u32(o + 0x1f4) == 0x800e_2b48;
                n.animator.load_record(t, speed, after, fidget, vec![None; 32], &g.anims);
                n.animator.events = p.read_u32(o + 0xe0);
                // (Its after state, +0xe6, playing is standing: the idle asked.)
                if let Some(a) = rugrats::actors::animator::logical_of(t[0][0]).filter(|&a| a != after.0 as i16) {
                    n.animator.mark_asked(a);
                }
                // (A state a level holds was asked once; what it completed
                // into plays on.)
                if let Some(h) = n.hold {
                    n.animator.mark_asked(h);
                }
                // (The original's height has the root's lift in it; the
                // port's place is the floor (+0x148) and the feet, its lift
                // added where drawn.)
                // (One never stepped keeps its own height there, and a thing that
                // is no body has none.)
                let floor = p.read_i32(o + 0x148);
                let y = n.pos[1];
                if n.unsettled {
                    // (Never settled, 0x800452f8: its place keeps its own
                    // height, the lift its hover.)
                    n.pos[1] = own_y;
                } else if floor != n.pos[1] && (floor + n.feet() - n.pos[1]).abs() < 1000 {
                    n.pos[1] = floor + n.feet();
                }
                // Its lift the original's height less its place, going on by
                // its keys from there (0x80055904).
                n.root.y = y - n.pos[1];
                let lo = n.root_scale();
                n.root.set_up(&n.animator, &g.anims, lo, p.read_i16(o + 0xde) as i32);
                // (Its body where its place is now.)
                n.sync_body(p.read_u8(o + 0x18c), p.read_i32(o + 0x184));
                if let rugrats::actors::npc::Mode::Posts { .. } = n.mode {
                    n.mode = rugrats::actors::npc::Mode::Posts { prev, cur };
                }
                // A follower's place in its bite (+0x699: 5, 1, 0, else 4).
                if let rugrats::actors::npc::Mode::Follow(_) = n.mode {
                    use rugrats::actors::npc::Bite;
                    // (Where it aims, +0x67c; at that place when what it
                    // follows, +0x678, is no player.)
                    n.goal = [0, 4, 8].map(|i| p.read_i32(o + 0x67c + i));
                    if p.read_u32(o + 0x678) != 0x800e_2b48 {
                        n.mode = rugrats::actors::npc::Mode::Follow(rugrats::actors::npc::Target::Point(n.goal));
                    }
                    n.bite = match p.read_u8(o + 0x699) {
                        5 => Bite::Strike,
                        1 => Bite::Recover,
                        0 => Bite::Resume,
                        // Mode 8 in state 28: the lunge.
                        8 if p.read_u8(o + 0x18c) == 28 => Bite::Lunge,
                        _ => Bite::Chase,
                    };
                    // (May lunge, +0x688 bit 128, and its kept speed.)
                    n.lunges = p.read_u32(o + 0x688) & 128 != 0;
                    n.lunge_speed = p.read_i16(o + 0x176) as i32;
                    // (The heading its stick last asked, +0x2c.)
                    n.aim = p.read_i16(o + 0x2c) as i32 & 4095;
                }
            }
        }
        // PRESS=f: Cross held on the original's pad that frame (a card
        // waiting for a press; the port has none up).
        let press = std::env::var("PRESS").ok().and_then(|v| v.parse::<usize>().ok()) == Some(f);
        let pad = if press { 1 << rugrats::input::CROSS } else { 0 };
        // PLAYERSYNC=1: the player where the original's is this frame (it
        // moves first, 0x80018c38; the characters aim at it): the
        // original's frame run first, its player given the port's.
        if std::env::var_os("PLAYERSYNC").is_some() {
            // (And what it carries where the original's was as its frame
            // began: the hand puts it there after the objects' update.)
            let held = p.read_u32(0x800e_2b48 + 0x1f0);
            let held_at = [0, 4, 8].map(|i| p.read_i32(held + 0x18 + i));
            // PLAYERANIM=1: and its animation as the original's frame
            // began, the port's step taking it on (its hand where the
            // original's is).
            if std::env::var_os("PLAYERANIM").is_some() {
                let (t, speed, after, fidget, pose) = record(&mut p, 0x800e_2b48);
                g.player.animator.load_record(t, speed, after, fidget, pose, &g.anims);
            }
            // PLAYERBEFORE=1: its place as the original's frame began, the
            // port's own step moving it (a knocked player pushed back).
            let before = ([0, 4, 8].map(|i| p.read_i32(0x800e_2b48 + 0x18 + i)), p.read_i16(0x800e_2b48 + 0x24) as i32 & 4095);
            p.frame(pad)?;
            let after = ([0, 4, 8].map(|i| p.read_i32(0x800e_2b48 + 0x18 + i)), p.read_i16(0x800e_2b48 + 0x24) as i32 & 4095);
            (g.player.pos, g.player.heading) = if std::env::var_os("PLAYERBEFORE").is_some() { before } else { after };
            if held != 0
                && let Some(k) = g.carried()
            {
                let at = held_at;
                g.things[k].pos = rrt::glam::Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
            }
            g.frame(&pads);
        } else {
            g.frame(&pads);
            p.frame(pad)?;
        }
        // EMITDBG=1: each frame, the trigger cursor, the shared sound count
        // and the direction seed on both sides, and each class 10 object's
        // clock (+0x64, +0x98's +0x00) and whether it is scheduled.
        if std::env::var_os("EMITDBG").is_some() {
            let cursor = p.read_u32(0x800c_4c20);
            let mut node = p.read_u32(0x800c_4c24);
            let mut i = 0;
            while node != 0 && node != cursor && i < 400 {
                node = p.read_u32(node + 4);
                i += 1;
            }
            println!("     CURSOR {f} orig {i} port next {} count orig {} port {} dir orig {:#x} port {:#x}", g.trigger_next, p.read_i32(0x800c_4ba0), g.shelf_sound, p.read_u32(0x800c_445c), g.flights.seeds.direction);
            for e in &g.emitters {
                let place = g.set.placements[e.placement];
                let Some(&o) = objs.iter().find(|&&o| p.read_i16(o + 0x96) as u16 as u32 == place.template && [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)) == place.pos) else { continue };
                let rec = p.read_u32(o + 0x98);
                println!("     EMIT {f} k {} port {:?} last {} gone {} vc {} | orig sched {} last {} gone {} vc {}", e.placement, g.trigger_active(rugrats::world::game::Trigger::Placement(e.placement)), e.last, e.gone, g.vcount, p.read_u32(o + 0x9c) != 0, p.read_u32(o + 0x64), p.read_i16(rec), p.read_u32(0x800b_cb60));
            }
        }
        if sounds && std::env::var_os("SOUNDRA").is_some() {
            for e in &g.npc_events {
                if let rugrats::actors::npc::Event::Footstep { k, note, .. } = e {
                    println!("     PORT STEP npc {k} id {} char {} note {note} at {:?}", g.npcs[*k].id, g.npcs[*k].character, g.npcs[*k].pos);
                }
            }
        }
        if sounds {
            let mine: Vec<[u8; 3]> = g
                .effects
                .iter()
                .map(|e| match e.key {
                    Some(k) => k,
                    None => {
                        let t = &table[3 * e.sound as usize..3 * e.sound as usize + 3];
                        [t[0], t[1], t[2]]
                    }
                })
                .collect();
            let mut theirs = std::mem::take(&mut *keyed.borrow_mut());
            // The port's owed from the frame before, then this frame's.
            for (from, s) in std::mem::take(&mut owed).into_iter().chain(mine.into_iter().map(|s| (f, s))) {
                if let Some(i) = theirs.iter().position(|t| *t == s) {
                    theirs.remove(i);
                    sound_same += 1;
                } else if from == f {
                    owed.push((from, s));
                } else {
                    sound_off += 1;
                    println!("{f:3} SOUND port {s:?} (frame {from}) not in the original's");
                }
            }
            for t in theirs {
                sound_off += 1;
                println!("{f:3} SOUND original {t:?} not in the port's");
            }
        }
        // DOORS=t: template t's doors, the original's and the port's.
        if let Some(t) = std::env::var("DOORS").ok().and_then(|v| v.parse::<u16>().ok()) {
            for d in g.doors.iter().filter(|d| d.template == t) {
                let start = d.start();
                if let Some(&o) = objs.iter().find(|&&o| p.read_i16(o + 0x96) as u16 == t && [0, 4, 8].map(|i| p.read_i32(o + 0xc0 + i)) == start) {
                    println!(
                        "{f:3} DOOR orig offset {:?} bit2 {} cc {:#x} d2 {} be {:#x} speed {:?} ends {} {} | port {}",
                        [0, 2, 4].map(|i| p.read_i16(o + 0xa8 + i)),
                        p.read_u32(o + 0x3c) & 2,
                        p.read_u32(o + 0xcc),
                        p.read_i16(o + 0xd2),
                        p.read_i16(o + 0xbe),
                        [0, 2, 4].map(|i| p.read_i16(o + 0xb0 + i)),
                        p.read_i16(o + 0xae),
                        p.read_i16(o + 0xb6),
                        d.state()
                    );
                }
            }
        }
        if flights {
            let mine: Vec<Rec> = std::mem::take(&mut g.flights.launched)
                .into_iter()
                .map(|l| {
                    let model = model_kind(l.flags);
                    let met = (l.sound, l.mark.map_or(0, |m| m.sprite), l.mark.map_or(0, |m| m.size));
                    (l.flags, l.at, l.radius, l.life, l.dir, l.speed, l.sprite.filter(|_| !model).map_or(-1, |s| s as i32), if model { [0; 2] } else { l.size }, l.damage, met)
                })
                .collect();
            let mut theirs = std::mem::take(&mut *flown.borrow_mut());
            for (from, r) in std::mem::take(&mut flown_owed).into_iter().chain(mine.into_iter().map(|r| (f, r))) {
                if let Some(i) = theirs.iter().position(|t| *t == r) {
                    theirs.remove(i);
                    flight_same += 1;
                } else if from == f {
                    flown_owed.push((from, r));
                } else {
                    flight_off += 1;
                    println!("{f:3} FLIGHT port {r:?} (frame {from}) not the original's");
                }
            }
            for t in theirs {
                flight_off += 1;
                println!("{f:3} FLIGHT original {t:?} not the port's");
            }
        }
        // PLAYER=1: the player's base track beside the port's, each frame.
        // TOUCH=1 with MOVES: what the body met (+0xf4: its template and
        // place) and the player's place.
        if std::env::var_os("TOUCH").is_some()
            && let Some(m) = moves
            && let Some(&(_, o)) = pairs.iter().find(|(k, _)| *k == m)
        {
            let t = p.read_u32(o + 0xf4);
            let met = if t != 0 { (p.read_i16(t + 0x96) as u16, [0, 4, 8].map(|i| p.read_i32(t + 0x18 + i))) } else { (0, [0; 3]) };
            let target = p.read_u32(o + 0x678);
            println!(
                "{f:3} TOUCH {t:#x} {met:?} player {:?} carrying {:#x} goal {:?} target {target:#x} at {:?}",
                [0, 4, 8].map(|i| p.read_i32(0x800e_2b48 + 0x18 + i)),
                p.read_u32(0x800e_2b48 + 0x1f0),
                [0, 4, 8].map(|i| p.read_i32(o + 0x67c + i)),
                if target != 0 { [0, 4, 8].map(|i| p.read_i32(target + 0x18 + i)) } else { [0; 3] }
            );
        }
        if std::env::var_os("PLAYER").is_some() {
            println!("{f:3} PLAYER orig {:?} health {} | port {:?} health {} crying {}", tracks(&mut p, 0x800e_2b48)[0], p.read_i16(0x800e_2b48 + 0x1a4), g.player.animator.record()[0], g.player.health, g.crying());
        }
        // (The level left: no more to compare.)
        if let Some(e) = &g.exit {
            println!("{f:3} EXIT {e:?}");
            break;
        }
        // (A cine begun: the app takes it after the frame, and the
        // original's world waits on its ForceCry or cine.)
        if let Some(c) = g.cine {
            println!("{f:3} CINE {c}");
            break;
        }
        let dt = p.read_i32(0x800c_3e98) - clock;
        clock += dt;
        lagged = dt != 10;
        if lagged {
            println!("{f:3} LAG dt {dt}");
            continue;
        }
        // PARTS=1 with MOVES: the original's first parts' places (+0x158,
        // 32 bytes a part: the matrix, then the place at +0x14) beside the
        // port's posed joints.
        if std::env::var_os("PARTS").is_some()
            && let Some(m) = moves
            && let Some(&(k, o)) = pairs.iter().find(|(k, _)| *k == m)
        {
            let base = p.read_u32(o + 0x158);
            let theirs: Vec<[i32; 3]> = (0..4u32).map(|i| [0, 4, 8].map(|j| p.read_i32(base + 32 * i + 0x14 + j))).collect();
            let ours: Vec<[i32; 3]> = g.npcs[k].posed(&g.trig).iter().take(4).map(|q| q.joint).collect();
            println!("{f:3} PARTS orig {theirs:?} de {} floor {} | port {ours:?} root y {} pos y {} feet {} body floor {:?}", p.read_i16(o + 0xde), p.read_i32(o + 0x148), g.npcs[k].root.y, g.npcs[k].pos[1], g.npcs[k].feet(), g.npcs[k].body.as_ref().map(|b| b.floor));
        }
        if let Some(m) = moves
            && let Some(&(k, o)) = pairs.iter().find(|(k, _)| *k == m)
        {
            let n = &g.npcs[k];
            let at = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
            let way = [0, 4, 8].map(|i| p.read_i32(o + 0x178 + i));
            println!(
                "{f:3} MOVE card {:?} sched {}/{} trig {} orig {at:?} h {} to {} mode {} wait {} ev {:#x} track0 {:?} speed {} way {way:?} v {:?} state {} flags {:#x} held {} posts {}<{} player health {} | port {:?} root y {} h {} speed {} {:?} {:?} {:?}",
                g.card,
                (p.read_u32(o + 0x9c) != 0) as u8,
                g.npcs[k].scheduled,
                g.triggers.len(),
                p.read_i16(o + 0x24),
                p.read_i16(o + 0x2c),
                p.read_u8(o + 0x699),
                p.read_i32(o + 0x69c),
                p.read_u32(o + 0xe0),
                tracks(&mut p, o)[0],
                p.read_i32(o + 0x184),
                [0, 2, 4].map(|i| p.read_i16(o + 0x170 + i)),
                p.read_u8(o + 0x18c),
                p.read_u32(o + 0x188) as u64 | (p.read_u32(o + 0x688) as u64) << 32,
                p.read_i16(o + 0x1b0),
                p.read_u8(o + 0x695),
                p.read_u8(o + 0x694),
                p.read_i16(0x800e_2b48 + 0x1a4),
                n.pos,
                n.root.y,
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
            // (Its height: the place plus the root curve's, as level 13's
            // runner has it; YCHECK=1 counts it.)
            let high = std::env::var_os("YCHECK").is_none() || at[1] == n.pos[1] + n.root.y;
            let place = at[0] == n.pos[0] && at[2] == n.pos[2] && h == n.heading && high;
            let i = pairs.iter().position(|&(kk, _)| kk == k).unwrap();
            each[i].1 |= t != start_orig[i];
            each[i].2 |= mine != start_port[i];
            if !same || !place {
                each[i].0 += 1;
                off += 1;
                if off <= 40 {
                    println!("{f:3} npc {k}: port {:?} root y {} h {} {} | orig {at:?} h {h} {:?}", n.pos, n.root.y, n.heading, n.animator.describe(), t);
                }
            }
        }
    }
    for (i, &(k, _)) in pairs.iter().enumerate() {
        let (n_off, o_moved, p_moved) = each[i];
        println!("npc {k} id {} char {} trigger {} mode {:?}: {n_off} off, original {}, port {}", g.npcs[k].id, g.npcs[k].character, g.npcs[k].trigger, g.npcs[k].mode, if o_moved { "animated" } else { "still" }, if p_moved { "animated" } else { "still" });
    }
    println!("{off} frames off");
    if flights {
        println!("{flight_off} flights off, {flight_same} the same");
    }
    if sounds {
        println!("{sound_off} sounds off, {sound_same} the same");
    }
    Ok(())
}
