//! The program as the window runs it: Rugrats::tick with scripted pads, one
//! tick at a time (rrt's Tick::scripted), from the legal screen on. Skipped
//! without a disc in work/disc.

use std::path::Path;
use std::time::Duration;

use rrt::prelude::{Buttons, Game, Pad, Tick};
use rugrats::app::Rugrats;
use rugrats::disc::GameDisc;
use rugrats::levels::Level;

fn app() -> Option<Rugrats> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc");
    let disc = GameDisc::open(&rrt::disc::Image::find(&dir).ok()?).ok()?;
    Rugrats::new(disc, false).ok()
}

/// Runs `ticks` ticks at 60 Hz; `presses` are (tick, button, ticks held).
fn run(app: &mut Rugrats, ticks: u64, presses: &[(u64, Buttons, u64)]) {
    let dt = Duration::from_secs_f64(1.0 / 60.0);
    let mut previous = Pad::default();
    for frame in 0..ticks {
        let buttons = presses
            .iter()
            .filter(|&&(at, _, held)| (at..at + held).contains(&frame))
            .fold(Buttons::default(), |a, &(_, b, _)| a | b);
        let pad = Pad { buttons, ..Pad::default() };
        let mut t = Tick::scripted(pad, previous, frame, dt);
        app.tick(&mut t);
        assert!(!t.exiting(), "the program ended at tick {frame}");
        previous = pad;
    }
}

/// Logos skipped, Start Game, Main Game, Reptar Puzzle's cinematic and
/// its cards: Tommy in Pickles' Home.
#[test]
fn a_new_game_reaches_the_hub() {
    let Some(mut app) = app() else { return };
    let (x, s, t) = (Buttons::CROSS, Buttons::START, Buttons::TRIANGLE);
    let mut presses = vec![(200, x, 4), (260, x, 4), (320, x, 4), (380, s, 4)];
    presses.extend([440, 500, 560, 620, 700, 1000, 1100, 1300].map(|f| (f, x, 4)));
    presses.extend([800, 900, 1200].map(|f| (f, t, 4)));
    run(&mut app, 2400, &presses);
    assert_eq!(app.level_in_play(), Some(Level::HUB));
}

/// Activities, Cookie Race: a level whose song bank once crashed the
/// program as it opened.
#[test]
fn activities_open_cookie_race() {
    let Some(mut app) = app() else { return };
    let (x, s, d, t) = (Buttons::CROSS, Buttons::START, Buttons::DOWN, Buttons::TRIANGLE);
    // Triangle skips the race's start cinematic.
    let presses = [
        (200, x, 4),
        (260, x, 4),
        (320, x, 4),
        (380, s, 4),
        (500, x, 6),
        (620, d, 6),
        (700, x, 6),
        (820, x, 6),
        (1100, t, 6),
    ];
    run(&mut app, 1600, &presses);
    assert_eq!(app.level_in_play(), Some(Level::COOKIE_RACE));
}
