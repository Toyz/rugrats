//! Boots level N and prints player fields: +0x148, +0x15c, +0x15e, the
//! floor, the body box (hierarchy header at +0x154).
use std::rc::Rc;
fn main() -> Result<(), String> {
    let n: u8 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?;
    let mut p = rugrats_hle::play::Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), rugrats_hle::play::Boot { level: n, ..Default::default() })?;
    let b = rugrats_hle::play::PLAYER;
    let h = p.read_u32(b + 0x154);
    let bx: Vec<i16> = (0..8).map(|k| p.read_i16(h + 4 + 2 * k)).collect();
    println!("pos {:?} 148 {} 144 {} 15c {} 15e {} box {:?}", p.player().pos, p.read_i32(b + 0x148), p.read_i32(b + 0x144), p.read_i16(b + 0x15c), p.read_u8(b + 0x15e), bx);
    Ok(())
}
