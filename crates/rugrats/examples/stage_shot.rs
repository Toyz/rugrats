//! Renders the title's stage headless, the backdrop with its level set to
//! LEVEL (0..=128), to OUT.png.
//!
//! cargo run -p rugrats --example stage_shot -- LEVEL OUT.png

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let level: u8 = a.next().and_then(|s| s.parse().ok()).unwrap_or(0x80);
    let out = a.next().unwrap_or("stage.png".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let mut title = rugrats::front::Title::open(&disc, 0)?;
    title.backdrop.level = level;
    title.stage.under.clear();
    title.backdrop.triangles(&mut title.stage.under);
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (512, 384);
    let target = Target::new(&gpu.device, w, h);
    let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &title.stage.vram, &title.stage.fixed);
    r.set_under(&gpu.device, &gpu.queue, &title.stage.under);
    r.set_overlay(&gpu.device, &gpu.queue, &title.stage.overlay);
    let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), title.stage.camera);
    gpu.queue.submit([cmd]);
    let rgba = target.read_back(&gpu).rgba;
    std::fs::write(&out, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())
}
