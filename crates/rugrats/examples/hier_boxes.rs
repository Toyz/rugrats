//! Prints each hierarchy's body box in database DB (header +4: lo, hi).
use std::path::Path;
fn main() -> Result<(), String> {
    let db: u8 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let disc = rugrats::disc::GameDisc::open(&cue)?;
    let game = rugrats::world::game::Game::open_from(&disc, rugrats::levels::Level(db), None, &rugrats::save::progress::Progress::default())?;
    for (k, h) in game.set.hierarchies.iter().enumerate() {
        if let Some(h) = h {
            let v = |at: usize| h.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32);
            println!("hier {k}: parts {} box ({}, {}, {})..({}, {}, {})", h.parts.len(), v(4), v(6), v(8), v(0xc), v(0xe), v(0x10));
        }
    }
    Ok(())
}
