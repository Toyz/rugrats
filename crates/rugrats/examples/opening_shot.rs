//! Renders the opening headless at stream frame F to OUT.png (512 x 240,
//! the original's screen); with TO and STEP, every STEP frames from F to
//! TO into the directory OUT (F.png and so on).
//!
//! cargo run -p rugrats --example opening_shot -- F OUT.png
//! cargo run -p rugrats --example opening_shot -- F OUT TO STEP

use std::sync::{Arc, Mutex};

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats::movie::Soundtrack;
use rugrats_render::Renderer;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let from: u16 = a.next().and_then(|s| s.parse().ok()).unwrap_or(300);
    let out = a.next().unwrap_or("cine.png".into());
    let to: Option<u16> = a.next().and_then(|s| s.parse().ok());
    let step: u16 = a.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let audio = Arc::new(Mutex::new(Soundtrack::new(Vec::new())));
    let env = |k: &str, d: usize| std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
    let mut cine = rugrats::cine::Cine::open(&disc, env("CINE", 0), env("DB", 22), 0, &audio)?;
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (640, 480);
    let target = Target::new(&gpu.device, w, h);
    let mut t = 0;
    let mut want = from;
    loop {
        while cine.frame().is_none_or(|f| f < want) && !cine.over() {
            cine.tick(t);
            t += 1;
        }
        let stage = &cine.stage;
        let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
        r.clear = stage.clear;
        r.set_moving(&gpu.device, &gpu.queue, &stage.moving);
        r.set_semi(&gpu.device, &gpu.queue, &stage.semi);
        r.set_under(&gpu.device, &gpu.queue, &stage.under);
        r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
        let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
        gpu.queue.submit([cmd]);
        let rgba = target.read_back(&gpu).rgba;
        let path = match to {
            Some(_) => format!("{out}/{want}.png"),
            None => out.clone(),
        };
        std::fs::write(&path, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())?;
        println!("frame {:?} -> {path}", cine.frame());
        match to {
            Some(to) if want + step <= to && !cine.over() => want += step,
            _ => return Ok(()),
        }
    }
}
