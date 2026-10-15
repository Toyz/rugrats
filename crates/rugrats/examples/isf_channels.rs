//! A database's ISF file (617 + DB): its sectors by XA file and channel,
//! with each channel's coding and sector count, and SPEECHTB.DAT's entries.
//!
//! cargo run -p rugrats --example isf_channels -- DB

use std::collections::BTreeMap;

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let start = disc.file_lba(617 + db)?;
    let size = disc.file_size(617 + db)?;
    let sectors = size.div_ceil(2048);
    let mut chans: BTreeMap<(u8, u8, u8, u8), (u32, u32, u32)> = BTreeMap::new();
    for k in 0..sectors {
        let s = disc.sector(start + k)?;
        let r = &s.raw;
        let e = chans.entry((r[16], r[17], r[18], r[19])).or_insert((k, k, 0));
        e.1 = k;
        e.2 += 1;
    }
    println!("ISF {}: {} sectors from LBA {start}", 617 + db, sectors);
    for (k, v) in &chans {
        println!(
            "  file {} chan {} sub {:02x} coding {:02x}: {} sectors, first {}, last {}",
            k.0, k.1, k.2, k.3, v.2, v.0, v.1
        );
    }
    if let Ok(t) = disc.file(585 + db) {
        let n = u16::from_le_bytes([t[0], t[1]]) as usize;
        let e: Vec<u16> = (0..n).map(|i| u16::from_le_bytes([t[4 + 2 * i], t[5 + 2 * i]])).collect();
        println!("SPEECHTB: {n} entries: {:x?}", &e[..n.min(40)]);
    }
    Ok(())
}
