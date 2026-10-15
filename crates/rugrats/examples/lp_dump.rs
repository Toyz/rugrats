//! Prints a database's LP records of template T: `lp_dump DB T`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let db: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let t: u32 = a.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = rugrats::cine::Set::load(&disc, db)?;
    for (k, p) in set.placements.iter().enumerate().filter(|(_, p)| p.template == t) {
        let tp = set.templates.get(t as usize);
        println!(
            "LP {k}: kind {} pos {:?} id {} copies {} group {:?} class {:?} shape {:?} model found {:?}",
            p.kind,
            p.pos,
            p.id,
            p.copies,
            p.group,
            tp.map(|x| x.raw[4]),
            tp.map(|x| x.shape),
            tp.map(|x| set.models.find(x.shape as u32))
        );
    }
    Ok(())
}
