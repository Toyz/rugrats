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

/// The program driven at 60 Hz, tick by tick.
struct Driver {
    app: Rugrats,
    frame: u64,
    previous: Pad,
}

impl Driver {
    fn tick(&mut self, buttons: Buttons) {
        let pad = Pad { buttons, ..Pad::default() };
        let mut t = Tick::scripted(pad, self.previous, self.frame, Duration::from_secs_f64(1.0 / 60.0));
        self.app.tick(&mut t);
        assert!(!t.exiting(), "the program ended at tick {}", self.frame);
        self.previous = pad;
        self.frame += 1;
    }

    /// One press of `b` (4 ticks), then 56 idle.
    fn press(&mut self, b: Buttons) {
        for k in 0..60 {
            self.tick(if k < 4 { b } else { Buttons::default() });
        }
    }

    /// Waits `n` ticks.
    fn wait(&mut self, n: u64) {
        for _ in 0..n {
            self.tick(Buttons::default());
        }
    }

    /// Presses `b` (held 4 ticks, then 26 idle) until `done`, at most
    /// `tries` times.
    fn press_until(&mut self, b: Buttons, tries: usize, done: impl Fn(&Rugrats) -> bool) {
        for _ in 0..tries {
            if done(&self.app) {
                return;
            }
            for k in 0..30 {
                self.tick(if k < 4 { b } else { Buttons::default() });
            }
        }
        assert!(done(&self.app), "stuck in {} at tick {}", self.app.state(), self.frame);
    }
}

/// From the legal screen through the logos and the opening (each skipped)
/// to the title, settled.
fn to_title() -> Option<Driver> {
    let mut d = Driver { app: app()?, frame: 0, previous: Pad::default() };
    d.press_until(Buttons::TRIANGLE, 60, |a| a.state() == "title");
    d.wait(90);
    Some(d)
}

/// Logos and the opening skipped, Start Game, Main Game, Reptar Puzzle's
/// cinematic and its cards: Tommy in Pickles' Home.
#[test]
fn a_new_game_reaches_the_hub() {
    let Some(mut d) = to_title() else { return };
    // Start Game, then Main Game.
    d.press(Buttons::CROSS);
    d.press_until(Buttons::CROSS, 10, |a| a.state() != "title");
    d.press_until(Buttons::CROSS | Buttons::TRIANGLE, 40, |a| a.level_in_play().is_some());
    assert_eq!(d.app.level_in_play(), Some(Level::HUB));
}

/// Activities, Cookie Race: a level whose song bank once crashed the
/// program as it opened.
#[test]
fn activities_open_cookie_race() {
    let Some(mut d) = to_title() else { return };
    // Start Game, down to Activities, then Cookie Race (the first).
    d.press(Buttons::CROSS);
    d.press(Buttons::DOWN);
    d.press(Buttons::CROSS);
    d.press_until(Buttons::CROSS, 10, |a| a.state() != "title");
    d.press_until(Buttons::TRIANGLE, 40, |a| a.level_in_play().is_some());
    assert_eq!(d.app.level_in_play(), Some(Level::COOKIE_RACE));
}

/// Untouched, the program shows the legal still, the three logos and the
/// opening, then the title (main, 0x800185f8); left idle there, a demo.
#[test]
fn the_boot_plays_the_opening_before_the_title() {
    let Some(app) = app() else { return };
    let mut d = Driver { app, frame: 0, previous: Pad::default() };
    let mut seen: Vec<&'static str> = Vec::new();
    while d.frame < 60 * 140 && d.app.state() != "demo" {
        d.tick(Buttons::default());
        if seen.last() != Some(&d.app.state()) {
            seen.push(d.app.state());
        }
    }
    assert_eq!(seen, ["legal", "logo", "opening", "title", "demo"]);
}
