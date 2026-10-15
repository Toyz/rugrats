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

/// The camera's quaternion-to-matrix (0x80020580), used by the cinematic
/// stream, gives the transpose of the parts' (0x8001fc48).
#[test]
fn camera_quaternion_matrices() {
    let Some(exe) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    let (q_at, out_at) = (0x8010_0000u32, 0x8010_0100u32);
    for q in [[1000i16, 0, 0, 1800], [300, 700, -900, 1500]] {
        for (k, v) in q.iter().enumerate() {
            m.bus.write_u32(q_at + 4 * k as u32, *v as i32 as u32).unwrap();
        }
        m.call(0x8002_0580, &[q_at as u64, out_at as u64]).unwrap();
        let theirs: Vec<i16> = (0..9).map(|k| m.bus.read_u16(out_at + 2 * k).unwrap() as i16).collect();
        let ours = rugrats_render::models::quat_matrix(q);
        let transposed = (0..9).all(|k| ((ours.col(k / 3)[k % 3] * 4096.0).round() as i32 - theirs[k] as i32).abs() <= 8);
        assert!(transposed, "q {q:?}: theirs {theirs:?}");
    }
}

/// Models::get finds the model 0x80044b74 returns, for every id of every
/// MOD file, the ids a file lacks included.
#[test]
fn model_lookup_matches_the_original() {
    let Some(exe) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/DATA");
    let mut files = 0;
    for db in 0..32 {
        let Ok(bytes) = std::fs::read(data.join(format!("DB{db:02}/DB{db:02}MOD.BIN"))) else { continue };
        let Ok(models) = rugrats_data::model::Models::parse(&bytes) else { continue };
        let mut m = machine_with_exe(&exe).unwrap();
        m.cpu.r[28] = 0x800c_3e80;
        let at = 0x8010_0000u32;
        for (k, b) in bytes.iter().enumerate() {
            m.bus.write_u8(at + k as u32, *b).unwrap();
        }
        let count = u16::from_le_bytes([bytes[4], bytes[5]]) as u32;
        let base = at + 8 + 4 * count;
        // The table, its count and the models' base, as load_models leaves them.
        m.bus.write_u32(0x800c_412c, at).unwrap();
        m.bus.write_u16(0x800c_4130, count as u16).unwrap();
        m.bus.write_u32(0x800c_4b6c, base).unwrap();
        for id in 0..2048u32 {
            let theirs = m.call(0x8004_4b74, &[id as u64]).unwrap() as u32;
            let ours = models.get(id).map(|o| o as *const _);
            let want = models.at_offset(theirs - base).map(|o| o as *const _);
            assert_eq!(ours, want, "DB{db:02} id {id}: the original gives offset {}", theirs - base);
        }
        files += 1;
    }
    assert!(files >= 28, "{files} files");
}

/// The matrix an object's rotation ends up as (object +0x04, which the
/// object transform 0x8001eb4c transposes): for a placed object, LP's
/// quaternion inverted (0x8001f684), turned into angles (0x800202b8) and
/// back (0x80020ee0) gives quat_matrix(q) for a turn about one axis; a
/// jointed object's base (0x80020d10 of the same angles) and a cinematic
/// actor's (0x8001f9fc) give its transpose.
#[test]
fn object_rotations() {
    let Some(exe) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    let (q_at, inv_at, angles_at, out_at) = (0x8010_0000u32, 0x8010_0040u32, 0x8010_0080u32, 0x8010_0100u32);
    let read = |m: &mut rrt::emu::Machine<_>| -> Vec<i32> { (0..9).map(|k| m.bus.read_u16(out_at + 2 * k).unwrap() as i16 as i32).collect() };
    let ours = |q: [i16; 4], t: bool| -> Vec<i32> {
        let r = rugrats_render::models::quat_matrix(q);
        let r = if t { r.transpose() } else { r };
        (0..9).map(|k| (r.col(k % 3)[k / 3] * 4096.0).round() as i32).collect()
    };
    let close = |a: &[i32], b: &[i32]| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 24);
    for q in [[0i16, 2896, 0, 2896], [0, -1567, 0, 3784], [0, 0, -1567, 3784], [1000, 0, 0, 3972], [0, 4080, 0, -356]] {
        for (k, v) in q.iter().enumerate() {
            m.bus.write_u32(q_at + 4 * k as u32, *v as i32 as u32).unwrap();
        }
        m.call(0x8001_f684, &[q_at as u64, inv_at as u64]).unwrap();
        m.call(0x8002_02b8, &[inv_at as u64, angles_at as u64]).unwrap();
        m.call(0x8002_0ee0, &[angles_at as u64, out_at as u64]).unwrap();
        let placed = read(&mut m);
        assert!(close(&placed, &ours(q, false)), "placed q {q:?}: {placed:?}");
        m.call(0x8002_0d10, &[angles_at as u64, out_at as u64]).unwrap();
        let jointed = read(&mut m);
        assert!(close(&jointed, &ours(q, true)), "jointed q {q:?}: {jointed:?}");
        m.call(0x8001_f9fc, &[q_at as u64, out_at as u64]).unwrap();
        let actor = read(&mut m);
        assert!(close(&actor, &ours(q, true)), "actor q {q:?}: {actor:?}");
    }
}
