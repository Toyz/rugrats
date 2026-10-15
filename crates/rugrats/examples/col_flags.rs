//! Counts the floor tiles carrying each flag bit, per level: `col_flags`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    for db in 1..=31 {
        let Ok(raw) = disc.file(466 + db) else { continue };
        let Ok(col) = rugrats_data::col::Col::parse(&raw) else { continue };
        let mut n = [0u32; 8];
        for c in &col.columns {
            for t in &c.tiles {
                for (b, v) in n.iter_mut().enumerate() {
                    *v += (t[4] >> b & 1) as u32;
                }
            }
        }
        println!("{db}: {n:?}");
    }
    Ok(())
}
