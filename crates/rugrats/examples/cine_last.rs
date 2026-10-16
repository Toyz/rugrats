//! Where cinematic N's last frames leave its actors and posed characters:
//! `cargo run --example cine_last N [frames]`.

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|a| a.parse().ok()).ok_or("cinematic number")?;
    let tail: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = rugrats::media::cine::Stream::read(&disc, n)?;
    println!("cine {n}: {} frames", s.frames.len());
    for (_, f) in &s.frames[s.frames.len().saturating_sub(tail)..] {
        println!("frame {}:", f.number);
        for a in &f.actors {
            println!("  actor {a:?}");
        }
        for p in &f.poses {
            let root = p.keys.iter().find(|k| k.0 == 0).map(|k| k.1);
            let yaw = root.map(|[_, y, _, w]| {
                ((-2.0 * (y as f64).atan2(w as f64) / std::f64::consts::TAU * 4096.0).round() as i32).rem_euclid(4096)
            });
            println!("  pose {} at {:?}, root {root:?} heading {yaw:?}", p.id, p.pos);
        }
    }
    Ok(())
}
