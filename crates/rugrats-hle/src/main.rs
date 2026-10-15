//! rugrats-hle: run the original game headless for N frames.
//!
//! ```text
//! rugrats-hle [FRAMES] [--shots DIR] [--every N] [--cue FILE]
//!             [--real-cd]              loads through libcd and the drive, not the shortcut
//!             [--press F:BUTTON[+BUTTON][:LEN],...]   hold buttons from frame F (4 frames by default)
//!             [--peek ADDR:LEN,...]    memory after the run, as words
//!             [--count ADDR,...]       calls of each function, per frame
//!             [--poke ADDR=WORD,...]   words written before every frame
//!             [--trace-from F]         record executed addresses from frame F; the last shown on a stop
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
    let mut fast_load = true;
    // (first frame, buttons, frames held)
    let mut presses: Vec<(u64, u16, u64)> = Vec::new();
    let mut peeks: Vec<(u32, u32)> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    // --poke ADDR=WORD: written before every frame.
    let mut pokes_each: Vec<(u32, u32)> = Vec::new();
    let mut trace_from: Option<u64> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--shots" => shots = args.next().map(PathBuf::from).unwrap_or(shots),
            "--every" => every = args.next().and_then(|s| s.parse().ok()).unwrap_or(every),
            "--cue" => cue = args.next().map(PathBuf::from),
            "--real-cd" => fast_load = false,
            "--press" => {
                for p in args.next().unwrap_or_default().split(',') {
                    let f: Vec<&str> = p.split(':').collect();
                    let frame = f.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                    let buttons = f.get(1).map_or(0, |n| n.split('+').filter_map(rugrats_hle::button::by_name).fold(0, |a, b| a | b));
                    let len = f.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
                    presses.push((frame, buttons, len));
                }
            }
            "--poke" => {
                for p in args.next().unwrap_or_default().split(',') {
                    if let Some((a, v)) = p.split_once('=') {
                        pokes_each.push((parse_num(a), parse_num(v)));
                    }
                }
            }
            "--trace-from" => trace_from = args.next().and_then(|s| s.parse().ok()),
            "--count" => counts.extend(args.next().unwrap_or_default().split(',').map(parse_num)),
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
    let mut hle = rugrats_hle::Hle::new(disc, fast_load).expect("load the game");
    std::fs::create_dir_all(&shots).ok();
    // --count: calls of each function, logged per frame when they change.
    let tallies: Vec<std::rc::Rc<std::cell::Cell<u64>>> = counts.iter().map(|_| Default::default()).collect();
    for (&f, t) in counts.iter().zip(&tallies) {
        let t = t.clone();
        hle.m.hook(f, move |_| {
            t.set(t.get() + 1);
            rrt::emu::HookResult::Continue
        });
    }
    let mut last: Vec<u64> = vec![0; counts.len()];
    let started = std::time::Instant::now();
    for f in 1..=frames {
        for &(a, v) in &pokes_each {
            hle.m.bus.load_ram(a, &v.to_le_bytes());
        }
        hle.pad = presses.iter().filter(|(at, _, len)| (*at..at + len).contains(&f)).fold(0, |a, p| a | p.1);
        if trace_from == Some(f) {
            hle.m.trace = Some(Vec::new());
        }
        if let Some(t) = &mut hle.m.trace
            && t.len() > 4_000_000
        {
            t.drain(..t.len() - 1000);
        }
        if let Err(e) = hle.run_frame() {
            tracing::error!("frame {f}: {e}");
            let r = &hle.m.cpu.r;
            tracing::error!("sp {:08x} ra {:08x} fp {:08x} gp {:08x} s0 {:08x} s1 {:08x}", r[29], r[31], r[30], r[28], r[16], r[17]);
            if let Some(t) = &hle.m.trace {
                let tail: Vec<String> = t.iter().rev().take(3_000_000).rev().map(|a| format!("{a:08x}")).collect();
                tracing::error!("last steps: {}", tail.join(" "));
            }
            break;
        }
        for (k, t) in tallies.iter().enumerate() {
            if t.get() != last[k] {
                println!("frame {f}: {:08x} called {} times", counts[k], t.get() - last[k]);
                last[k] = t.get();
            }
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
    tracing::info!("display {:?}, {} primitives drawn", hle.m.bus.hw.gpu.display, hle.m.bus.hw.gpu.drawn);
    let cd = &hle.m.bus.hw.cd;
    tracing::info!("CD commands {:?}; {} XA sectors to the sound side", cd.commands, cd.xa_sectors);
    for (addr, n) in &hle.m.bus.hw.unhandled {
        tracing::info!("unmodelled I/O {addr:08x} x{n}");
    }
    for (a, n) in peeks {
        let words: Vec<String> =
            (0..n.div_ceil(4)).map(|k| format!("{:08x}", hle.m.bus.read_u32(a + 4 * k).unwrap_or(0))).collect();
        println!("{a:08x}: {}", words.join(" "));
    }
}
