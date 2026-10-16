use rugrats::disc::GameDisc;
use std::path::Path;
#[test]
fn probe() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc/");
    let cue = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "cue"))
        .unwrap();
    let disc = GameDisc::open(&cue).unwrap();
    let g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(6)).unwrap();
    for t in [926u32, 999, 1000, 1006, 1046, 1047] {
        let tp = &g.set.templates[t as usize];
        let n = g.set.placements.iter().filter(|p| p.kind == 1 && p.template == t).count();
        let at = g.set.placements.iter().find(|p| p.kind == 1 && p.template == t).map(|p| p.pos);
        eprintln!(
            "template {t}: class {} touch {} flags {:#x} pickable {} placed {n} at {:?} shape {}",
            tp.raw[4],
            tp.raw[0x0a],
            tp.flags,
            tp.pickable(),
            at,
            tp.shape
        );
    }
}
