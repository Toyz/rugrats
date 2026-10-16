//! A level in play as the boot runs it: its frame, then what it asked for
//! (sounds, bars, a load, the ending, an exit, a card, screens, a
//! cinematic).

use rrt::prelude::Tick;

use rugrats_render::Vtx;

use super::Rugrats;
use super::flow::{Boot, Then, piece_pages};
use crate::actors::player::DT;
use crate::levels::Level;
use crate::levels::golf::LEVELS;
use crate::media::cine;
use crate::ui::front::Card;
use crate::ui::screens::{Page, message};
use crate::world::game::{Exit, Game};

/// The screen's fade before a cinematic in a level (0x80018ad0): 127 dt,
/// a step of 256 / (127 / dt) a frame (21 at dt 10), 255 at most.
const FADE_TIME: i32 = 127;

impl Rugrats {
    /// One frame of the fade before cinematic `n`: the level as it
    /// stopped under a full-screen subtracted grey (0x8001cf2c: POLY_F4,
    /// draw mode 0x640, so B - F), then the cinematic.
    pub(super) fn fade(
        &mut self,
        mut game: Box<Game>,
        n: usize,
        (left, level, overlay): (i32, i32, usize),
        now: u64,
    ) -> Boot {
        if !now.is_multiple_of(2) {
            return Boot::Fade { game, n, left, level, overlay };
        }
        if left <= 0 {
            game.stage.overlay.truncate(overlay);
            let here = game.level;
            return self.scene(n, here, Then::Resume(game), now);
        }
        let level = level + 256 / (FADE_TIME / DT);
        game.stage.overlay.truncate(overlay);
        let grey = level.min(255) as u32;
        let v = |x: f32, y: f32| Vtx {
            pos: [x, y, 0.0],
            colour: grey | grey << 8 | grey << 16,
            uv: 0,
            mode: Vtx::FLAT | Vtx::SEMI | 2 << 21,
            window: 0,
        };
        game.stage.overlay.extend([
            v(0.0, 0.0),
            v(512.0, 0.0),
            v(0.0, 240.0),
            v(512.0, 0.0),
            v(512.0, 240.0),
            v(0.0, 240.0),
        ]);
        Boot::Fade { game, n, left: left - DT, level, overlay }
    }

    /// One frame of the level (every other tick), and the state that
    /// follows.
    pub(super) fn play(&mut self, mut game: Box<Game>, t: &Tick, now: u64) -> Boot {
        if !now.is_multiple_of(2) {
            return Boot::Game { game };
        }
        self.pads.update([Some(&t.pad), None]);
        game.state_block = self.progress.block;
        game.frame(&self.pads);
        self.play_effects(game.effects.iter().copied());
        if let Some(n) = game.say_now.take() {
            self.say_now(&mut game.lines, n);
        }
        self.say(&mut game.lines, &game.speech);
        let mut bought = false;
        for &n in &game.found {
            // 0x80094a58: each bar may buy a piece.
            self.progress.find_bar(game.level, n);
            if self.progress.buy_piece(&mut self.rand) {
                rrt::tracing::info!("15 bars bought a puzzle piece");
                bought = true;
            }
        }
        game.bars = self.progress.bars_available();
        // Load Game from the pause (0x80070970): the record's state, then
        // Pickles' Home at the saved level's spot (exit 1, 0x80018230,
        // 0x80070cc8).
        if let Some(block) = game.loaded.take() {
            self.progress.block = *block;
            self.stop_song();
            self.level = Some(self.progress.level());
            let page = message::loading(&self.disc, Level::HUB, false, true);
            return self.screen(vec![page], Level::HUB, Then::Level(Level::HUB), None, now);
        }
        if let Some(n) = game.song.take() {
            self.play_song(n);
        }
        if let Some(n) = game.room_song.take() {
            self.switch_song(n);
        }
        if game.ending {
            // The end (0x8007737c case 84, the teardown): the progress
            // cleared, the credits, then the hub at level 7's spot.
            self.progress.clear_for_ending();
            self.level = Some(Level::REPTAR_2010);
            self.stop_song();
            return self.credits(now);
        }
        if game.quit {
            // "Returning to / Main Menus / Loading" (0x8003e05c).
            self.golf_round = None;
            self.level = None;
            self.stop_song();
            let page = message::returning(&self.disc, self.progress.activity());
            return self.screen(vec![page], Level::HUB, Then::Title, None, now);
        }
        if let Some(exit) = game.exit {
            return self.exit(game, exit, now);
        }
        if let Some(file) = game.card.take() {
            // A hint card the level shows (0x80044434).
            match Card::open(&self.disc, file) {
                Ok(card) => {
                    self.world = None;
                    return Boot::Card { card: Box::new(card), n: 0, then: Then::Back(game) };
                }
                Err(e) => rrt::tracing::error!("card {file}: {e}"),
            }
        } else if bought || game.board || !game.pages.is_empty() {
            // A bought piece's screens (0x80094a58), the board in Pickles'
            // Home (use callback 1, 0x8004d3f0), or the level's own (the
            // golf box).
            let pages = if bought {
                piece_pages(&self.disc, &self.progress, game.level)
            } else if std::mem::take(&mut game.board) {
                vec![Page::Board]
            } else {
                std::mem::take(&mut game.pages)
            };
            let (here, vram) = (game.level, Some(game.stage.vram.clone()));
            return self.screen(pages, here, Then::Back(game), vram, now);
        } else if let Some(n) = game.cine.take() {
            // A cinematic inside the level (PostCine, 0x80018ad0): the
            // level waits for it. With the music byte's 32 the song fades
            // down over 60 ticks (0x8006ef0c); without its 16 the screen
            // fades first.
            let flags = cine::music_flags(&self.disc, n);
            if flags & 32 != 0
                && let Ok(mut spu) = self.spu.lock()
            {
                spu.fade_song(0, 60);
            }
            if flags & 16 == 0 {
                let overlay = game.stage.overlay.len();
                return Boot::Fade { game, n, left: FADE_TIME, level: 0, overlay };
            }
            let here = game.level;
            return self.scene(n, here, Then::Resume(game), now);
        }
        Boot::Game { game }
    }

    /// The level ended at `exit`: the golf round's turn, the way back to
    /// the Activities menu, the finish's bars and piece, then the next
    /// level (its door's cinematic first).
    fn exit(&mut self, mut game: Box<Game>, Exit { mut level, mut cine }: Exit, now: u64) -> Boot {
        // Activity golf (0x80096a30): a holed ball ends the turn; the next
        // player plays the hole from the tee, after the last the score
        // card and the next hole.
        if let (Some(n), Some(r)) = (game.strokes.take(), self.golf_round.as_mut()) {
            let (hole, p) = (r.hole, r.player);
            r.strokes[p][hole] = n;
            if p + 1 < r.slots.len() {
                r.player += 1;
                let (pl, ch) = (r.player, r.character());
                game.exit = None;
                game.with_script(|s, g| s.activity_turn(g, pl, ch));
                // The banner (0x800965e8) until Cross.
                let banner = r.banner(&self.disc);
                let here = game.level;
                return self.screen(vec![banner], here, Then::Back(game), None, now);
            }
            r.player = 0;
            if hole < 8 {
                // One frame of the card with "Loading Hole n..." through
                // the load, then the card until Cross (0x80095c2c).
                r.hole += 1;
                let next = LEVELS[r.hole];
                let cards = vec![r.card(Some(r.hole as u32 + 1)), r.card(None)];
                return self.screen(cards, Level::HUB, Then::Enter(next), None, now);
            }
            // After hole nine: the card, then the Activities menu (no
            // "Returning to").
            let card = r.card(None);
            self.golf_round = None;
            self.level = None;
            self.stop_song();
            return self.screen(vec![card], Level::HUB, Then::Title, None, now);
        }
        if self.progress.activity() && level == Level::HUB {
            // "Returning to the Activities Menu".
            self.golf_round = None;
            self.level = None;
            self.stop_song();
            let page = message::returning(&self.disc, true);
            return self.screen(vec![page], Level::HUB, Then::Title, None, now);
        }
        let here = game.level;
        if let Some(won) = game.won {
            let mut f = self.progress.finish(game.record, won, &mut self.rand);
            // The fourth bar (0x80076ba8, 0x80076bf0, 0x80076c24).
            if game.fourth_bar && f.bars > 0 {
                self.progress.find_bar(game.record, (31 - f.bars) as usize);
                f.bars += 1;
            }
            rrt::tracing::info!("level {here} finished: {f:?}");
            if let Some(next) = f.next {
                (level, cine) = (next, None);
            }
            // The bar award, then the piece and the board.
            let mut pages = Vec::new();
            if f.bars > 0 {
                pages.push(message::bars(&self.disc, f.bars, self.progress.bars_available()));
            }
            if f.piece {
                pages.extend(piece_pages(&self.disc, &self.progress, game.record));
            }
            if !pages.is_empty() {
                let vram = Some(game.stage.vram.clone());
                self.stop_song();
                return self.screen(pages, here, Then::Enter(level), vram, now);
            }
        }
        match cine {
            Some(n) => self.scene(n, here, Then::Level(level), now),
            None => self.enter(level, now),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::app::GolfRound;
    use crate::disc::GameDisc;

    fn app() -> Option<Rugrats> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc");
        let disc = GameDisc::open(&rrt::disc::Image::find(&dir).ok()?).ok()?;
        Rugrats::new(disc, false).ok()
    }

    fn holed(app: &mut Rugrats, mut game: Box<Game>, strokes: u32) -> Boot {
        game.strokes = Some(strokes);
        let exit = Exit::to(game.level);
        game.exit = Some(exit);
        app.exit(game, exit, 0)
    }

    /// A two-player round (0x80096a30): player 2's banner on the same
    /// hole, then the "Loading Hole 2..." card and the card with PRESS X,
    /// then hole two's cine and player 1's banner.
    #[test]
    fn a_golf_round_takes_turns_and_holes() {
        let Some(mut app) = app() else { return };
        app.progress.set_activity(true);
        app.golf_round = Some(GolfRound::new(vec![0, 1]));
        let Boot::Game { game } = app.start(LEVELS[0], 0) else { panic!("hole one did not open") };
        let Boot::Screen { screen: _, then: Then::Back(game) } = holed(&mut app, game, 3) else {
            panic!("no banner for player 2")
        };
        assert_eq!(app.golf_round.as_ref().map(|r| (r.hole, r.player)), Some((0, 1)));
        let Boot::Screen { then: Then::Enter(next), .. } = holed(&mut app, game, 5) else {
            panic!("no card after the last player")
        };
        assert_eq!(next, LEVELS[1]);
        let r = app.golf_round.as_ref().unwrap();
        assert_eq!((r.hole, r.player, r.strokes[0][0], r.strokes[1][0]), (1, 0, 3, 5));
        match app.enter(next, 0) {
            Boot::Scene { then: Then::Pages(pages, _), .. } => {
                assert!(matches!(pages.as_slice(), [Page::Message { .. }]), "player 1's banner");
            }
            _ => panic!("hole two did not start with its cine and the banner"),
        }
    }

    /// Screens, cinematics and cards run through as a player would: the
    /// board's pieces placed (all twelve: Reptar 2010, 0x80092c68), every
    /// card and page passed; stops at a level in play or anything else.
    fn settle(app: &mut Rugrats, mut boot: Boot) -> Boot {
        for _ in 0..50 {
            boot = match boot {
                Boot::Screen { then, .. } => {
                    let (earned, _) = app.progress.piece_bits();
                    app.progress.set_placed_bits(earned);
                    let all = earned.count_ones() == 12 && app.progress.flags() & 512 == 0;
                    app.after(if all { Then::Enter(Level::REPTAR_2010) } else { then }, 0, 0)
                }
                Boot::Scene { n, then, .. } | Boot::Card { n, then, .. } => app.after(then, n, 0),
                b => return b,
            };
        }
        panic!("the screens did not end")
    }

    /// The ending cinematic each level's win plays (0x8001532c).
    fn ending(level: Level) -> usize {
        match level.0 {
            2 => 5,
            3 => 14,
            4 => 48,
            5 => 40,
            6 => 47,
            9 => 19,
            10 => 78,
            11 => 62,
            12 => 68,
            13 => 21,
            15 => 72,
            17 => 26,
            18 => 24,
            20 => 64,
            21 => 50,
            l if Level(l) == LEVELS[8] => 35,
            l => panic!("no ending for level {l}"),
        }
    }

    /// The whole game from a new one: Reptar Puzzle's first piece, every
    /// main level won through its door and its ending cinematic (golf
    /// hole by hole), the bonus levels as the count opens them, the board
    /// filled, Reptar 2010, and Reptar Dance's ending into the credits.
    #[test]
    fn the_whole_game_from_new_to_the_credits() {
        let Some(mut app) = app() else { return };
        let mut doors = vec![2, 3, 4, 5, 6, 25, 9, 10, 11, 12].into_iter().map(Level);
        let mut seen = Vec::new();
        let first = app.enter(Level::HUB, 0);
        let mut boot = settle(&mut app, first);
        for _ in 0..80 {
            let Boot::Game { mut game } = boot else { panic!("not in a level") };
            let level = game.level;
            seen.push(level.0);
            let exit = if level == Level::HUB {
                // Through the next door (0x8004f78c).
                let Some(next) = doors.next() else { panic!("doors ran out before Reptar 2010: {seen:?}") };
                Exit::to(next)
            } else if level == Level::REPTAR_2010 {
                // Reptar Dance (cine 85) ends the game.
                game.cine_finished(85);
                assert!(game.ending);
                assert_eq!(app.progress.pieces(), 12);
                assert!(matches!(app.credits(0), Boot::Credits { .. }));
                assert!([2, 3, 4, 5, 6, 9, 10, 11, 12, 15, 17, 20, 21].iter().all(|l| seen.contains(l)), "{seen:?}");
                return;
            } else if level.is_hole() && level != LEVELS[8] {
                // At par: the next hole (golf.rs).
                let k = LEVELS.iter().position(|&l| l == level).unwrap();
                Exit::to(LEVELS[k + 1])
            } else {
                // The ninth hole records the course (golf.rs).
                if level.is_hole() {
                    game.record = Level::GOLF;
                }
                game.cine_finished(ending(level));
                assert_eq!(game.won, Some(true), "level {level}");
                game.exit.unwrap()
            };
            game.exit = Some(exit);
            let next = app.exit(game, exit, 0);
            boot = settle(&mut app, next);
        }
        panic!("no end after 80 levels: {seen:?}");
    }
}
