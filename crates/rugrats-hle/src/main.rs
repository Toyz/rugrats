//! rugrats-hle: run the original game headless for N frames.
//!
//! ```text
//! rugrats-hle [FRAMES] [--shots DIR] [--every N] [--cue FILE]
//!             [--peek ADDR:LEN,...]    memory after the run, as words
//! ```
//!
//! Writes the displayed picture every N frames, and all of VRAM at the end,
//! as PNG.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::rc::Rc;

use rrt::emu::BusExt;

fn parse_num(s: &str) -> u32 {
    match s.strip_prefix("0x") {
        Some(h) => u32::from_str_radix(h, 16).unwrap_or(0),
        None => s.parse().unwrap_or(0),
    }
}

fn png(path: &std::path::Path, w: usize, h: usize, rgba: &[u8]) {
    let bytes = rrt::image::png::encode(w as u32, h as u32, rgba);
    if let Err(e) = std::fs::write(path, bytes) {
        tracing::error!("{}: {e}", path.display());
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .with_ansi(false)
        .init();
    let mut args = std::env::args().skip(1);
    let (mut frames, mut shots, mut every) = (120u64, PathBuf::from("work/hle"), 30u64);
    let mut cue: Option<PathBuf> = None;
    let mut peeks: Vec<(u32, u32)> = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--shots" => shots = args.next().map(PathBuf::from).unwrap_or(shots),
            "--every" => every = args.next().and_then(|s| s.parse().ok()).unwrap_or(every),
            "--cue" => cue = args.next().map(PathBuf::from),
            "--peek" => {
                for p in args.next().unwrap_or_default().split(',') {
                    if let Some((a, n)) = p.split_once(':') {
                        peeks.push((parse_num(a), parse_num(n)));
                    }
                }
            }
            n => frames = n.parse().unwrap_or(frames),
        }
    }
    let cue = cue.unwrap_or_else(|| rrt::disc::Image::find(std::path::Path::new("work/disc")).expect("no .cue under work/disc"));
    let disc = Rc::new(rrt::disc::Image::open(&cue).expect("open the disc"));
    let mut hle = rugrats_hle::Hle::new(disc).expect("load the game");
    std::fs::create_dir_all(&shots).ok();
    let started = std::time::Instant::now();
    for f in 1..=frames {
        if let Err(e) = hle.run_frame() {
            tracing::error!("frame {f}: {e}");
            break;
        }
        if every > 0 && f % every == 0 {
            let (w, h, rgba) = hle.screen();
            png(&shots.join(format!("frame{f:05}.png")), w, h, &rgba);
        }
    }
    let gpu = &hle.m.bus.hw.gpu;
    png(&shots.join("vram.png"), 1024, 512, &gpu.vram_rgba());
    tracing::info!(
        "{} frames, {} steps in {:.1}s; pc {:08x}; {} files read",
        hle.frame,
        hle.steps,
        started.elapsed().as_secs_f64(),
        rrt::emu::Cpu::pc(&hle.m.cpu),
        hle.reads.borrow().len()
    );
    tracing::info!(
        "interrupts: {} dispatched, held {:?} (masked, critical, no dispatcher); i_stat {:x} i_mask {:x}",
        hle.dispatched,
        hle.held,
        hle.m.bus.hw.i_stat,
        hle.m.bus.hw.i_mask
    );
    for (addr, n) in &hle.m.bus.hw.unhandled {
        tracing::info!("unmodelled I/O {addr:08x} x{n}");
    }
    for (a, n) in peeks {
        let words: Vec<String> =
            (0..n.div_ceil(4)).map(|k| format!("{:08x}", hle.m.bus.read_u32(a + 4 * k).unwrap_or(0))).collect();
        println!("{a:08x}: {}", words.join(" "));
    }
}
