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

fn disc() -> Option<rugrats::disc::GameDisc> {
    let cue = rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()?;
    rugrats::disc::GameDisc::open(&cue).ok()
}

/// The cinematics' music bytes (0x800b8bd7 + 16 n): the golf cameras play
/// on (8), Golf Intro pauses and fades back (32), No More Cookies' stop.
#[test]
fn cinematics_keep_pause_or_stop_the_song() {
    let Some(disc) = disc() else { return };
    use rugrats::media::cine::music_flags;
    assert_eq!(music_flags(&disc, 29) & 40, 8);
    assert_eq!(music_flags(&disc, 34) & 40, 32);
    assert_eq!(music_flags(&disc, 64) & 40, 0);
}
