//! The title's babies against the original's, frame by frame, from the
//! oracle (rugrats-hle): their states, corners, positions, angles, speeds
//! and rand's state. Skipped without the disc.

use std::path::Path;
use std::rc::Rc;

use rrt::emu::BusExt;
use rugrats::rand::Rand;
use rugrats::ui::front::{Baby, InUse};

#[test]
fn babies_match_the_original() {
    let Ok(cue) = rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")) else {
        eprintln!("skipped: no disc");
        return;
    };
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    while hle.m.bus.read_u32(0x800c_4b20).unwrap() == 0 || hle.m.bus.read_u32(0x800c_4b24).unwrap() == 0 {
        hle.run_frame().unwrap();
    }
    let mut rng = Rand::default();
    // The front end's setup draws once (0x80048b78).
    rng.next();
    let mut babies = [Baby::default(), Baby::default()];
    let mut used = InUse::default();
    let mut counter = u32::MAX;
    let mut frames = 0;
    while frames < 150 {
        hle.run_frame().unwrap();
        let b = &mut hle.m.bus;
        let now = b.read_u32(0x800c_4080).unwrap();
        if now == counter {
            continue;
        }
        counter = now;
        frames += 1;
        for baby in &mut babies {
            baby.think(&mut used, &mut rng);
            baby.step();
        }
        assert_eq!(rng.state, b.read_u32(0x800c_3f94).unwrap(), "rand at frame {frames}");
        for (at, ours) in [0x800c_4b20u32, 0x800c_4b24].into_iter().zip(&babies) {
            let o = b.read_u32(at).unwrap();
            let h = |b: &mut rugrats_hle::bus::PsxBus, k: u32| b.read_u16(o + k).unwrap() as i16;
            let w = |b: &mut rugrats_hle::bus::PsxBus, k: u32| b.read_u32(o + k).unwrap() as i32;
            let theirs = Baby {
                state: h(b, 0x5b2),
                corner: h(b, 0x5b4),
                kind: h(b, 0x5b8),
                hierarchy: ours.hierarchy,
                pos: [w(b, 0x18), w(b, 0x1c), w(b, 0x20)],
                target: [w(b, 0x19c), w(b, 0x1a0), w(b, 0x1a4)],
                speed: [h(b, 0x5aa), h(b, 0x5ac), h(b, 0x5ae)],
                angles: [h(b, 0x24), h(b, 0x26), h(b, 0x28)],
                turn_to: [h(b, 0x2c), h(b, 0x2e), h(b, 0x30)],
                turn: h(b, 0x5b0),
            };
            let hier = b.read_u32(o + 0x154).unwrap();
            assert_eq!(
                b.read_u32(0x800e_2440 + 4 * ours.hierarchy as u32).unwrap(),
                hier,
                "frame {frames}: which baby"
            );
            assert_eq!(*ours, theirs, "frame {frames}");
        }
    }
}
