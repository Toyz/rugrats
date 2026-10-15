//! The ANM animation each state plays for the level's player:
//! `anim_states DB`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let db: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let game = rugrats::world::game::Game::open(&disc, db)?;
    for a in 0..60i16 {
        if let Some(id) = game.player.anm(a) {
            println!("state {a}: anm {id}, {} frames", game.anims.frames_of(id).len());
        }
    }
    Ok(())
}
