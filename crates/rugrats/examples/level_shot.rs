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
    for p in game.set.placements.iter().filter(|p| p.level != 0) {
        println!("  door {p:?} template {:?}", game.set.templates.get(p.template as usize));
    }
    for p in game.set.placements.iter().filter(|p| game.set.templates.get(p.template as usize).is_some_and(|t| t.hierarchy().is_some()) && (p.pos[0] - 5775).abs() < 3000 && (p.pos[2] - 19403).abs() < 3000) {
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
