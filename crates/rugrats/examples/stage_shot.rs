//! Renders the title's stage headless after TICKS ticks to OUT.png.
//!
//! cargo run -p rugrats --example stage_shot -- TICKS OUT.png

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(120);
    let out = a.next().unwrap_or("stage.png".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let mut rng = rugrats::rand::Rand::default();
    let mut title = rugrats::front::Title::open(&disc, 0, &mut rng)?;
    let pads = rugrats::input::Pads::default();
    for t in 0..ticks {
        if title.frame_due(t) {
            title.frame(&pads, &mut rng);
        }
    }
    let stage = &title.stage;
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (640, 480);
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
    std::fs::write(&out, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())
}
