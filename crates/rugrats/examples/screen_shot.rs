//! Runs the result screens headless with scripted input: a piece message
//! and the board (EARNED and PLACED bit masks, default one piece), or the
//! bar page (BARS=n). Writes a picture every EVERY frames to DIR.
//!
//! cargo run -p rugrats --example screen_shot -- SCRIPT DIR EVERY

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::screens::{Page, Screen, message};
use rugrats_render::Renderer;

fn bit(name: &str) -> u16 {
    let b = match name {
        "up" => input::UP,
        "down" => input::DOWN,
        "left" => input::LEFT,
        "right" => input::RIGHT,
        "cross" => input::CROSS,
        "triangle" => input::TRIANGLE,
        _ => return 0,
    };
    1 << b
}

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let script = a.next().unwrap_or("none*60".into());
    let dir = a.next().unwrap_or("screen".into());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let every: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let env = |k: &str, d: u16| std::env::var(k).ok().and_then(|v| u16::from_str_radix(v.trim_start_matches("0x"), 16).ok()).unwrap_or(d);
    let (earned, placed) = (env("EARNED", 1), env("PLACED", 0));
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let pages = match std::env::var("BARS").ok().and_then(|v| v.parse().ok()) {
        Some(n) => vec![message::bars(&disc, n, 7)],
        None if std::env::var_os("RESTART").is_some() => vec![Page::Restart],
        None if std::env::var("LOADING").is_ok() => {
            let l = std::env::var("LOADING").unwrap().parse().unwrap_or(2);
            vec![message::loading(&disc, l, false, false)]
        }
        None => vec![message::piece(&disc), Page::Board],
    };
    let level = rugrats::game::Game::open(&disc, 2)?;
    let mut screen = Screen::open(&disc, pages, 2, earned, placed, Some(&level.stage.vram))?;
    let mut rand = rugrats::rand::Rand::default();
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (640, 480);
    let target = Target::new(&gpu.device, w, h);
    let mut pads = Pads::default();
    let mut frame = 0usize;
    for part in script.split(',') {
        let (buttons, n) = part.split_once('*').unwrap_or((part, "1"));
        let bits = buttons.split('+').map(bit).fold(0, |a, b| a | b);
        for _ in 0..n.parse::<usize>().unwrap_or(1) {
            pads.push_bits(0, bits);
            let going = screen.frame(&pads, &mut rand);
            if !screen.sounds.is_empty() {
                println!("{frame}: sounds {:?}", screen.sounds);
            }
            if frame.is_multiple_of(every) || !going {
                let stage = &screen.stage;
                let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
                r.clear = stage.clear;
                r.set_under(&gpu.device, &gpu.queue, &stage.under);
                r.set_moving(&gpu.device, &gpu.queue, &stage.moving);
                r.set_semi(&gpu.device, &gpu.queue, &stage.semi);
                r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
                let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
                gpu.queue.submit([cmd]);
                let rgba = target.read_back(&gpu).rgba;
                std::fs::write(format!("{dir}/{frame}.png"), rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())?;
            }
            if !going {
                println!("{frame}: over, placed {:#x}, all {}, restart {}", screen.placed, screen.all_placed, screen.restart);
                return Ok(());
            }
            frame += 1;
        }
    }
    Ok(())
}
