//! The title's babies, frame by frame from rand's first state, in the
//! oracle's title_babies line format (state, corner, baby, position,
//! target, angles, targets, speeds), to diff against it.
//!
//! cargo run -p rugrats --example babies_sim -- N

use rugrats::front::{Baby, InUse};
use rugrats::rand::Rand;

fn main() {
    let frames: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut rng = Rand::default();
    // The front end's setup draws once before the first frame.
    rng.next();
    let mut babies = [Baby::default(), Baby::default()];
    let mut used = InUse::default();
    for frame in 1..=frames {
        for b in &mut babies {
            b.think(&mut used, &mut rng);
            b.step();
        }
        let mut line = format!("{frame:5} rng {:08x}", rng.state);
        for b in &babies {
            line += &format!(
                " | st {} c {} k {} h Some({}) pos {},{},{} tgt {},{},{} ang {},{},{} to {},{},{} sp {},{},{}",
                b.state, b.corner, b.kind, b.hierarchy,
                b.pos[0], b.pos[1], b.pos[2], b.target[0], b.target[1], b.target[2],
                b.angles[0], b.angles[1], b.angles[2], b.turn_to[0], b.turn_to[1], b.turn_to[2],
                b.speed[0], b.speed[1], b.turn
            );
        }
        println!("{line}");
    }
}
