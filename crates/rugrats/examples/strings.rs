//! Prints the game's strings (0x800bb4e8 table): `strings FROM TO`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let a: Vec<u32> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    for n in a[0]..=a[1] {
        println!("{n}: {:?}", rugrats::ui::front::string(&disc, n));
    }
    Ok(())
}
