//! Lists cinematic N's frames whose unknown fields (frame +0x02, actor
//! +0x16) are not 0.
//!
//! cargo run -p rugrats --example cine_unknowns -- N

use rugrats::disc::GameDisc;
use rugrats::media::cine::Stream;

fn main() -> Result<(), String> {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = Stream::read(&disc, n)?;
    for (_, f) in &s.frames {
        let ids: Vec<i16> = std::env::args().skip(2).filter_map(|s| s.parse().ok()).collect();
        let a: Vec<(i16, u8, u8, [i32; 3])> =
            f.actors.iter().filter(|a| ids.contains(&a.id)).map(|a| (a.id, a.mouth, a.eyes, a.pos)).collect();
        let p: Vec<(i16, u8, u8, [i32; 3])> =
            f.poses.iter().filter(|a| ids.contains(&a.id)).map(|a| (a.id, a.mouth, a.eyes, a.pos)).collect();
        if !a.is_empty() || !p.is_empty() {
            println!("frame {} actors {:?} poses {:?}", f.number, a, p);
        }
    }
    println!("{} frames", s.frames.len());
    Ok(())
}
