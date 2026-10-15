//! Prints executable words: `exe_words ADDR COUNT` (hex address).
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let addr =
        u32::from_str_radix(a.next().unwrap_or_default().trim_start_matches("0x"), 16).map_err(|e| e.to_string())?;
    let n: u32 = a.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    for k in 0..n {
        let w = disc.exe_u32(addr + 4 * k)?;
        println!("{:08x}: {w:08x} ({})", addr + 4 * k, w as i32);
    }
    Ok(())
}
