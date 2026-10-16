//! The game's sound banks against rrt's libsnd (skipped without work/fs).

use std::path::Path;

use rrt::kit::libsnd::vab::Vab;

/// Every bank on the disc parses, with as many tones as its header says.
/// Nine banks list programs past the header's count (worklog 72).
#[test]
fn every_bank_parses() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/DATA/SOUND/VAB");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut n = 0;
    for e in entries.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "VH")) {
        let vh = std::fs::read(e.path()).unwrap();
        let v = Vab::parse(&vh).unwrap_or_else(|err| panic!("{:?}: {err}", e.path()));
        assert_eq!(v.tones.len(), u16::from_le_bytes([vh[20], vh[21]]) as usize, "{:?}", e.path());
        n += 1;
    }
    assert!(n > 10, "only {n} banks");
}
