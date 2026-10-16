use rugrats::disc::GameDisc;
use rugrats_data::anim::Key;
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
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(1)).unwrap();
    for climb in [300, 600, 900] {
        g.player.animator.climb = climb;
        let Some(anm) = g.player.animator.anm(5) else {
            eprintln!("climb {climb}: no anm");
            continue;
        };
        let frames = g.anims.frames_of(anm);
        let lifts: Vec<i16> = frames
            .iter()
            .map(|f| {
                g.anims
                    .keys_of(f)
                    .iter()
                    .find_map(|k| match k {
                        Key::Lift { dy, .. } => Some(*dy),
                        _ => None,
                    })
                    .unwrap_or(i16::MIN)
            })
            .collect();
        eprintln!(
            "climb {climb}: anm {anm} frames {} durations {:?} lifts {:?} feet {}",
            frames.len(),
            frames.iter().map(|f| f.duration).collect::<Vec<_>>(),
            lifts,
            g.player.feet()
        );
    }
    for a in [3i16, 4, 15, 28] {
        let Some(anm) = g.player.animator.anm(a) else { continue };
        let frames = g.anims.frames_of(anm);
        let n = frames.iter().filter(|f| g.anims.keys_of(f).iter().any(|k| matches!(k, Key::Lift { .. }))).count();
        eprintln!("logical {a}: anm {anm} frames {} with lift {n}", frames.len());
    }
}
