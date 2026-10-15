//! Prints a level's speech index (file 585 + DB): `speech_index DB [LINE
//! OUT.wav]`, and with LINE decodes that line to OUT.wav (44100 stereo).
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let raw = disc.file(585 + db)?;
    let n = u16::from_le_bytes([raw[0], raw[1]]) as usize;
    let ids: Vec<u16> = (0..n).map(|k| u16::from_le_bytes([raw[4 + 2 * k], raw[5 + 2 * k]])).collect();
    let at = 4 + 2 * n + (n & 1) * 2;
    println!("{n} lines; xa file {} lba {} size {}", 617 + db, disc.file_lba(617 + db)?, disc.file_size(617 + db)?);
    for (k, id) in ids.iter().enumerate() {
        let e = &raw[at + 4 * k..at + 4 * k + 4];
        println!("{:4} flags {:x}: offset {} len {} channel {}", id & 4095, id >> 12, u16::from_le_bytes([e[0], e[1]]), e[2], e[3]);
    }
    let mut a = std::env::args().skip(2);
    if let (Some(n), Some(out)) = (a.next().and_then(|s| s.parse().ok()), a.next()) {
        let mut speech = rugrats::speech::Speech::load(&disc, db);
        let buf = speech.line(&disc, n).ok_or("no such line")?;
        let peak = buf.iter().map(|s| s.0.unsigned_abs().max(s.1.unsigned_abs())).max().unwrap_or(0);
        println!("line {n}: {:.2} s, peak {peak}", buf.len() as f64 / 44100.0);
        let len = buf.len() as u32 * 4;
        let mut w = Vec::new();
        w.extend(b"RIFF");
        w.extend((36 + len).to_le_bytes());
        w.extend(b"WAVEfmt ");
        w.extend(16u32.to_le_bytes());
        w.extend(1u16.to_le_bytes());
        w.extend(2u16.to_le_bytes());
        w.extend(44100u32.to_le_bytes());
        w.extend((44100u32 * 4).to_le_bytes());
        w.extend(4u16.to_le_bytes());
        w.extend(16u16.to_le_bytes());
        w.extend(b"data");
        w.extend(len.to_le_bytes());
        for (l, r) in &buf {
            w.extend(l.to_le_bytes());
            w.extend(r.to_le_bytes());
        }
        std::fs::write(&out, w).map_err(|e| e.to_string())?;
    }
    Ok(())
}
