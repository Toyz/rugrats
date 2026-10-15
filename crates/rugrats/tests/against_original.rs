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
    let ours = rugrats::media::still::picture(&disc.file(234).unwrap(), &disc.mdec).unwrap();

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

/// Every logo-movie frame the original handed the MDEC (oracle dumps after
/// the legal screen, `RUGRATS_MDEC_DUMP`) is one the port decodes, word
/// for word, and the port decodes every frame of the three logos.
#[test]
fn the_logo_movies_decode_as_the_original_decodes_them() {
    let Some(cue) = cue() else {
        eprintln!("skipped: no disc");
        return;
    };
    let dumps = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/oracle/mdec");
    let Ok(listing) = std::fs::read_dir(&dumps) else {
        eprintln!("skipped: no oracle dumps");
        return;
    };
    let disc = GameDisc::open(&cue).unwrap();
    let mut ours = std::collections::HashSet::new();
    for n in 0..3 {
        let e = disc.movies[n];
        let mut frames = rugrats_data::movie::Frames::default();
        let (mut lba, mut got) = (e.lba, 0);
        while got < e.frames {
            let s = disc.sector(lba).unwrap();
            lba += 1;
            let Some(f) = rugrats_data::movie::chunk(&s.raw[..]).and_then(|c| frames.push(c)) else { continue };
            got += 1;
            match rugrats_data::bs::rle(&f.bs) {
                Ok(rle) => {
                    ours.insert(rle);
                }
                Err(err) => panic!("movie {n} frame {}: {err}", f.number),
            }
        }
    }
    let mut checked = 0;
    for entry in listing.flatten() {
        let dump = std::fs::read(entry.path()).unwrap();
        // 0x38xxxxxx is a still (15-bit); movies decode at 24 bits.
        if dump[3] != 0x32 {
            continue;
        }
        let theirs: Vec<u16> = dump[4..].chunks(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
        assert!(ours.contains(&theirs), "{} is no frame the port decodes", entry.path().display());
        checked += 1;
    }
    eprintln!("{checked} movie frames match");
}
