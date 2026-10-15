//! Plays level DB headless with scripted input and writes a picture every
//! EVERY frames to DIR (N.png), printing the player each frame it moves.
//! SCRIPT is comma-separated BUTTONS*FRAMES, buttons joined by + (up,
//! down, left, right, cross, square, circle, triangle, l1, r1, l2, r2,
//! none).
//!
//! cargo run -p rugrats --example play_script -- DB SCRIPT DIR EVERY

use rrt::gpu::{Gpu, Target};
use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats_render::Renderer;

fn bit(name: &str) -> u16 {
    let b = match name {
        "up" => input::UP,
        "down" => input::DOWN,
        "left" => input::LEFT,
        "right" => input::RIGHT,
        "cross" => input::CROSS,
        "square" => input::SQUARE,
        "circle" => input::CIRCLE,
        "triangle" => input::TRIANGLE,
        "l1" => input::L1,
        "r1" => input::R1,
        "l2" => input::L2,
        "r2" => input::R2,
        "start" => input::START,
        _ => return 0,
    };
    1 << b
}

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let db: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let script = a.next().unwrap_or("up*30".into());
    let dir = a.next().unwrap_or("play".into());
    let every: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let mut game = rugrats::game::Game::open(&disc, db)?;
    for a in [1i16, 2, 3, 4, 15, 28] {
        let id = game.player.anm(a);
        let frames = id.map(|i| game.anims.frames_of(i).len());
        println!("state {a}: anm {id:?}, {frames:?} frames");
    }
    if let Some(h) = std::env::var("HEADING").ok().and_then(|s| s.parse().ok()) {
        game.player.heading = h;
        game.player.target = h;
        game.set_camera_yaw(h as f32);
    }
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    let (w, h) = (512, 240);
    let target = Target::new(&gpu.device, w, h);
    let mut pads = Pads::default();
    let mut frame = 0usize;
    let mut steps = Vec::new();
    for part in script.split(',') {
        let (buttons, n) = part.split_once('*').unwrap_or((part, "1"));
        let bits = buttons.split('+').map(bit).fold(0, |a, b| a | b);
        steps.push((bits, n.parse::<usize>().unwrap_or(1)));
    }
    for (bits, n) in steps {
        for _ in 0..n {
            pads.push_bits(0, bits);
            let before = game.player.pos;
            game.frame(&pads);
            if game.player.pos != before {
                println!("{frame}: {:?} heading {} state {:?} v {:?} floor {} track {:?}", game.player.pos, game.player.heading, game.player.state, game.player.v, game.player.floor, game.player.track());
            }
            if frame.is_multiple_of(every) {
                let stage = &game.stage;
                let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
                r.clear = stage.clear;
                r.set_moving(&gpu.device, &gpu.queue, &stage.moving);
                r.set_semi(&gpu.device, &gpu.queue, &stage.semi);
                r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
                let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
                gpu.queue.submit([cmd]);
                let rgba = target.read_back(&gpu).rgba;
                std::fs::write(format!("{dir}/{frame}.png"), rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())?;
            }
            frame += 1;
        }
    }
    Ok(())
}
