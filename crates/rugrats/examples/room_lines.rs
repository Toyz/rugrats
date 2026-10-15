//! Lists the rooms whose flag 128 says a speech line (+0x64 of 5 and
//! up): `room_lines DB`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = rugrats::cine::Set::load(&disc, db)?;
    for (k, r) in set.room_info.iter().enumerate().filter(|(_, r)| r.flags & 128 != 0) {
        println!("room {k}: +0x64 {}", r.song);
    }
    Ok(())
}
