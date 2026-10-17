//! Boots level N and prints the player and the time it took.
use std::rc::Rc;
fn main() -> Result<(), String> {
    let n: u8 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?;
    let t = std::time::Instant::now();
    let mut p = rugrats_hle::play::Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), rugrats_hle::play::Boot { level: n, ..Default::default() })?;
    println!("level {} in {:?}: {:?}", p.read_i32(rugrats_hle::play::LEVEL), t.elapsed(), p.player());
    Ok(())
}
