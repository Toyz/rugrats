//! Behaviour the port must keep while it changes: levels played headless
//! with scripted pads, as the play_script example runs them. Skipped
//! without a disc in work/disc.

use std::path::Path;

use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;

fn disc() -> Option<GameDisc> {
    let cue = rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()?;
    GameDisc::open(&cue).ok()
}

/// Steps of (buttons, frames).
fn run(g: &mut Game, steps: &[(&[usize], usize)], mut each: impl FnMut(&mut Game, usize)) {
    let mut pads = Pads::default();
    let mut frame = 0;
    for &(buttons, n) in steps {
        let bits = buttons.iter().fold(0u16, |a, &b| a | 1 << b);
        for _ in 0..n {
            pads.push_bits(0, bits);
            g.frame(&pads);
            each(g, frame);
            frame += 1;
        }
    }
}

fn open(disc: &GameDisc, level: usize) -> Game {
    Game::open_from(disc, level, None, &Progress::default()).unwrap()
}

fn put(g: &mut Game, at: [i32; 3], heading: i32) {
    g.player.pos = at;
    g.player.heading = heading;
    g.player.target = heading;
    g.set_camera_yaw(heading as f32);
}

/// Level 13's race: Angelica walks her posts to the goal and cine 20.
#[test]
fn level_13_race_reaches_its_goal() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 13);
    let mut cine = None;
    run(&mut g, &[(&[], 3000)], |g, _| cine = cine.or(g.cine.take()));
    assert_eq!(cine, Some(20));
}

/// Every hub door from its spot (or beside it) leads to its level.
#[test]
fn hub_doors_lead_to_their_levels() {
    let Some(disc) = disc() else { return };
    let doors: [([i32; 3], usize); 9] = [
        ([16597, -5365, 14246], 11),
        ([34864, -5365, -5324], 5),
        ([22563, -5365, -6729], 13),
        ([37002, 779, 9866], 2),
        ([26808, 779, -8518], 25),
        ([38751, -5365, 17399], 12),
        ([9330, 779, 20900], 10),
        ([1300, 779, -2713], 18),
        ([22800, 7947, 2900], 9),
    ];
    for (at, level) in doors {
        let mut g = open(&disc, 1);
        let h = g.player.heading;
        put(&mut g, at, h);
        let mut exit = None;
        run(&mut g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 3)], |g, _| exit = exit.or(g.exit.take()));
        assert_eq!(exit.map(|e| e.0), Some(level), "door at {at:?}");
    }
}

/// The hub's back door slides open and lets Tommy into the yard.
#[test]
fn hub_back_door_opens_to_the_yard() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    put(&mut g, [7155, 779, 22000], 0);
    run(&mut g, &[(&[], 2), (&[input::UP], 40)], |_, _| {});
    assert!(g.player.pos[2] < 23500, "the shut door stops him");
    run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 40), (&[input::UP], 60)], |_, _| {});
    assert!(g.player.pos[2] > 26000, "through the open door: {:?}", g.player.pos);
}

/// The puzzle board goes back to Reptar 2010 once it has begun.
#[test]
fn board_returns_to_level_7() {
    let Some(disc) = disc() else { return };
    let mut p = Progress::default();
    p.set_flags(p.flags() | 512);
    let mut g = Game::open_from(&disc, 1, None, &p).unwrap();
    put(&mut g, [3103, 779, 1200], 2048);
    let mut exit = None;
    run(&mut g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 3)], |g, _| exit = exit.or(g.exit.take()));
    assert_eq!(exit.map(|e| e.0), Some(7));
}

/// Golf holes do not hurt (Damage is dropped in 23-31).
#[test]
fn golf_hazards_do_not_hurt() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 25);
    put(&mut g, [31966, -4719, 9241], 0);
    run(&mut g, &[(&[input::UP], 120)], |_, _| {});
    assert_eq!(g.player.health, 255);
}

/// Level 6: four boxes stacked on the -2048 step reach the switch
/// platform (-4736).
#[test]
fn level_6_box_stack_reaches_the_platform() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    let boxes: Vec<usize> = (0..g.things.len()).filter(|&k| g.things[k].template == 1016).take(4).collect();
    for (i, &k) in boxes.iter().enumerate() {
        g.things[k].pos = rrt::glam::Vec3::new(44500.0, -2304.0 - 506.0 * i as f32, 13900.0);
    }
    put(&mut g, [44500, -2293, 12900], 0);
    run(&mut g, &[(&[], 3), (&[input::UP], 60)], |_, _| {});
    assert!(g.player.pos[2] > 14336 && g.player.pos[1] < -4800, "on the platform: {:?}", g.player.pos);
}

/// Level 16's pie toss: five hits on Angelica win it.
#[test]
fn level_16_toss_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 16);
    put(&mut g, [31600, -1750, -8141], 1024);
    let mut steps: Vec<(&[usize], usize)> = vec![(&[input::UP], 15), (&[], 330)];
    for _ in 0..60 {
        steps.push((&[input::UP, input::TRIANGLE], 1));
        steps.push((&[input::UP], 14));
    }
    // Won before the clock (30 s of unfrozen time) could run out: the
    // timer goes off with Angelica's health back to full.
    let mut ended = None;
    run(&mut g, &steps, |g, f| {
        if ended.is_none() && f > 20 && g.timer.is_none() {
            ended = Some(f);
        }
    });
    assert!(ended.is_some_and(|f| f < 1000), "won at {ended:?}");
}

/// Every reachable level opens and plays a few seconds of running about.
#[test]
fn every_level_runs() {
    let Some(disc) = disc() else { return };
    for level in [1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 23, 24, 25, 26, 27, 28, 29, 30, 31] {
        let mut g = open(&disc, level);
        run(&mut g, &[(&[], 5), (&[input::UP], 60), (&[input::CROSS], 1), (&[input::LEFT], 40), (&[], 100)], |_, _| {});
    }
}
