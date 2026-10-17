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
    let db = rugrats::levels::Level(a.next().and_then(|s| s.parse().ok()).unwrap_or(1));
    let script = a.next().unwrap_or("up*30".into());
    let dir = a.next().unwrap_or("play".into());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let every: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let mut game = rugrats::world::game::Game::open(&disc, db)?;
    // START=x,y,z,heading puts the player there first.
    if let Ok(v) = std::env::var("START") {
        let n: Vec<i32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        if let [x, y, z, h] = n[..] {
            game.player.pos = [x, y, z];
            game.player.heading = h;
            game.player.target = h;
            game.set_camera_yaw(h as f32);
        }
    }
    // BOXAT=x,z: the object boxes covering that point.
    if let Ok(v) = std::env::var("BOXAT") {
        let p: Vec<i32> = v.split(',').filter_map(|s| s.parse().ok()).collect();
        for b in game.boxes.iter().filter(|b| b.covers([p[0], 0, p[1]], 160)) {
            println!("box at {:?}: y {}..{} climb {}", b.pos, b.min_y, b.max_y, b.climb);
        }
    }
    // PHASE=n: the level's set-up for phase n (Script::skip_to).
    if let Some(n) = std::env::var("PHASE").ok().and_then(|s| s.parse().ok()) {
        game.with_script(|s, g| s.skip_to(g, n));
    }
    // FINISHED=n: the level told cinematic n finished (its end hook).
    if let Some(n) = std::env::var("FINISHED").ok().and_then(|s| s.parse().ok()) {
        game.cine_finished(n);
    }
    if let Ok(v) = std::env::var("START") {
        let p: Vec<i32> = v.split(',').filter_map(|s| s.parse().ok()).collect();
        if p.len() == 3 {
            game.player.pos = [p[0], p[1], p[2]];
        }
    }
    if let Some(h) = std::env::var("HEADING").ok().and_then(|s| s.parse().ok()) {
        game.player.heading = h;
        game.player.target = h;
        game.set_camera_yaw(h as f32);
    }
    if std::env::var_os("LISTTHINGS").is_some() {
        for (k, t) in game.things.iter().enumerate() {
            println!("thing {k}: template {} at {:?}", t.template, t.pos);
        }
    }
    // THINGS=k:x,y,z;k:x,y,z: things moved before the run.
    if let Ok(v) = std::env::var("THINGS") {
        for part in v.split(';') {
            if let Some((k, at)) = part.split_once(':') {
                let p: Vec<f32> = at.split(',').filter_map(|s| s.parse().ok()).collect();
                if let (Ok(k), 3) = (k.parse::<usize>(), p.len())
                    && let Some(t) = game.things.get_mut(k)
                {
                    t.pos = rrt::glam::Vec3::new(p[0], p[1], p[2]);
                }
            }
        }
    }
    if let Some(h) = std::env::var("HEALTH").ok().and_then(|s| s.parse().ok()) {
        game.player.health = h;
    }
    let gpu = Gpu::headless().map_err(|e| e.to_string())?;
    // SHOT_SIZE=WxH for another size than 640 x 480.
    let (w, h) = std::env::var("SHOT_SIZE")
        .ok()
        .and_then(|s| s.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))))
        .unwrap_or((640, 480));
    let target = Target::new(&gpu.device, w, h);
    let mut pads = Pads::default();
    let mut frame = 0usize;
    let mut steps = Vec::new();
    let mut shown = None;
    for part in script.split(',') {
        let (buttons, n) = part.split_once('*').unwrap_or((part, "1"));
        let bits = buttons.split('+').map(bit).fold(0, |a, b| a | b);
        steps.push((bits, n.parse::<usize>().unwrap_or(1)));
    }
    for (bits, n) in steps {
        for _ in 0..n {
            pads.push_bits(0, bits);
            // TP=frame:x,y,z: a teleport at that frame.
            if let Some((f, at)) = std::env::var("TP").ok().as_deref().and_then(|v| v.split_once(':')) {
                let p: Vec<i32> = at.split(',').filter_map(|s| s.parse().ok()).collect();
                if f.parse() == Ok(frame) && p.len() == 3 {
                    game.player.pos = [p[0], p[1], p[2]];
                }
            }
            let before = game.player.pos;
            game.frame(&pads);
            for (k, t) in game.things.iter().enumerate() {
                if t.state != rugrats::world::game::ThingState::Rest {
                    println!("{frame}: thing {k} {:?} at {:?}", t.state, t.pos);
                }
            }
            if let Some(d) = game
                .script
                .as_ref()
                .map(|s| s.describe())
                .filter(|d| !d.is_empty() && std::env::var_os("DESCRIBE").is_some())
            {
                println!("{frame}: {d}");
            }
            if std::env::var_os("GOODIES").is_some() && frame == 0 {
                println!("goodies {:?}", game.goodies.iter().map(|g| (g.kind, g.pos, g.taken)).collect::<Vec<_>>());
            }
            if std::env::var_os("DOORS").is_some() && frame == 0 {
                for d in &game.doors {
                    println!("door {}", d.describe(&game.set.models, &game.trig));
                }
            }
            if std::env::var_os("DOORS").is_some() && frame.is_multiple_of(10) {
                println!(
                    "{frame}: doors {:?}",
                    game.doors
                        .iter()
                        .filter(|d| d.moving || d.open)
                        .map(|d| (d.template, d.open, d.moving))
                        .collect::<Vec<_>>()
                );
            }
            if std::env::var_os("WALKERS").is_some() && frame.is_multiple_of(30) {
                println!("{frame}: walkers {:?}", game.walkers.iter().map(|w| (w.template, w.pos)).collect::<Vec<_>>());
            }
            if std::env::var_os("MOVERS").is_some() && frame.is_multiple_of(30) {
                println!("{frame}: movers {:?}", game.movers.iter().map(|m| (m.id, m.pos)).collect::<Vec<_>>());
            }
            if frame.is_multiple_of(30) {
                {
                    let phase = game.script.as_ref().map_or(0, |l| l.phase());
                    // NPCS=id,id: the characters to show (level 2's goose and
                    // Chuckie by default).
                    let ids: Vec<u16> = std::env::var("NPCS")
                        .ok()
                        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
                        .unwrap_or(vec![11, 1776]);
                    // CHARS=c: every character of hierarchy c instead.
                    let chars: Vec<usize> = std::env::var("CHARS")
                        .ok()
                        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
                        .unwrap_or_default();
                    let picked: Vec<&rugrats::actors::npc::Npc> = if chars.is_empty() {
                        ids.iter().filter_map(|&id| game.npcs.iter().find(|n| n.id == id)).collect()
                    } else {
                        game.npcs.iter().filter(|n| chars.contains(&n.character)).collect()
                    };
                    let npcs: Vec<_> = picked
                        .into_iter()
                        .map(|n| (n.id, n.pos, n.health, format!("{:?}", n.mode), n.hidden))
                        .collect();
                    println!(
                        "{frame}: phase {} health {} tries {} npcs {npcs:?}",
                        phase, game.player.health, game.tries
                    );
                }
            }
            if let Some(n) = game.training.showing().filter(|_| shown != game.training.showing()) {
                println!("{frame}: training message {n}");
            }
            shown = game.training.showing();
            if !game.speech.is_empty() {
                println!("{frame}: speech {:?}", game.speech);
            }
            if let Some(b) = game.loaded.take() {
                println!("{frame}: loaded a record, level {}", u32::from_le_bytes(b[0..4].try_into().unwrap()));
            }
            if let Some(e) = game.exit.take() {
                println!("{frame}: exit to {e:?}");
            }
            if game.quit {
                println!("{frame}: quit");
            }
            if let Some(n) = game.cine.take() {
                println!("{frame}: the level asks for cine {n}");
                game.cine_finished(n);
            }
            if game.player.pos != before {
                println!(
                    "{frame}: {:?} heading {} state {:?} v {:?} floor {} track {:?}",
                    game.player.pos,
                    game.player.heading,
                    game.player.state,
                    game.player.v,
                    game.player.floor,
                    game.player.track()
                );
            }
            if std::env::var_os("OVERLAY").is_some() {
                println!(
                    "{frame}: overlay {} vertices, first {:?}",
                    game.stage.overlay.len(),
                    game.stage.overlay.first().map(|v| v.pos)
                );
            }
            if frame.is_multiple_of(every) {
                if std::env::var_os("BIGMETER").is_some()
                    && let Some(sp) = &game.sprites
                {
                    let f: f32 = std::env::var("BIGF").ok().and_then(|s| s.parse().ok()).unwrap_or(480.0);
                    sp.draw(258, (192, 76), 32.0, 24.0, 768.0, 0.0, f, &mut game.stage.overlay);
                    sp.draw(258, (0, 0), 32.0, 24.0, 768.0, 0.0, f, &mut game.stage.overlay);
                }
                let stage = game.view();
                let mut r = Renderer::new(&gpu.device, &gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
                r.clear = stage.clear;
                r.set_moving(&gpu.device, &gpu.queue, &stage.moving);
                r.set_under(&gpu.device, &gpu.queue, &stage.under);
                r.set_semi(&gpu.device, &gpu.queue, &stage.semi);
                r.set_overlay(&gpu.device, &gpu.queue, &stage.overlay);
                let cmd = r.draw(&gpu.device, &gpu.queue, &target.view, (w, h), stage.camera);
                gpu.queue.submit([cmd]);
                let rgba = target.read_back(&gpu).rgba;
                std::fs::write(format!("{dir}/{frame}.png"), rrt::image::png::encode(w, h, &rgba))
                    .map_err(|e| e.to_string())?;
            }
            frame += 1;
        }
    }
    if std::env::var_os("LISTTHINGS").is_some() {
        for (k, t) in game.things.iter().enumerate() {
            println!("end thing {k}: {:?} at {:?}", t.state, t.pos);
        }
    }
    Ok(())
}
