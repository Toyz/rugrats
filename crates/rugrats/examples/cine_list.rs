//! Reads every cinematic's stream and prints its frames and sound length.
//!
//! cargo run -p rugrats --example cine_list

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    for n in 0..rugrats::media::cine::CINES {
        let name = disc.exe_string_at(0x800b_8bd0 + 16 * n as u32).unwrap_or_default();
        match rugrats::media::cine::Stream::read(&disc, n) {
            Ok(s) => println!(
                "{n:2} {name:28} {:5} frames to {:5.1} s, {:5.1} s sound",
                s.frames.len(),
                s.frames.last().map_or(0.0, |f| f.0 as f64 / 60.0),
                s.sound.len() as f64 / 44100.0
            ),
            Err(e) => println!("{n:2} {name:28} {e}"),
        }
    }
    Ok(())
}
