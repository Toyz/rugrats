//! The level's objects on the original against the port's set: what the
//! object set-up (0x8004691c) changes as it makes them. Lists each object in
//! the rooms' lists (0x800c4a8c) with no placement of its template at its
//! place (moved or copied), the templates whose record differs from the
//! file's (the table at 0x800c4300), and the models whose boxes differ
//! (+0x88: count +0x0a, 16-byte boxes at +0x0c).
//!
//! `fixup_check LEVEL`
use std::collections::BTreeSet;
use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, Play};

fn main() -> Result<(), String> {
    let level: u8 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let g = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    let character = Some(g.character as u32).filter(|&c| c != 1);
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, character, cinematics: false })?;
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    let mut objs = Vec::new();
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                objs.push(p.read_u32(node + 8));
                node = p.read_u32(node + 4);
            }
        }
    }
    let set = &g.set;
    // Objects with no placement of their template at their place.
    let mut models = BTreeSet::new();
    for &o in &objs {
        let template = p.read_i16(o + 0x96) as u16;
        let pos = [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i));
        let angles = [0, 2, 4].map(|i| p.read_i16(o + 0x24 + i));
        let model = p.read_u32(o + 0x88);
        if model != 0 {
            models.insert((model, template));
        }
        let placed = set.placements.iter().any(|q| q.kind == 1 && q.template == template as u32 && q.pos == pos);
        if !placed {
            let near = set.placements.iter().filter(|q| q.kind == 1 && q.template == template as u32).map(|q| q.pos).collect::<Vec<_>>();
            println!(
                "object {o:#x} template {template} id {} at {pos:?} angles {angles:?} room {} flags {:#x}: no placement there (the template's: {near:?})",
                p.read_i16(o + 0x38) as u16,
                p.read_u8(o + 0x3b) as i8,
                p.read_u32(o)
            );
        }
    }
    // The templates.
    let base = p.read_u32(0x800c_4300);
    for (k, t) in set.templates.iter().enumerate() {
        let n = t.raw.len().min(56);
        let orig: Vec<u8> = (0..n as u32).map(|i| p.read_u8(base + 56 * k as u32 + i)).collect();
        if orig[..] != t.raw[..n] {
            let at: Vec<usize> = (0..n).filter(|&i| orig[i] != t.raw[i]).collect();
            println!(
                "template {k}: bytes {at:?} orig {:?} port {:?}",
                at.iter().map(|&i| orig[i]).collect::<Vec<_>>(),
                at.iter().map(|&i| t.raw[i]).collect::<Vec<_>>()
            );
        }
    }
    // The models' boxes, by an object of each.
    for (m, template) in models {
        let Some(t) = set.templates.get(template as usize) else { continue };
        let Some(mine) = set.models.get(t.shape as u32) else { continue };
        let n = p.read_i16(m + 0x0a) as usize;
        let boxes = p.read_u32(m + 0x0c);
        let theirs: Vec<([i16; 3], [i16; 3])> =
            (0..n as u32).map(|b| ([0, 2, 4].map(|i| p.read_i16(boxes + 16 * b + i)), [8, 10, 12].map(|i| p.read_i16(boxes + 16 * b + i)))).collect();
        if theirs != mine.boxes {
            println!("model of template {template} (shape {}): orig {theirs:?} port {:?}", t.shape, mine.boxes);
        }
    }
    // IDS=template: the objects of that template, id and place, beside
    // the port's placements of it.
    if let Some(t) = std::env::var("IDS").ok().and_then(|v| v.parse::<u16>().ok()) {
        for &o in &objs {
            if p.read_i16(o + 0x96) as u16 == t {
                println!("orig id {} at {:?}", p.read_i16(o + 0x38) as u16, [0, 4, 8].map(|i| p.read_i32(o + 0x18 + i)));
            }
        }
        for q in set.placements.iter().filter(|q| q.kind == 1 && q.template == t as u32) {
            println!("port id {} at {:?}", q.id, q.pos);
        }
    }
    println!("{} objects", objs.len());
    Ok(())
}
