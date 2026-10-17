//! Lists the original's objects as the collision tests see them, from the
//! rooms' object lists (0x800c4a8c: 112 bytes a room, lists at +0x48 and
//! +0x4c), beside the port's placements at the same spot.
//!
//! `solids_dump LEVEL`
use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, Play};

#[path = "common/oracle.rs"]
mod oracle;

fn main() -> Result<(), String> {
    let level: u8 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(18);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = GameDisc::open(&cue)?;
    let game = Game::open_from(&disc, Level(level), None, &Progress::default())?;
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level, ..Default::default() })?;
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    let types = p.read_u32(0x800c_4300);
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                let o = p.read_u32(node + 8);
                let kind = p.read_i16(o + 0x96) as u16 as u32;
                let flags = p.read_u32(types + 56 * kind);
                let pos = [0, 4, 8].map(|k| p.read_i32(o + 0x18 + k));
                let angles = [0, 1, 2].map(|k| p.read_i16(o + 0x24 + 2 * k));
                let bound: Vec<i16> = (0..8).map(|k| p.read_i16(o + 0x78 + 2 * k)).collect();
                let size = p.read_i16(o + 0x86);
                let model = p.read_u32(o + 0x88);
                let boxes = if model == 0 { -1 } else { p.read_i16(model + 10) as i32 };
                let raw: Vec<String> = (0..8).map(|k| format!("{:04x}", p.read_i16(o + 0x8c + 2 * k) as u16)).collect();
                let placed = game.set.placements.iter().position(|pl| pl.pos == pos);
                let port = placed.map(|k| {
                    let pl = &game.set.placements[k];
                    let a = game.trig.placed_angles(pl.rotation);
                    let mut mark = if a == angles { String::new() } else { " ANGLES DIFFER".into() };
                    let shape = game.set.templates.get(pl.template as usize).map(|t| t.shape as u32);
                    if let Some(m) = shape.and_then(|s| game.set.models.get(s)) {
                        if m.radius as i32 != size as i32 {
                            mark += &format!(" RADIUS {}", m.radius);
                        }
                        if model != 0 && m.boxes.len() as i32 != boxes {
                            mark += &format!(" BOXES {}", m.boxes.len());
                        }
                        if model != 0 {
                            let at = p.read_u32(model + 12);
                            let same = m.boxes.iter().enumerate().all(|(j, (lo, hi))| {
                                (0..3).all(|k| p.read_i16(at + 16 * j as u32 + 2 * k) == lo[k as usize] && p.read_i16(at + 16 * j as u32 + 8 + 2 * k) == hi[k as usize])
                            });
                            if !same {
                                mark += " BOX VALUES";
                            }
                        }
                    } else {
                        mark += " NO MODEL";
                    }
                    format!("placement {k} template {} rotation {:?} angles {a:?}{mark}", pl.template, pl.rotation)
                });
                println!(
                    "room {r} +{list:x} {o:08x} type {kind} flags {flags:#x} oflags {:#x} pos {pos:?} angles {angles:?} bound {bound:?} size {size} boxes {boxes} 8c.. {} | {}",
                    p.read_u32(o),
                    raw.join(" "),
                    port.unwrap_or_else(|| "no placement here".into())
                );
                node = p.read_u32(node + 4);
            }
        }
    }
    Ok(())
}
