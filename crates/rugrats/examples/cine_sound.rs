//! Cinematic N's soundtrack: its length, and its loudness by half second
//! from FROM s to TO s.
//!
//! cargo run -p rugrats --example cine_sound -- N FROM TO

use rugrats::disc::GameDisc;
use rugrats::media::cine::Stream;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1).map(|s| s.parse::<f64>().unwrap_or(0.0));
    let (n, from, to) = (a.next().unwrap_or(0.0) as usize, a.next().unwrap_or(0.0), a.next().unwrap_or(60.0));
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = Stream::read(&disc, n)?;
    let rate = 44100.0;
    println!("{} frames, sound {:.2} s", s.frames.len(), s.sound.len() as f64 / rate);
    let mut t = from;
    while t < to {
        let (i, j) = ((t * rate) as usize, (((t + 0.5) * rate) as usize).min(s.sound.len()));
        if i >= j {
            break;
        }
        let peak = s.sound[i..j].iter().map(|&(l, r)| (l as i32).abs().max((r as i32).abs())).max().unwrap_or(0);
        println!("{t:5.1} s peak {peak}");
        t += 0.5;
    }
    Ok(())
}
