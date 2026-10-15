//! Lists a database's placements with a speech line and their trigger
//! radius (template +0x34): `speech_spots DB`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = rugrats::cine::Set::load(&disc, db)?;
    for (k, p) in set.placements.iter().enumerate().filter(|(_, p)| p.kind == 1 && (p.speech != 0 || p.template == 613)) {
        let r = set.templates.get(p.template as usize).map_or(0, |t| u16::from_le_bytes([t.raw[0x34], t.raw[0x35]]));
        println!("LP {k}: template {} at {:?} line {} radius {r}", p.template, p.pos, p.speech);
    }
    Ok(())
}
