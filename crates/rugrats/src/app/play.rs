//! A level in play as the boot runs it: its frame, then what it asked for
//! (sounds, bars, a load, the ending, an exit, a card, screens, a
//! cinematic).

use rrt::prelude::Tick;

use super::Rugrats;
use super::flow::{Boot, Then, piece_pages};
use crate::levels::Level;
use crate::levels::golf::LEVELS;
use crate::ui::front::Card;
use crate::ui::screens::{Page, message};
use crate::world::game::{Exit, Game};

impl Rugrats {
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
            // level waits for it.
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
                r.hole += 1;
                let (next, card) = (LEVELS[r.hole], r.card(true));
                return self.screen(vec![card], Level::HUB, Then::Enter(next), None, now);
            }
            // After hole nine: the card, then the Activities menu (no
            // "Returning to").
            let card = r.card(true);
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
