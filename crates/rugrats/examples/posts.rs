//! Prints a database's posts (LP kind 2): `posts DB`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = rugrats::cine::Set::load(&disc, db)?;
    for (k, p) in rugrats::npc::posts(&set.placements).iter().enumerate() {
        println!("post {}: {:?} next {:?} flags {:#x}", k + 1, p.pos, p.next, p.flags);
    }
    Ok(())
}
