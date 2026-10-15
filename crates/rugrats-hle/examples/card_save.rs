//! Runs the original to frame N (the title menu), then calls its "create
//! the save file" path (0x800707a8: the file `Games` on the card in slot 1,
//! its title frame, and the first write of the save) on top of it, so the
//! oracle's card folder holds the file exactly as the game writes it.
//!
//! cargo run -p rugrats-hle --example card_save -- [N]

use std::rc::Rc;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let frames: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1700);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    while hle.frame < frames {
        hle.run_frame().unwrap();
    }
    let r = hle.call_nested(0x8007_07a8, &[0]).unwrap();
    println!("0x800707a8(0) = {r}");
}
