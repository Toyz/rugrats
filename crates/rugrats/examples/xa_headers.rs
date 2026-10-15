//! Prints XA subheaders: `xa_headers LBA COUNT`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let a: Vec<u32> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    for lba in a[0]..a[0] + a[1] {
        let r = disc.sector(lba)?.raw;
        println!("{lba}: file {} chan {} sub {:02x} coding {:02x}", r[16], r[17], r[18], r[19]);
    }
    Ok(())
}
