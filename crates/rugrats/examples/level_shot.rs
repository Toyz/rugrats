//! Renders database DB in play headless to OUT.png (512 x 240).
//!
//! cargo run -p rugrats --example level_shot -- DB OUT.png

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let db: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let out = a.next().unwrap_or("level.png".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let game = rugrats::game::Game::open(&disc, db)?;
    println!("player at {:?}, heading {}", game.player.pos, game.player.heading);
    let mut g2 = game;
    g2.frame(&rugrats::input::Pads::default());
    println!("song asked: {:?}; level set {:?}", g2.song, rugrats::sound::level_set(&disc, db));
    let p = g2.player.pos;
    for (k, r) in g2.set.room_info.iter().enumerate() {
        let inside = (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i]);
        if inside || r.flags & 128 != 0 {
            println!("room {k}: {r:?} inside {inside}");
        }
    }
    for p in g2.set.placements.iter().filter(|p| p.kind == 6) {
        let b = p.template.to_le_bytes();
        let c = (p.pos[0] as u32).to_le_bytes();
        println!("kind6 room {} bytes {:?} {:?}", b[0], &b[1..], c);
    }
    let picks: Vec<_> = g2
        .set
        .placements
        .iter()
        .filter(|p| {
            p.kind == 1
                && g2.set.templates.get(p.template as usize).is_some_and(|t| t.pickable() && t.hierarchy().is_none())
        })
        .map(|p| (p.template, p.pos))
        .collect();
    println!("{} pickables: {:?}", picks.len(), &picks[..picks.len().min(12)]);
    let game = g2;
    println!("{} boxes", game.boxes.len());
    for n in &game.npcs {
        println!(
            "npc track {:?} parts {} models {:?}",
            n.animator.track(),
            n.hierarchy.parts.len(),
            n.hierarchy.parts.iter().map(|p| p.model).collect::<Vec<_>>()
        );
    }
    for z in [20148] {
        for b in game.boxes.iter().filter(|b| (b.pos.x - 7570.0).abs() < 10.0 && b.pos.y > 0.0) {
            for k in 0..22 {
                let p = [5775 + 157 * k, 0, 19403 + 65 * k];
                println!("    {p:?} depth {}", b.depth(p, 160));
            }
            let c = b.pos + b.rot * rrt::glam::Vec3::new(0.0, 0.0, -802.0);
            let e = b.pos + b.rot * rrt::glam::Vec3::new(1524.0, 0.0, -802.0);
            println!("    plane centre {c:?} end {e:?}");
            println!("  z {z}: box lo {:?} hi {:?} at {:?} y {}..{}", b.lo, b.hi, b.pos, b.min_y, b.max_y);
        }
    }
    for b in game.boxes.iter().filter(|b| (b.pos.x as i32 - 5775).abs() < 4000 && (b.pos.z as i32 - 19403).abs() < 4000)
    {
        println!("  box {b:?}");
    }
    for p in game
        .set
        .placements
        .iter()
        .filter(|p| p.kind == 1 && (p.pos[0] - 5775).abs() < 4000 && (p.pos[2] - 19403).abs() < 4000)
    {
        let t = game.set.templates.get(p.template as usize);
        let boxes = t.and_then(|t| game.set.models.get(t.shape as u32)).map(|m| m.boxes.len());
        println!("  near {p:?} {t:?} boxes {boxes:?}");
    }
    for p in game.set.placements.iter().filter(|p| p.level != 0 && false) {
        println!("  door {p:?} template {:?}", game.set.templates.get(p.template as usize));
    }
    for p in game.set.placements.iter().filter(|p| {
        game.set.templates.get(p.template as usize).is_some_and(|t| t.hierarchy().is_some())
            && (p.pos[0] - 5775).abs() < 3000
            && (p.pos[2] - 19403).abs() < 3000
    }) {
        println!("  {p:?} hier {:?}", game.set.templates[p.template as usize].hierarchy());
    }
    let stage = &game.stage;
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (512, 240);
    let target = Target::new(&gpu.device, w, h);
    let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
    r.clear = stage.clear;
    r.set_moving(&gpu.device, &gpu.queue, &stage.moving);
    r.set_semi(&gpu.device, &gpu.queue, &stage.semi);
    r.set_under(&gpu.device, &gpu.queue, &stage.under);
    r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
    let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
    gpu.queue.submit([cmd]);
    let rgba = target.read_back(&gpu).rgba;
    std::fs::write(&out, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())?;
    Ok(())
}
