//! Runs the original through the opening and writes the picture shown at
//! each cinematic frame counter (0x800c4c34) from FROM to TO into DIR as
//! N.png, N the stream frame shown (the counter less one).
//!
//! cargo run -p rugrats-hle --example cine_frames -- FROM TO DIR

use std::rc::Rc;

use rrt::emu::BusExt;

fn main() {
    let mut a = std::env::args().skip(1);
    let from: u16 = a.next().and_then(|s| s.parse().ok()).unwrap_or(380);
    let to: u16 = a.next().and_then(|s| s.parse().ok()).unwrap_or(700);
    let dir = a.next().unwrap_or_else(|| "work/hle/frames".into());
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    let mut last = 0;
    for _ in 0..12000 {
        let at = hle.m.bus.read_u16(0x800c_4c34).unwrap();
        hle.run_frame().unwrap();
        // The picture for a counter shows a tick after it is reached.
        if at != last && at > from && at <= to + 1 {
            let (w, h, rgba) = hle.screen();
            let path = format!("{dir}/{}.png", at - 1);
            std::fs::write(&path, rrt::image::png::encode(w as u32, h as u32, &rgba)).unwrap();
        }
        last = at;
        if at > to + 1 {
            break;
        }
    }
}
