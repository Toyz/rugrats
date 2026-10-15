//! libgte's trigonometry and the camera's projection distance against the
//! original's functions in rrt-emu. Skipped without the disc.

use std::path::Path;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;

#[test]
fn trigonometry_and_projection_match_the_original() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let (Ok(bytes), Ok(cue)) = (std::fs::read(root.join("work/fs/SLUS_006.50")), rrt::disc::Image::find(&root.join("work/disc"))) else {
        eprintln!("skipped: no disc");
        return;
    };
    let trig = Trig::load(&GameDisc::open(&cue).unwrap()).unwrap();
    let mut m = machine_with_exe(&PsxExe::parse(&bytes).unwrap()).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    for a in (-5000..5000).step_by(7) {
        assert_eq!(trig.sin(a), m.call(0x8009_efd8, &[a as u32 as u64]).unwrap() as i32, "rsin({a})");
        assert_eq!(trig.cos(a), m.call(0x8009_f0a8, &[a as u32 as u64]).unwrap() as i32, "rcos({a})");
    }
    let cam = 0x8010_0000u32;
    for fov in 2..2048 {
        m.bus.write_u16(cam + 0x42, 0).unwrap();
        m.call(0x8001_dfb8, &[cam as u64, fov as u64]).unwrap();
        let theirs = m.bus.read_u16(cam + 0x42).unwrap() as i16 as i32;
        assert_eq!(trig.geom_screen(fov), Some(theirs), "fov {fov}");
    }
    assert_eq!(trig.geom_screen(955), Some(178));
}

/// 0x80020ee0's matrix from angles, before MatrixNormal (0x8009f398): the
/// port's against the original with MatrixNormal stubbed out.
#[test]
fn euler_matrices_match_the_original() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let (Ok(bytes), Ok(cue)) = (std::fs::read(root.join("work/fs/SLUS_006.50")), rrt::disc::Image::find(&root.join("work/disc"))) else {
        eprintln!("skipped: no disc");
        return;
    };
    let trig = Trig::load(&GameDisc::open(&cue).unwrap()).unwrap();
    let mut m = machine_with_exe(&PsxExe::parse(&bytes).unwrap()).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    m.stub(0x8009_f398, 0);
    let (at, out) = (0x8010_0000u32, 0x8010_0100u32);
    for angles in [[0i16, 0, 0], [1024, 0, 0], [0, 2048, 0], [0, 0, 512], [300, -700, 1500], [2048, 1024, -100], [-4000, 77, 3333]] {
        for (k, a) in angles.iter().enumerate() {
            m.bus.write_u16(at + 2 * k as u32, *a as u16).unwrap();
        }
        m.call(0x8002_0ee0, &[at as u64, out as u64]).unwrap();
        let theirs: Vec<i32> = (0..9).map(|k| m.bus.read_u16(out + 2 * k).unwrap() as i16 as i32).collect();
        let ours: Vec<i32> = trig.euler(angles).iter().flatten().map(|&v| v as i16 as i32).collect();
        assert_eq!(ours, theirs, "angles {angles:?}");
    }
}
