//! Renders hint card FILE (672-713) headless to OUT.png.
//!
//! cargo run -p rugrats --example card_shot -- FILE OUT.png

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let file: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(672);
    let out = a.next().unwrap_or("card.png".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let card = rugrats::front::Card::open(&disc, file)?;
    let stage = &card.stage;
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (512, 240);
    let target = Target::new(&gpu.device, w, h);
    let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
    r.clear = stage.clear;
    r.set_under(&gpu.device, &gpu.queue, &stage.under);
    r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
    let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
    gpu.queue.submit([cmd]);
    let rgba = target.read_back(&gpu).rgba;
    std::fs::write(&out, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())
}
