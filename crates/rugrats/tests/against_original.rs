//! The port's screens against the original's, run in the oracle
//! (rugrats-hle) from the same disc. Skipped without a disc in work/disc.

use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;

fn cue() -> Option<std::path::PathBuf> {
    rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()
}

/// The legal screen the port draws is the one the original displays, pixel
/// for pixel, while show_still holds it (SLUS_006.50:0x80030100).
#[test]
fn the_legal_screen_matches_the_original() {
    let Some(cue) = cue() else {
        eprintln!("skipped: no disc");
        return;
    };
    let disc = GameDisc::open(&cue).unwrap();
    let ours = rugrats::still::picture(&disc.file(234).unwrap(), &disc.mdec).unwrap();

    let image = Rc::new(rrt::disc::Image::open(&cue).unwrap());
    let mut hle = rugrats_hle::Hle::new(image, true).unwrap();
    let mut matched = false;
    let mut held = 0;
    for _ in 0..300 {
        hle.run_frame().unwrap();
        // show_still's hold loop: VSync(0) returns to 0x80030120. Its first
        // frames still have the drawing in libgpu's queue.
        if rrt::emu::Cpu::pc(&hle.m.cpu) != 0x8003_0120 {
            continue;
        }
        held += 1;
        if held < 30 {
            continue;
        }
        let (w, h, rgba) = hle.screen();
        assert_eq!((w, h), (512, 240));
        let differ = (0..w * h)
            .filter(|&i| ours.get((i % w) as u32, (i / w) as u32).unwrap()[..3] != rgba[i * 4..i * 4 + 3])
            .count();
        if differ != 0 {
            let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/port");
            let _ = std::fs::create_dir_all(&out);
            let _ = std::fs::write(out.join("legal-ours.png"), ours.to_png());
            let _ = std::fs::write(out.join("legal-original.png"), rrt::image::png::encode(w as u32, h as u32, &rgba));
            let diffs: Vec<usize> = (0..w * h)
                .filter(|&i| ours.get((i % w) as u32, (i / w) as u32).unwrap()[..3] != rgba[i * 4..i * 4 + 3])
                .collect();
            for &i in diffs.iter().step_by((diffs.len() / 12).max(1)).take(12) {
                let o = ours.get((i % w) as u32, (i / w) as u32).unwrap();
                eprintln!("({}, {}) ours {:?} theirs {:?}", i % w, i / w, &o[..3], &rgba[i * 4..i * 4 + 3]);
            }
        }
        assert_eq!(differ, 0, "{differ} pixels differ at frame {}", hle.frame);
        matched = true;
        break;
    }
    assert!(matched, "the original never showed a 512 x 240 picture");
}
