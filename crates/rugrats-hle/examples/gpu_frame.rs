//! Runs the original to frame N, writes the picture, then lists every
//! polygon and rectangle the next two frames draw (VRAM coordinates), and
//! VRAM itself with GPU_VRAM=PATH: what makes up a screen.
//!
//! cargo run -p rugrats-hle --example gpu_frame -- N OUT.png [--press F:BUTTON...]

use std::rc::Rc;

fn main() {
    let mut args = std::env::args().skip(1);
    let frames: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1800);
    let out = args.next().unwrap_or_else(|| "work/hle/frame.png".into());
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    while hle.frame < frames {
        hle.run_frame().unwrap();
    }
    let (w, h, rgba) = hle.screen();
    std::fs::write(&out, rrt::image::png::encode(w as u32, h as u32, &rgba)).unwrap();
    println!("frame {} -> {out} ({w}x{h}), display {:?}", hle.frame, hle.m.bus.hw.gpu.display);
    if let Some(path) = std::env::var_os("GPU_VRAM") {
        let bytes: Vec<u8> = hle.m.bus.hw.gpu.vram.iter().flat_map(|h| h.to_le_bytes()).collect();
        std::fs::write(path, bytes).unwrap();
    }
    hle.m.bus.hw.gpu.trace = Some([-4096, -4096, 4096, 4096]);
    hle.run_frame().unwrap();
    hle.run_frame().unwrap();
    hle.m.bus.hw.gpu.trace = None;
}
