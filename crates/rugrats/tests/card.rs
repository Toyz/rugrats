//! The memory card file the port writes, against the original's code run
//! in rrt-emu, and against the file the oracle's card folder holds when the
//! original made it (rugrats-hle's card_save example). Skipped without the
//! extracted executable or the disc.

use std::path::Path;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};
use rugrats::card::{self, Art, Save};

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn exe() -> Option<(PsxExe, Art)> {
    let bytes = std::fs::read(root().join("work/fs/SLUS_006.50")).ok()?;
    let exe = PsxExe::parse(&bytes).ok()?;
    let art = Art::from_exe(&bytes[0x800..])?;
    Some((exe, art))
}

/// Every byte's Shift-JIS is 0x80070e84's.
#[test]
fn shift_jis_matches_the_original() {
    let Some((exe, art)) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    for c in 0..=255u8 {
        let theirs = m.call(0x8007_0e84, &[c as u64]).unwrap() as u16;
        assert_eq!(art.sjis(c), theirs, "{c:#x}");
    }
}

/// The title frame is 0x80071ee0's for the save file.
#[test]
fn the_title_frame_matches_the_original() {
    let Some((exe, art)) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    let (buf, name) = (0x8010_0000u32, 0x8010_0400u32);
    for k in 0..card::TITLE_SIZE as u32 {
        m.bus.write_u8(buf + k, 0).unwrap();
    }
    for (k, b) in b"Games\0".iter().enumerate() {
        m.bus.write_u8(name + k as u32, *b).unwrap();
    }
    m.call(0x8007_1ee0, &[buf as u64, name as u64, 1]).unwrap();
    let theirs: Vec<u8> = (0..card::TITLE_SIZE as u32).map(|k| m.bus.read_u8(buf + k).unwrap()).collect();
    assert_eq!(art.title_frame("Games", 1).to_vec(), theirs);
}

/// Sealing gives the sums 0x8007062c writes, and the load's check
/// (0x80070234) agrees with the port's on good and spoilt saves.
#[test]
fn the_sums_match_the_original() {
    let Some((exe, _)) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    const AT: u32 = 0x800e_3c98;
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    let mut save = Save::default();
    let mut x = 0x1234_5678u32;
    for w in save.words.iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *w = x;
    }
    for (k, w) in save.words.iter().enumerate() {
        m.bus.write_u32(AT + 4 * k as u32, *w).unwrap();
    }
    // 0x8007062c seals, then asks for the card's status (0x8007146c,
    // stubbed to none) and stops.
    m.stub(0x8007_146c, 0);
    m.call(0x8007_062c, &[0]).unwrap();
    let theirs: Vec<u32> = (0..card::WORDS as u32).map(|k| m.bus.read_u32(AT + 4 * k).unwrap()).collect();
    save.seal();
    assert_eq!(save.words, theirs);
    assert!(save.check());
    assert_eq!(m.call(0x8007_0234, &[]).unwrap(), 1);
    for spoil in [0usize, 1, 90, 1379] {
        let mut s = save.clone();
        s.words[spoil] ^= 0x100;
        m.bus.write_u32(AT + 4 * spoil as u32, s.words[spoil]).unwrap();
        assert_eq!(m.call(0x8007_0234, &[]).unwrap() == 1, s.check(), "word {spoil}");
        m.bus.write_u32(AT + 4 * spoil as u32, save.words[spoil]).unwrap();
    }
}

/// The empty save the port writes is the file the original made in the
/// oracle's card folder, byte for byte (the last frame's tail included).
#[test]
fn the_empty_save_file_matches_the_originals() {
    let Some((_, art)) = exe() else {
        eprintln!("skipped: no work/fs");
        return;
    };
    let name = art.file_name(card::FILE);
    let Ok(theirs) = std::fs::read(root().join("work/oracle/memcard/slot1").join(&name)) else {
        eprintln!("skipped: no file from the oracle (cargo run -p rugrats-hle --example card_save)");
        return;
    };
    let dir = root().join("work/test/card");
    let _ = std::fs::remove_dir_all(&dir);
    let cards = card::Cards::at(&dir.join("slot1"), &dir.join("slot2"));
    cards.store(0, &art, &Save::default()).unwrap();
    let ours = std::fs::read(dir.join("slot1").join(&name)).unwrap();
    assert_eq!(ours.len(), theirs.len());
    let first = ours.iter().zip(&theirs).position(|(a, b)| a != b);
    assert_eq!(first, None, "first difference at {first:?}");
    assert!(cards.load(0, &art).is_some_and(|s| s.check()));
    assert!(!cards.present(1));
}

/// A game state put in a record comes back from the file's bytes, and the
/// sums hold.
#[test]
fn a_record_round_trips() {
    let mut p = rugrats::progress::Progress::default();
    p.enter(2);
    p.find_bar(2, 3);
    p.find_bar(2, 6);
    let mut save = rugrats::card::Save::default();
    save.put_record(1, &p.block);
    assert!(save.check());
    let back = rugrats::card::Save::from_bytes(&save.to_bytes()).unwrap();
    assert_eq!(back.record(1), Some(p.block));
    assert_eq!(back.words[3], 1);
    let q = rugrats::progress::Progress { block: back.record(1).unwrap() };
    assert!(q.bar_found(2, 3) && q.bar_found(2, 6) && !q.bar_found(2, 4));
    assert_eq!(q.bars_available(), 2);
    assert_eq!(q.level(), 2);
}
