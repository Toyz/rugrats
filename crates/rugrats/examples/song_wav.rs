//! Renders a song of the menu's song set through the software SPU to a
//! WAV: SONG (default 0), SECONDS (default 30), OUT.
//!
//! cargo run -p rugrats --example song_wav -- SONG SECONDS OUT.wav

use rugrats::disc::GameDisc;
use rugrats::sound::{Loaded, MENU_SET, RATE, Spu};

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let song: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let secs: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(30);
    let out = a.next().unwrap_or("song.wav".into());
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = Loaded::load(&disc, MENU_SET)?;
    println!(
        "{} songs; music bank {} programs, {} tones",
        set.songs.len(),
        set.music.vab.programs.len(),
        set.music.vab.tones.len()
    );
    let mut spu = Spu::default();
    spu.set_banks(set.music.clone(), set.effects.clone());
    spu.play(set.songs.get(song).ok_or("no such song")?, 0, rugrats::sound::MUSIC_VOLUME);
    let mut buf = vec![0i16; 2 * RATE as usize * secs];
    spu.render(&mut buf);
    let peak = buf.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
    let clipped = buf.iter().filter(|s| s.unsigned_abs() >= 32767).count();
    let rms = (buf.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / buf.len() as f64).sqrt();
    println!("peak {peak}, rms {rms:.0}, clipped {clipped}");
    let mut w = Vec::new();
    let len = buf.len() as u32 * 2;
    w.extend(b"RIFF");
    w.extend((36 + len).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(2u16.to_le_bytes());
    w.extend(RATE.to_le_bytes());
    w.extend((RATE * 4).to_le_bytes());
    w.extend(4u16.to_le_bytes());
    w.extend(16u16.to_le_bytes());
    w.extend(b"data");
    w.extend(len.to_le_bytes());
    for s in &buf {
        w.extend(s.to_le_bytes());
    }
    std::fs::write(&out, w).map_err(|e| e.to_string())
}
