//! Runs the original to the title menu, then for N frames lists the RNG
//! state (0x800c3f94) and the two babies' objects (0x800c4b20, 0x800c4b24):
//! state, corner, which baby, position, its target, angles, their
//! targets, the speeds, and which hierarchy copy (an index into
//! 0x800e2440). With PARTS=1, also the first baby's part quaternions on the
//! last frame.
//!
//! cargo run -p rugrats-hle --example title_babies -- N

use std::rc::Rc;

use rrt::emu::BusExt;

fn main() {
    let frames: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(200);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    // The title is up once both babies exist.
    while hle.m.bus.read_u32(0x800c_4b20).unwrap() == 0 || hle.m.bus.read_u32(0x800c_4b24).unwrap() == 0 {
        hle.run_frame().unwrap();
    }
    println!("title at oracle frame {}", hle.frame);
    let mut last_counter = u32::MAX;
    let mut shown = 0;
    while shown < frames {
        hle.run_frame().unwrap();
        let b = &mut hle.m.bus;
        // One line a game frame: when the front end's frame counter moves.
        let counter = b.read_u32(0x800c_4080).unwrap();
        if counter == last_counter {
            continue;
        }
        last_counter = counter;
        shown += 1;
        let rng = b.read_u32(0x800c_3f94).unwrap();
        let mut line = format!("{counter:5} rng {rng:08x}");
        for at in [0x800c_4b20u32, 0x800c_4b24] {
            let o = b.read_u32(at).unwrap();
            let h = |b: &mut rugrats_hle::bus::PsxBus, k: u32| b.read_u16(o + k).unwrap() as i16;
            let w = |b: &mut rugrats_hle::bus::PsxBus, k: u32| b.read_u32(o + k).unwrap() as i32;
            let hier = b.read_u32(o + 0x154).unwrap();
            let index = (0..24).find(|&k| b.read_u32(0x800e_2440 + 4 * k).unwrap() == hier);
            line += &format!(
                " | st {} c {} k {} h {:?} pos {},{},{} tgt {},{},{} ang {},{},{} to {},{},{} sp {},{},{}",
                h(b, 0x5b2), h(b, 0x5b4), h(b, 0x5b8), index,
                w(b, 0x18), w(b, 0x1c), w(b, 0x20), w(b, 0x19c), w(b, 0x1a0), w(b, 0x1a4),
                h(b, 0x24), h(b, 0x26), h(b, 0x28), h(b, 0x2c), h(b, 0x2e), h(b, 0x30),
                h(b, 0x5aa), h(b, 0x5ac), h(b, 0x5b0)
            );
        }
        println!("{line}");
    }
    if std::env::var_os("PARTS").is_some() {
        let b = &mut hle.m.bus;
        let o = b.read_u32(0x800c_4b20).unwrap();
        let hier = b.read_u32(o + 0x154).unwrap();
        let n = b.read_u16(hier).unwrap() as i16;
        for k in 0..n.max(0) as u32 {
            let p = hier + 52 + 40 * k;
            let q = [4, 8, 12, 16].map(|a| b.read_u32(p + a).unwrap() as i32);
            println!("part {k} q {q:?}");
        }
    }
}
