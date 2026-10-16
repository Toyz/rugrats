//! Prints a template's 56-byte record: `template_bytes LEVEL TEMPLATE...`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let level: u8 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let game = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(level))?;
    for t in a.filter_map(|s| s.parse::<usize>().ok()) {
        if let Some(r) = game.set.templates.get(t) {
            let hex: Vec<String> = r.raw.iter().map(|b| format!("{b:02x}")).collect();
            println!("template {t}: shape {} flags {:#x} raw {}", r.shape, r.flags, hex.join(" "));
        }
    }
    Ok(())
}
