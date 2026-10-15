//! The boot's states and the way between them: the logos, the title, the
//! cinematics and their cards, the screens between levels, and a level.

use rrt::prelude::{Picture, Tick};
use rugrats_render::Vram;

use super::{GolfRound, Rugrats};
use crate::audio::sound;
use crate::input::TRIANGLE;
use crate::levels::Level;
use crate::levels::golf::LEVELS;
use crate::media::cine::Cine;
use crate::media::movie::Movie;
use crate::save::progress::Progress;
use crate::ui::front::{self, Card, Outcome, Title};
use crate::ui::screens::{Page, Screen, message};
use crate::world::game::{Effect, Game, start_cine};

/// The opening cinematic: its number and its database.
const OPENING: usize = 0;
const OPENING_DB: usize = 22;

/// The credits movie (movie_play(6, 1) in 0x8008b1e4).
const CREDITS: usize = 6;

/// What follows a cinematic and its card: a level to load, or the level
/// it played in to go on with (play_cine inside the level loop,
/// 0x80018ad0), told the cinematic finished (CineFinish, 0x800783d8).
pub(super) enum Then {
    /// Level n, at once.
    Level(Level),
    /// Level n with its start cinematic (SwitchEpisode).
    Enter(Level),
    /// The level the cinematic interrupted, told it finished.
    Resume(Box<Game>),
    /// The level a screen interrupted.
    Back(Box<Game>),
    /// Screens, then more.
    Pages(Vec<Page>, Box<Then>),
    /// Cinematic n in a level's database, then more.
    Scene(usize, Level, Box<Then>),
    /// The title menu.
    Title,
}

/// The boot as main runs it (SLUS_006.50:0x800185f8).
pub(super) enum Boot {
    /// The legal screen (`show_still(234)`, 0x800186cc), held until the
    /// tick given.
    Legal { until: u64 },
    /// The THQ, Nickelodeon and n-Space logos: movies 0, 1, 2
    /// (0x8008b230, 0x8008b27c, 0x8008b0b4), each skippable.
    Logo { n: usize, movie: Box<Movie> },
    /// The title menu (0x8003eb28), which main's chooser (0x80030158)
    /// runs after the logos.
    Title { title: Box<Title> },
    /// A demo movie (3, 4 or 5: DEMO01-03.STR), which the chooser plays
    /// after an idle title (0x8008b100, 0x8008b14c, 0x8008b198).
    Demo { movie: Box<Movie> },
    /// The opening cinematic, cine 0 on database 22 (`Opening Cine Init`,
    /// 0x8008f078), which the chooser plays when the title is left idle.
    Opening { cine: Box<Cine> },
    /// A cinematic between levels: a level's start (0x80074958) or a
    /// door's (0x8004f78c), then level `then`.
    Scene { cine: Box<Cine>, n: usize, then: Then },
    /// A hint card after a cinematic (0x80044434), then `then`.
    Card { card: Box<Card>, n: usize, then: Then },
    /// A level in play.
    Game { game: Box<Game> },
    /// The credits after Reptar Dance (movie 6, CREDITS.STR, any button
    /// skips), then the hub.
    Credits { movie: Box<Movie> },
    /// The screens between and inside levels (the piece, the board, the
    /// bars): the level frozen, not drawn.
    Screen { screen: Box<Screen>, then: Then },
    /// Where the port ends for now.
    FrontEnd,
}

/// The pages after a piece was given in `level` (0x8009480c): the message,
/// then the board unless it is level 1's first piece.
pub(super) fn piece_pages(disc: &crate::disc::GameDisc, progress: &Progress, level: Level) -> Vec<Page> {
    if level == Level::HUB && progress.pieces() == 1 {
        vec![message::help(disc)]
    } else {
        vec![message::piece(disc), Page::Board]
    }
}

impl Rugrats {
    /// One tick of state `boot`; the state that follows.
    pub(super) fn step(&mut self, boot: Boot, t: &Tick) -> Boot {
        let now = t.frame;
        match boot {
            Boot::Legal { until } if now >= until => self.logo(0, now),
            Boot::Logo { n, mut movie } => {
                if self.movie(&mut movie, t, now) {
                    Boot::Logo { n, movie }
                } else if n + 1 < 3 {
                    self.logo(n + 1, now)
                } else {
                    self.title(now)
                }
            }
            Boot::Credits { mut movie } => {
                if self.movie(&mut movie, t, now) {
                    Boot::Credits { movie }
                } else {
                    self.start(Level::HUB, now)
                }
            }
            Boot::Demo { mut movie } => {
                if self.movie(&mut movie, t, now) {
                    Boot::Demo { movie }
                } else {
                    self.opening(now)
                }
            }
            Boot::Scene { cine, n, then } => self.scene_tick(cine, n, then, t, now),
            Boot::Screen { screen, then } => self.screen_tick(screen, then, t, now),
            Boot::Card { mut card, n, then } => {
                if now.is_multiple_of(2) {
                    self.pads.update([Some(&t.pad), None]);
                    if card.frame(&self.pads) {
                        return self.after(then, n, now);
                    }
                }
                Boot::Card { card, n, then }
            }
            Boot::Opening { mut cine } => {
                // play_cine samples the pads once a frame, after drawing it.
                if cine.tick(now) {
                    self.pads.update([Some(&t.pad), None]);
                    if self.pads.pressed(0, TRIANGLE) || cine.over() {
                        self.silence();
                        self.world = None;
                        return self.title(now);
                    }
                }
                Boot::Opening { cine }
            }
            Boot::Title { title } => self.title_tick(title, t, now),
            Boot::Game { game } => self.play(game, t, now),
            boot => boot,
        }
    }

    fn logo(&mut self, n: usize, now: u64) -> Boot {
        match Movie::open(&self.disc, n, true, now, &self.soundtrack) {
            Ok(movie) => Boot::Logo { n, movie: Box::new(movie) },
            Err(e) => {
                rrt::tracing::error!("movie {n}: {e}");
                Boot::FrontEnd
            }
        }
    }

    fn opening(&mut self, now: u64) -> Boot {
        match Cine::open(&self.disc, OPENING, OPENING_DB, now, &self.soundtrack) {
            Ok(cine) => Boot::Opening { cine: Box::new(cine) },
            Err(e) => {
                rrt::tracing::error!("the opening: {e}");
                self.front_end()
            }
        }
    }

    /// The chooser's demo: one of the three, at random.
    fn demo(&mut self, now: u64) -> Boot {
        let n = 3 + (self.rand.next() % 3) as usize;
        match Movie::open(&self.disc, n, true, now, &self.soundtrack) {
            Ok(movie) => Boot::Demo { movie: Box::new(movie) },
            Err(e) => {
                rrt::tracing::error!("demo {n}: {e}");
                self.opening(now)
            }
        }
    }

    /// The credits, then the hub.
    pub(super) fn credits(&mut self, now: u64) -> Boot {
        match Movie::open(&self.disc, CREDITS, true, now, &self.soundtrack) {
            Ok(movie) => Boot::Credits { movie: Box::new(movie) },
            Err(e) => {
                rrt::tracing::error!("credits: {e}");
                self.start(Level::HUB, now)
            }
        }
    }

    fn title(&mut self, now: u64) -> Boot {
        // The title's setup (0x8003eb28) opens the menu's song set, and
        // its loop (0x8003d884) plays song 0.
        self.song_set(sound::MENU_SET);
        self.play_song(0);
        match Title::open(&self.disc, now, &mut self.rand) {
            Ok(mut title) => {
                title.settings = self.progress.audio();
                Boot::Title { title: Box::new(title) }
            }
            Err(e) => {
                rrt::tracing::error!("the title: {e}");
                self.front_end()
            }
        }
    }

    fn title_tick(&mut self, mut title: Box<Title>, t: &Tick, now: u64) -> Boot {
        if !title.frame_due(now) {
            return Boot::Title { title };
        }
        // The menu samples the pads at the end of each frame.
        self.pads.update([Some(&t.pad), None]);
        let outcome = title.frame(&self.pads, &mut self.rand);
        // The Audio menu's settings, and the song's volume as they change
        // (0x8006ffa8).
        let mut audio = title.settings;
        audio.music = audio.music.clamp(0, 100);
        self.progress.set_audio(audio);
        if let (Some(v), Ok(mut spu)) = (title.song_volume, self.spu.lock()) {
            spu.song_volume = v;
        }
        self.play_effects(title.sounds.iter().map(|&s| Effect::new(s)));
        match outcome {
            Some(Outcome::Load(block)) => {
                // The record's state; Pickles' Home at the saved level's
                // spot (0x80070cc8).
                self.progress = Progress { block: *block };
                self.level = Some(self.progress.level());
                self.stop_song();
                self.enter(Level::HUB, now)
            }
            Some(Outcome::Golf(slots)) => {
                // Mini-golf's round (0x80096a30) after its setup.
                self.progress = Default::default();
                self.progress.set_activity(true);
                self.golf_round = Some(GolfRound::new(slots));
                self.stop_song();
                self.enter(LEVELS[0], now)
            }
            Some(Outcome::Play(mode)) => {
                if mode == front::MAIN_GAME {
                    self.progress = Default::default();
                } else if mode != front::TRAINING {
                    self.progress = Default::default();
                    self.progress.set_activity(true);
                }
                self.stop_song();
                rrt::tracing::info!("title: game mode {mode}");
                // Mini-golf from the Activities menu: one player, Tommy,
                // until the setup screen gives more.
                let level = Level(mode);
                if level == LEVELS[0] && self.progress.activity() {
                    self.golf_round = Some(GolfRound::new(vec![0]));
                }
                self.enter(level, now)
            }
            Some(Outcome::Idle) => {
                self.stop_song();
                self.demo(now)
            }
            None => Boot::Title { title },
        }
    }

    /// Level `level`, its start cinematic first (0x80074958): level 1's
    /// "Reptar Puzzle" once (+0x30 bit 4), giving the first piece; level
    /// 2's until its checkpoint (bit 2).
    pub(super) fn enter(&mut self, level: Level, now: u64) -> Boot {
        // Level 7's EpisodeStart (0x8007b748): the final level reached.
        if level == Level::REPTAR_2010 {
            self.progress.set_flags(self.progress.flags() | 512);
        }
        // Cine 73's start (0x800731ec): Mirrorland entered, +0x30 bit 16.
        if level == Level::MIRRORLAND {
            self.progress.set_flags(self.progress.flags() | 16);
        }
        // A new level starts without its checkpoint (+0x30 bits 1, 2).
        if self.level != Some(level) {
            self.progress.set_flags(self.progress.flags() & !3);
        }
        let flags = self.progress.flags();
        let cine = start_cine(level).filter(|_| match level {
            Level::HUB => flags & 4 == 0,
            Level::GRANDPAS_TEETH => flags & 2 == 0,
            _ => true,
        });
        // Level 25's intro only from outside the course (0x80074958).
        let cine = match (level, self.level) {
            (Level::GOLF_DOOR, Some(Level(24 | 25 | 28))) => Some(31),
            _ if level == Level::GOLF_DOOR && self.progress.activity() => Some(31),
            _ => cine,
        };
        // Reptar Puzzle gives the first piece (0x80094bfc(1)): "Help Tommy
        // win twelve puzzle pieces", the board only with more.
        let mut then = Then::Level(level);
        if level == Level::HUB && cine.is_some() {
            self.progress.set_flags(flags | 4);
            if self.progress.give_piece(Level::HUB, &mut self.rand) {
                then = Then::Pages(piece_pages(&self.disc, &self.progress, Level::HUB), Box::new(then));
            }
        }
        // The "already earned" notes (0x80017c0c): coming from another
        // level into one whose piece was given, or into Cookie Race or Egg
        // Hunt with their bars won.
        let mut notes = vec![message::loading(&self.disc, level, self.progress.activity(), false)];
        if level != Level::HUB && self.level != Some(level) && self.progress.piece_given(level) {
            notes.push(message::had_piece(&self.disc));
        }
        // The bars note comes after the start cinematic and its card.
        if matches!(level, Level::COOKIE_RACE | Level::EGG_HUNT)
            && self.progress.won(level)
            && !self.progress.activity()
        {
            then = Then::Pages(vec![message::had_bars(&self.disc)], Box::new(then));
        }
        // The notes come before the start cinematic.
        let next = match cine {
            Some(n) => Then::Scene(n, level, Box::new(then)),
            None => then,
        };
        self.screen(notes, level, next, None, now)
    }

    /// Cinematic `n` in `level`'s database, then `then`.
    pub(super) fn scene(&mut self, n: usize, level: Level, then: Then, now: u64) -> Boot {
        self.stop_song();
        match Cine::open(&self.disc, n, level.index(), now, &self.soundtrack) {
            Ok(cine) => {
                self.world = None;
                Boot::Scene { cine: Box::new(cine), n, then }
            }
            Err(e) => {
                rrt::tracing::error!("cine {n}: {e}");
                self.after(then, n, now)
            }
        }
    }

    fn scene_tick(&mut self, mut cine: Box<Cine>, n: usize, then: Then, t: &Tick, now: u64) -> Boot {
        if !cine.tick(now) {
            return Boot::Scene { cine, n, then };
        }
        self.pads.update([Some(&t.pad), None]);
        if !(self.pads.pressed(0, TRIANGLE) || cine.over()) {
            return Boot::Scene { cine, n, then };
        }
        self.silence();
        // CineFinish (0x800783d8) shows the cinematic's card. No hint
        // cards in activity play.
        let card = front::card_after(n).filter(|_| !self.progress.activity());
        match card.map(|f| Card::open(&self.disc, f)) {
            Some(Ok(card)) => {
                self.world = None;
                Boot::Card { card: Box::new(card), n, then }
            }
            _ => self.after(then, n, now),
        }
    }

    /// The screens `pages` (sprites from `level`'s database), then
    /// `then`. `vram` is what the level left (for its bar icon).
    pub(super) fn screen(&mut self, pages: Vec<Page>, level: Level, then: Then, vram: Option<Vram>, now: u64) -> Boot {
        let (earned, placed) = self.progress.piece_bits();
        match Screen::open(&self.disc, pages, level.index(), earned, placed, vram.as_ref()) {
            Ok(screen) => {
                self.world = None;
                Boot::Screen { screen: Box::new(screen), then }
            }
            Err(e) => {
                rrt::tracing::error!("screen: {e}");
                self.after(then, 0, now)
            }
        }
    }

    fn screen_tick(&mut self, mut screen: Box<Screen>, mut then: Then, t: &Tick, now: u64) -> Boot {
        if !now.is_multiple_of(2) {
            return Boot::Screen { screen, then };
        }
        self.pads.update([Some(&t.pad), None]);
        let going = screen.frame(&self.pads, &mut self.rand);
        self.play_effects(screen.sounds.iter().map(|&s| Effect::new(s)));
        // Lines from the level's set, as the board runs in it.
        if let Then::Back(game) = &mut then {
            self.say(&mut game.lines, &screen.speech);
        }
        if going {
            return Boot::Screen { screen, then };
        }
        self.progress.set_placed_bits(screen.placed);
        // All twelve placed: Reptar 2010 (0x80092c68 step 5).
        let then =
            if screen.all_placed && self.progress.flags() & 512 == 0 { Then::Enter(Level::REPTAR_2010) } else { then };
        // Restart: the level again (golf from Hole One), its checkpoint
        // cleared.
        let then = match then {
            Then::Back(game) if screen.restart => {
                let level = if game.level.is_hole() { Level::GOLF_DOOR } else { game.level };
                self.progress.set_flags(self.progress.flags() & !2);
                self.level = None;
                Then::Enter(level)
            }
            t => t,
        };
        self.after(then, 0, now)
    }

    /// What follows cinematic `n` (and its card).
    pub(super) fn after(&mut self, then: Then, n: usize, now: u64) -> Boot {
        match then {
            Then::Level(level) => self.start(level, now),
            Then::Enter(level) => self.enter(level, now),
            Then::Back(game) => {
                self.world = None;
                Boot::Game { game }
            }
            Then::Scene(n, level, then) => self.scene(n, level, *then, now),
            Then::Title => {
                let back = self.progress.activity();
                let mut boot = self.title(now);
                if let (true, Boot::Title { title }) = (back, &mut boot) {
                    title.activities();
                }
                boot
            }
            Then::Pages(pages, then) => {
                let level = self.level.unwrap_or(Level::HUB);
                self.screen(pages, level, *then, None, now)
            }
            Then::Resume(mut game) => {
                self.world = None;
                game.cine_finished(n);
                Boot::Game { game }
            }
        }
    }

    /// Level `level` opened and in play, its song set loaded.
    pub(super) fn start(&mut self, level: Level, _now: u64) -> Boot {
        let from = self.level.replace(level);
        self.progress.enter(level);
        match Game::open_from(&self.disc, level, from, &self.progress) {
            Ok(mut game) => {
                // Activity golf: the turn's player and body (0x80096a30).
                if let Some(r) = &self.golf_round {
                    let (p, c) = (r.player, r.character());
                    game.with_script(|s, g| s.activity_turn(g, p, c));
                }
                self.world = None;
                // The level's song set (0x80017c0c, 0x8006e96c), and for
                // levels 22 and up its song at once (0x8006fea8).
                self.stop_song();
                if let Some((set, song)) = sound::level_set(&self.disc, level.index()) {
                    self.song_set(set);
                    if let Some(n) = song {
                        self.play_song(n);
                    }
                }
                Boot::Game { game: Box::new(game) }
            }
            Err(e) => {
                rrt::tracing::error!("level {level}: {e}");
                self.front_end()
            }
        }
    }

    pub(super) fn front_end(&mut self) -> Boot {
        rrt::tracing::info!("boot: the front end is not ported yet");
        self.picture = Picture::filled(320, 240, [0, 0, 0, 255]);
        Boot::FrontEnd
    }
}
