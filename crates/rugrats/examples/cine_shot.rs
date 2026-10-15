//! Renders the opening headless at stream frame F to OUT.png (512 x 240,
//! the original's screen).
//!
//! cargo run -p rugrats --example cine_shot -- F OUT.png

use std::sync::{Arc, Mutex};

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats::movie::Soundtrack;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let want: u16 = a.next().and_then(|s| s.parse().ok()).unwrap_or(300);
    let out = a.next().unwrap_or("cine.png".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let audio = Arc::new(Mutex::new(Soundtrack::new(Vec::new())));
    let mut cine = rugrats::cine::Cine::open(&disc, 0, 22, 0, &audio)?;
    let mut t = 0;
    while cine.frame().is_none_or(|f| f < want) && !cine.over() {
        cine.tick(t);
        t += 1;
    }
    let stage = &cine.stage;
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
    println!("frame {:?} -> {out}", cine.frame());
    Ok(())
}
