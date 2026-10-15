//! The renderer's maths against the original's functions, called in rrt-emu.

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};

fn exe() -> Option<PsxExe> {
    let work = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/SLUS_006.50");
    PsxExe::parse(&std::fs::read(work).ok()?).ok()
}

/// quat_matrix matches 0x8001fc48 (quaternion to 4.12 matrix, row-major
/// m[3][3]) within a unit of rounding, for quaternions at the scales the
/// keys use.
#[test]
fn quaternion_matrices_match_the_original() {
    let Some(exe) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    let (q_at, out_at) = (0x8010_0000u32, 0x8010_0100u32);
    for q in [[0i16, 0, 0, 2048], [1000, 0, 0, 1800], [0, -1500, 0, 1200], [300, 700, -900, 1500], [-2000, 1500, 800, 300]] {
        for (k, v) in q.iter().enumerate() {
            m.bus.write_u32(q_at + 4 * k as u32, *v as i32 as u32).unwrap();
        }
        m.call(0x8001_fc48, &[q_at as u64, out_at as u64]).unwrap();
        let theirs: Vec<i16> = (0..9).map(|k| m.bus.read_u16(out_at + 2 * k).unwrap() as i16).collect();
        let ours = rugrats_render::models::quat_matrix(q);
        for r in 0..3 {
            for c in 0..3 {
                let o = (ours.col(c)[r] * 4096.0).round() as i32;
                let t = theirs[r * 3 + c] as i32;
                assert!((o - t).abs() <= 8, "q {q:?} m[{r}][{c}]: ours {o} theirs {t} (all theirs {theirs:?})");
            }
        }
    }
}
