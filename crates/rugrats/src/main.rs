//! rugrats: Rugrats: Search for Reptar, ported to the PC.
//!
//! Runs only ported code, on retro_rt's loop: what the original does is
//! rewritten here a subsystem at a time. Where the port ends, it says so on
//! the console and holds the screen.
//!
//! ```text
//! rugrats [DISC.cue] [the standard flags: --shot, --frames, --press, --replay, --fullscreen, ...]
//! ```

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rrt::audio::Output;
use rrt::gpu::Target;
use rrt::prelude::*;
use rugrats::cine::Cine;
use rugrats::disc::GameDisc;
use rugrats::front::{Outcome, Title};
use rugrats::game::Game as Level;
use rugrats::input::{self, Pads, Query};
use rugrats::movie::{Movie, Soundtrack};
use rugrats::screens::{self, Page, Screen};
use rugrats::sound::{self, Spu};
use rugrats::stage::Stage;
use rugrats::still;
use rugrats_render::Renderer;

/// Files the boot reads, by the game's numbering.
const LEGAL512_BS: usize = 234;

/// What follows a cinematic and its card: a level to load, or the level
/// it played in to go on with (play_cine inside the level loop,
/// 0x80018ad0), told the cinematic finished (CineFinish, 0x800783d8).
enum Then {
    /// Level n, at once.
    Level(usize),
    /// Level n with its start cinematic (SwitchEpisode).
    Enter(usize),
    /// The level the cinematic interrupted, told it finished.
    Resume(Box<Level>),
    /// The level a screen interrupted.
    Back(Box<Level>),
    /// Screens, then more.
    Pages(Vec<Page>, Box<Then>),
    /// Cinematic n on database db, then more.
    Scene(usize, usize, Box<Then>),
    /// The title menu.
    Title,
}

/// The boot as main runs it (SLUS_006.50:0x800185f8).
enum Boot {
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
    Card { card: Box<rugrats::front::Card>, n: usize, then: Then },
    /// A level in play.
    Game { game: Box<Level> },
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
fn piece_pages(disc: &GameDisc, progress: &rugrats::progress::Progress, level: usize) -> Vec<Page> {
    if level == 1 && progress.pieces() == 1 {
        vec![screens::message::help(disc)]
    } else {
        vec![screens::message::piece(disc), Page::Board]
    }
}

/// The credits movie (movie_play(6, 1) in 0x8008b1e4).
const CREDITS: usize = 6;

/// The opening cinematic: its number and its database.
const OPENING: usize = 0;
const OPENING_DB: usize = 22;

struct Rugrats {
    disc: GameDisc,
    boot: Boot,
    pads: Pads,
    screen: Picture,
    soundtrack: Arc<Mutex<Soundtrack>>,
    /// The SPU's voices: music and effects.
    spu: Arc<Mutex<Spu>>,
    /// The song set loaded (the menu's, a level's).
    songs: Option<sound::Loaded>,
    effect_table: Vec<[u8; 3]>,
    /// The game state (0x800c5678), what a save holds.
    progress: rugrats::progress::Progress,
    /// The level last played (state +0x04), for the hub's return spot.
    level: Option<usize>,
    /// Levels whose start cinematic has played (state +0x30's bits).
    /// Kept so the sound plays; None headless or with no device.
    _audio: Option<Output>,
    /// The game's rand, which the babies and the chooser share.
    rand: rugrats::rand::Rand,
    /// Activity mini-golf's strokes by hole (0x800c46d0 +0x81c).
    golf_scores: [u32; 9],
    /// What draws a 3D screen, made for its stage's generation, the 4:3
    /// picture it draws into, and the VRAM revision it holds.
    world: Option<(Renderer, Target, u64, u64)>,
}

impl Rugrats {
    fn new(disc: GameDisc, with_audio: bool) -> Result<Rugrats, String> {
        let screen = still::picture(&disc.file(LEGAL512_BS)?, &disc.mdec).map_err(|e| e.to_string())?;
        let soundtrack: Arc<Mutex<Soundtrack>> = Default::default();
        let spu: Arc<Mutex<Spu>> = Default::default();
        let effect_table = sound::effect_table(&disc)?;
        let audio = with_audio
            .then(|| Output::open(Arc::new(Mutex::new(sound::Mix { track: soundtrack.clone(), spu: spu.clone() }))))
            .and_then(|r| r.map_err(|e| rrt::tracing::warn!("no audio: {e}")).ok());
        Ok(Rugrats {
            disc,
            boot: Boot::Legal { until: still::HOLD },
            pads: Pads::default(),
            screen,
            soundtrack,
            spu,
            songs: None,
            effect_table,
            level: None,
            progress: Default::default(),
            _audio: audio,
            world: None,
            rand: Default::default(),
            golf_scores: [0; 9],
        })
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

    /// Loads song set `set` (0x8006e96c) unless it is the one loaded.
    fn song_set(&mut self, set: sound::SongSet) {
        if self.songs.as_ref().is_some_and(|l| l.set == set) {
            return;
        }
        match sound::Loaded::load(&self.disc, set) {
            Ok(l) => {
                if let Ok(mut spu) = self.spu.lock() {
                    spu.set_banks(l.music.clone(), l.effects.clone());
                }
                self.songs = Some(l);
            }
            Err(e) => rrt::tracing::error!("song set {set:?}: {e}"),
        }
    }

    /// Song `n` of the set loaded (MidiPlaySong, 0x8006eac4), forever.
    fn play_song(&mut self, n: usize, volume: i32) {
        let Some(song) = self.songs.as_ref().and_then(|l| l.songs.get(n)) else { return };
        if let Ok(mut spu) = self.spu.lock() {
            spu.play(song, 0, volume);
        }
    }

    fn stop_song(&mut self) {
        if let Ok(mut spu) = self.spu.lock() {
            spu.stop_song();
        }
    }

    fn title(&mut self, now: u64) -> Boot {
        // The title's setup (0x8003eb28) opens the menu's song set, and
        // its loop (0x8003d884) plays song 0.
        self.song_set(sound::MENU_SET);
        self.play_song(0, self.progress.audio().music);
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

    /// Game mode `mode`: for now its database of the same number.
    /// Level `level`, its start cinematic first (0x80074958): level 1's
    /// "Reptar Puzzle" once (+0x30 bit 4), giving the first piece; level
    /// 2's until its checkpoint (bit 2).
    fn enter(&mut self, level: usize, now: u64) -> Boot {
        // Level 7's EpisodeStart (0x8007b748): the final level reached.
        if level == 7 {
            self.progress.set_flags(self.progress.flags() | 512);
        }
        // Cine 73's start (0x800731ec): Mirrorland entered, +0x30 bit 16.
        if level == 15 {
            self.progress.set_flags(self.progress.flags() | 16);
        }
        // A new level starts without its checkpoint (+0x30 bits 1, 2).
        if self.level != Some(level) {
            self.progress.set_flags(self.progress.flags() & !3);
        }
        let flags = self.progress.flags();
        let cine = rugrats::game::start_cine(level).filter(|_| match level {
            1 => flags & 4 == 0,
            2 => flags & 2 == 0,
            _ => true,
        });
        // Level 25's intro only from outside the course (0x80074958).
        let cine = match (level, self.level) {
            (25, Some(24 | 25 | 28)) => Some(31),
            _ if level == 25 && self.progress.activity() => Some(31),
            _ => cine,
        };
        // Reptar Puzzle gives the first piece (0x80094bfc(1)): "Help Tommy
        // win twelve puzzle pieces", the board only with more.
        let mut then = Then::Level(level);
        if level == 1 && cine.is_some() {
            self.progress.set_flags(flags | 4);
            if self.progress.give_piece(1, &mut self.rand) {
                then = Then::Pages(piece_pages(&self.disc, &self.progress, 1), Box::new(then));
            }
        }
        // The "already earned" notes (0x80017c0c): coming from another
        // level into one whose piece was given, or into Cookie Race or Egg
        // Hunt with their bars won.
        let mut notes = vec![screens::message::loading(&self.disc, level, self.progress.activity(), false)];
        if level != 1 && self.level != Some(level) && self.progress.piece_given(level) {
            notes.push(screens::message::had_piece(&self.disc));
        }
        // The bars note comes after the start cinematic and its card.
        if matches!(level, 13 | 18) && self.progress.won(level) && !self.progress.activity() {
            then = Then::Pages(vec![screens::message::had_bars(&self.disc)], Box::new(then));
        }
        match (cine, notes.is_empty()) {
            (Some(n), true) => self.scene(n, level, then, now),
            (None, true) => self.after(then, 0, now),
            (cine, false) => {
                // The notes come before the start cinematic.
                let next = match cine {
                    Some(n) => Then::Scene(n, level, Box::new(then)),
                    None => then,
                };
                self.screen(notes, level, next, None, now)
            }
        }
    }

    /// Cinematic `n` on database `db`, then `then`.
    fn scene(&mut self, n: usize, db: usize, then: Then, now: u64) -> Boot {
        self.stop_song();
        match Cine::open(&self.disc, n, db, now, &self.soundtrack) {
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

    /// The screens `pages` (sprites from database `db`), then `then`.
    /// `vram` is what the level left (for its bar icon).
    fn screen(&mut self, pages: Vec<Page>, db: usize, then: Then, vram: Option<rugrats_render::Vram>, now: u64) -> Boot {
        let (earned, placed) = self.progress.piece_bits();
        match Screen::open(&self.disc, pages, db, earned, placed, vram.as_ref()) {
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

    /// What follows cinematic `n` (and its card).
    fn after(&mut self, then: Then, n: usize, now: u64) -> Boot {
        match then {
            Then::Level(level) => self.game(level, now),
            Then::Enter(level) => self.enter(level, now),
            Then::Back(game) => {
                self.world = None;
                Boot::Game { game }
            }
            Then::Scene(n, db, then) => self.scene(n, db, *then, now),
            Then::Title => {
                let back = self.progress.activity();
                let mut boot = self.title(now);
                if let (true, Boot::Title { title }) = (back, &mut boot) {
                    title.activities();
                }
                boot
            }
            Then::Pages(pages, then) => {
                let db = self.level.unwrap_or(1);
                self.screen(pages, db, *then, None, now)
            }
            Then::Resume(mut game) => {
                self.world = None;
                game.cine_finished(n);
                Boot::Game { game }
            }
        }
    }

    fn game(&mut self, mode: usize, _now: u64) -> Boot {
        let from = self.level.replace(mode);
        self.progress.enter(mode);
        match Level::open_from(&self.disc, mode, from, &self.progress) {
            Ok(game) => {
                self.world = None;
                // The level's song set (0x80017c0c, 0x8006e96c), and for
                // levels 22 and up its song at once (0x8006fea8).
                self.stop_song();
                if let Some((set, song)) = sound::level_set(&self.disc, mode) {
                    self.song_set(set);
                    if let Some(n) = song {
                        self.play_song(n, self.progress.audio().music);
                    }
                }
                Boot::Game { game: Box::new(game) }
            }
            Err(e) => {
                rrt::tracing::error!("game mode {mode}: {e}");
                self.front_end()
            }
        }
    }

    fn front_end(&mut self) -> Boot {
        rrt::tracing::info!("boot: the front end is not ported yet");
        self.screen = Picture::filled(320, 240, [0, 0, 0, 255]);
        Boot::FrontEnd
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
}

impl Game for Rugrats {
    fn tick(&mut self, t: &mut Tick) {
        if t.pressed().contains(Buttons::SELECT) && t.pad.held(Buttons::START) {
            t.exit();
        }
        let now = t.frame;
        match &mut self.boot {
            Boot::Legal { until } => {
                if now >= *until {
                    self.boot = self.logo(0, now);
                }
            }
            Boot::Logo { n, movie } => {
                // The pads are sampled, and a skip looked for, once a frame
                // of the movie: every other tick.
                let skip = now.is_multiple_of(2) && {
                    self.pads.update([Some(&t.pad), None]);
                    self.pads.find(Query::Pressed).is_some()
                };
                let playing = movie.tick(now, skip, &self.disc.mdec);
                self.screen = movie.picture.clone();
                if !playing {
                    let next = *n + 1;
                    if let Ok(mut s) = self.soundtrack.lock() {
                        *s = Soundtrack::default();
                    }
                    self.boot = if next < 3 { self.logo(next, now) } else { self.title(now) };
                }
            }
            Boot::Credits { movie } => {
                let skip = now.is_multiple_of(2) && {
                    self.pads.update([Some(&t.pad), None]);
                    self.pads.find(Query::Pressed).is_some()
                };
                let playing = movie.tick(now, skip, &self.disc.mdec);
                self.screen = movie.picture.clone();
                if !playing {
                    if let Ok(mut s) = self.soundtrack.lock() {
                        *s = Soundtrack::default();
                    }
                    self.boot = self.game(rugrats::game::HUB, now);
                }
            }
            Boot::Demo { movie } => {
                let skip = now.is_multiple_of(2) && {
                    self.pads.update([Some(&t.pad), None]);
                    self.pads.find(Query::Pressed).is_some()
                };
                let playing = movie.tick(now, skip, &self.disc.mdec);
                self.screen = movie.picture.clone();
                if !playing {
                    if let Ok(mut s) = self.soundtrack.lock() {
                        *s = Soundtrack::default();
                    }
                    self.boot = self.opening(now);
                }
            }
            Boot::Scene { cine, n, .. } => {
                if cine.tick(now) {
                    self.pads.update([Some(&t.pad), None]);
                    if self.pads.pressed(0, input::TRIANGLE) || cine.over() {
                        if let Ok(mut s) = self.soundtrack.lock() {
                            *s = Soundtrack::default();
                        }
                        let n = *n;
                        let Boot::Scene { then, .. } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                        // CineFinish (0x800783d8) shows the cinematic's card.
                        // No hint cards in activity play.
                        let card = rugrats::front::card_after(n).filter(|_| !self.progress.activity());
                        self.boot =
                            match card.map(|f| rugrats::front::Card::open(&self.disc, f)) {
                                Some(Ok(card)) => {
                                    self.world = None;
                                    Boot::Card { card: Box::new(card), n, then }
                                }
                                _ => self.after(then, n, now),
                            };
                    }
                }
            }
            Boot::Screen { screen, then } => {
                if now.is_multiple_of(2) {
                    self.pads.update([Some(&t.pad), None]);
                    let going = screen.frame(&self.pads, &mut self.rand);
                    if let Ok(mut spu) = self.spu.lock() {
                        for &s in &screen.sounds {
                            spu.effect(&self.effect_table, s as usize, self.progress.audio().effects);
                        }
                    }
                    // Lines from the level's set, as the board runs in it.
                    if let Then::Back(game) = then {
                        for &n in &screen.speech {
                            if let Ok(mut s) = self.soundtrack.lock() {
                                if !s.playing() {
                                    if let Some(sound) = game.lines.line(&self.disc, n) {
                                        *s = Soundtrack::new(sound);
                                    }
                                }
                            }
                        }
                    }
                    if !going {
                        let (placed, all, restart) = (screen.placed, screen.all_placed, screen.restart);
                        self.progress.set_placed_bits(placed);
                        let Boot::Screen { then, .. } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                        // All twelve placed: Reptar 2010 (0x80092c68 step 5).
                        let then = if all && self.progress.flags() & 512 == 0 { Then::Enter(7) } else { then };
                        // Restart: the level again (golf from Hole One),
                        // its checkpoint cleared.
                        let then = match then {
                            Then::Back(game) if restart => {
                                let level = if (23..=31).contains(&game.level) { 25 } else { game.level };
                                self.progress.set_flags(self.progress.flags() & !2);
                                self.level = None;
                                Then::Enter(level)
                            }
                            t => t,
                        };
                        self.boot = self.after(then, 0, now);
                    }
                }
            }
            Boot::Card { card, n, .. } => {
                if now.is_multiple_of(2) {
                    self.pads.update([Some(&t.pad), None]);
                    if card.frame(&self.pads) {
                        let n = *n;
                        let Boot::Card { then, .. } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                        self.boot = self.after(then, n, now);
                    }
                }
            }
            Boot::Opening { cine } => {
                // play_cine samples the pads once a frame, after drawing it.
                if cine.tick(now) {
                    self.pads.update([Some(&t.pad), None]);
                    if self.pads.pressed(0, input::TRIANGLE) || cine.over() {
                        if let Ok(mut s) = self.soundtrack.lock() {
                            *s = Soundtrack::default();
                        }
                        self.world = None;
                        self.boot = self.title(now);
                    }
                }
            }
            Boot::Title { title } => {
                if title.frame_due(now) {
                    // The menu samples the pads at the end of each frame.
                    self.pads.update([Some(&t.pad), None]);
                    let outcome = title.frame(&self.pads, &mut self.rand);
                    // The Audio menu's settings, and the song's volume
                    // as they change (0x8006ffa8).
                    let mut audio = title.settings;
                    audio.music = audio.music.clamp(0, 100);
                    self.progress.set_audio(audio);
                    if let (Some(v), Ok(mut spu)) = (title.song_volume, self.spu.lock()) {
                        spu.song_volume = v;
                    }
                    if let Ok(mut spu) = self.spu.lock() {
                        for &s in &title.sounds {
                            spu.effect(&self.effect_table, s as usize, self.progress.audio().effects);
                        }
                    }
                    if let Some(Outcome::Load(block)) = &outcome {
                        // The record's state; Pickles' Home at the saved
                        // level's spot (0x80070cc8).
                        self.progress = rugrats::progress::Progress { block: **block };
                        self.level = Some(self.progress.level());
                        self.stop_song();
                        self.boot = self.enter(rugrats::game::HUB, now);
                    } else if let Some(Outcome::Play(mode)) = outcome {
                        if mode == rugrats::front::MAIN_GAME {
                            self.progress = Default::default();
                        } else if mode != rugrats::front::TRAINING {
                            self.progress = Default::default();
                            self.progress.set_activity(true);
                        }
                        self.stop_song();
                        rrt::tracing::info!("title: game mode {mode}");
                        self.boot = self.enter(mode as usize, now);
                    } else if outcome == Some(Outcome::Idle) {
                        self.stop_song();
                        self.boot = self.demo(now);
                    }
                }
            }
            Boot::Game { game } => {
                if now.is_multiple_of(2) {
                    self.pads.update([Some(&t.pad), None]);
                    game.state_block = self.progress.block;
                    game.frame(&self.pads);
                    if let Ok(mut spu) = self.spu.lock() {
                        for &(e, v) in &game.effects {
                            spu.effect(&self.effect_table, e as usize, v * self.progress.audio().effects / sound::EFFECTS_VOLUME);
                        }
                    }
                    // Speech (0x8008da4c): one line at a time; one asked for
                    // while another plays is dropped.
                    for &n in &game.speech {
                        if let Ok(mut s) = self.soundtrack.lock() {
                            if !s.playing() {
                                if let Some(sound) = game.lines.line(&self.disc, n) {
                                    *s = Soundtrack::new(sound);
                                }
                            }
                        }
                    }
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
                    // Load Game from the pause (0x80070970): the record's state,
                    // then Pickles' Home at the saved level's spot (exit 1,
                    // 0x80018230, 0x80070cc8).
                    if let Some(block) = game.loaded.take() {
                        self.progress.block = *block;
                        let saved = self.progress.level();
                        self.stop_song();
                        self.level = Some(saved);
                        let page = screens::message::loading(&self.disc, rugrats::game::HUB, false, true);
                        self.boot = self.screen(vec![page], 1, Then::Level(rugrats::game::HUB), None, now);
                        return;
                    }
                    if let Some(song) = game.song.take().and_then(|n| self.songs.as_ref().and_then(|l| l.songs.get(n)))
                    {
                        if let Ok(mut spu) = self.spu.lock() {
                            spu.play(song, 0, self.progress.audio().music);
                        }
                    }
                    if game.ending {
                        // The end (0x8007737c case 84, the teardown): the
                        // progress cleared, the credits, then the hub at
                        // level 7's spot.
                        self.progress.clear_for_ending();
                        self.level = Some(7);
                        self.stop_song();
                        self.boot = match Movie::open(&self.disc, CREDITS, true, now, &self.soundtrack) {
                            Ok(movie) => Boot::Credits { movie: Box::new(movie) },
                            Err(e) => {
                                rrt::tracing::error!("credits: {e}");
                                self.game(rugrats::game::HUB, now)
                            }
                        };
                        return;
                    }
                    if game.quit {
                        // "Returning to / Main Menus / Loading" (0x8003e05c).
                        self.level = None;
                        self.stop_song();
                        let page = screens::message::returning(&self.disc, self.progress.activity());
                        self.boot = self.screen(vec![page], 1, Then::Title, None, now);
                    } else if let Some((mut level, mut cine)) = game.exit {
                        // Activity golf (0x80096a30): each hole's strokes
                        // kept for the score card.
                        if let Some(n) = game.strokes {
                            if let Some(k) = rugrats::golf::LEVELS.iter().position(|&l| l == game.level) {
                                if k == 0 {
                                    self.golf_scores = [0; 9];
                                }
                                self.golf_scores[k] = n;
                            }
                        }
                        // The card before each next hole (0x80095c2c).
                        if game.strokes.is_some() && self.progress.activity() && level != rugrats::game::HUB {
                            let page = Page::Scores(vec![self.golf_scores]);
                            self.boot = self.screen(vec![page], 1, Then::Enter(level), None, now);
                            return;
                        }
                        if self.progress.activity() && level == rugrats::game::HUB {
                            // "Returning to the Activities Menu"; golf goes
                            // hole to hole first, then its score card.
                            let golf = game.strokes.is_some();
                            self.level = None;
                            self.stop_song();
                            let mut pages = Vec::new();
                            if golf {
                                pages.push(Page::Scores(vec![self.golf_scores]));
                            }
                            pages.push(screens::message::returning(&self.disc, true));
                            self.boot = self.screen(pages, 1, Then::Title, None, now);
                            return;
                        }
                        if let Some(won) = game.won {
                            let mut f = self.progress.finish(game.record, won, &mut self.rand);
                            // The fourth bar (0x80076ba8, 0x80076bf0, 0x80076c24).
                            if game.fourth_bar && f.bars > 0 {
                                self.progress.find_bar(game.record, (31 - f.bars) as usize);
                                f.bars += 1;
                            }
                            rrt::tracing::info!("level {} finished: {f:?}", game.level);
                            if let Some(next) = f.next {
                                (level, cine) = (next, None);
                            }
                            // The bar award, then the piece and the board.
                            let mut pages = Vec::new();
                            if f.bars > 0 {
                                pages.push(screens::message::bars(&self.disc, f.bars, self.progress.bars_available()));
                            }
                            if f.piece {
                                pages.extend(piece_pages(&self.disc, &self.progress, game.record));
                            }
                            if !pages.is_empty() {
                                let here = game.level;
                                let vram = Some(game.stage.vram.clone());
                                self.stop_song();
                                self.boot = self.screen(pages, here, Then::Enter(level), vram, now);
                                return;
                            }
                        }
                        let here = game.level;
                        self.boot = match cine {
                            Some(n) => {
                                self.scene(n, here, Then::Level(level), now)
                            }
                            None => self.enter(level, now),
                        };
                    } else if let Some(file) = game.card.take() {
                        // A hint card the level shows (0x80044434).
                        match rugrats::front::Card::open(&self.disc, file) {
                            Ok(card) => {
                                let Boot::Game { game } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                                self.world = None;
                                self.boot = Boot::Card { card: Box::new(card), n: 0, then: Then::Back(game) };
                            }
                            Err(e) => rrt::tracing::error!("card {file}: {e}"),
                        }
                    } else if bought || game.board || !game.pages.is_empty() {
                        // A bought piece's screens (0x80094a58), the board in
                        // Pickles' Home (use callback 1, 0x8004d3f0), or the
                        // level's own (the golf box).
                        let pages = if bought {
                            piece_pages(&self.disc, &self.progress, game.level)
                        } else if std::mem::take(&mut game.board) {
                            vec![Page::Board]
                        } else {
                            std::mem::take(&mut game.pages)
                        };
                        let here = game.level;
                        let vram = Some(game.stage.vram.clone());
                        let Boot::Game { game } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                        self.boot = self.screen(pages, here, Then::Back(game), vram, now);
                    } else if let Some(n) = game.cine.take() {
                        // A cinematic inside the level (PostCine, 0x80018ad0):
                        // the level waits for it.
                        let here = game.level;
                        let Boot::Game { game } = std::mem::replace(&mut self.boot, Boot::FrontEnd) else { unreachable!() };
                        self.boot = self.scene(n, here, Then::Resume(game), now);
                    }
                }
            }
            Boot::FrontEnd => {}
        }
    }

    fn draw(&mut self, d: &mut Draw<'_>) {
        let stage: &Stage = match &self.boot {
            Boot::Opening { cine } => &cine.stage,
            Boot::Scene { cine, .. } => &cine.stage,
            Boot::Card { card, .. } => &card.stage,
            Boot::Screen { screen, .. } => &screen.stage,
            Boot::Title { title } => &title.stage,
            Boot::Game { game } => &game.stage,
            _ => {
                d.present_picture(&self.screen, None);
                return;
            }
        };
        let (w, h) = (d.height * 4 / 3, d.height);
        if self.world.as_ref().is_none_or(|w| w.2 != stage.generation) {
            let r = Renderer::new(&d.gpu.device, &d.gpu.queue, Target::FORMAT, &stage.vram, &stage.fixed);
            self.world = Some((r, Target::new(&d.gpu.device, w, h), stage.generation, stage.vram_rev));
        }
        let (renderer, target, _, rev) = self.world.as_mut().unwrap();
        if *rev != stage.vram_rev {
            renderer.load_vram_rect(&d.gpu.queue, 0, 0, 1024, 512, &stage.vram.words);
            *rev = stage.vram_rev;
        }
        if (target.width, target.height) != (w, h) {
            *target = Target::new(&d.gpu.device, w, h);
        }
        renderer.clear = stage.clear;
        renderer.set_moving(&d.gpu.device, &d.gpu.queue, &stage.moving);
        renderer.set_semi(&d.gpu.device, &d.gpu.queue, &stage.semi);
        renderer.set_under(&d.gpu.device, &d.gpu.queue, &stage.under);
        renderer.set_overlay(&d.gpu.device, &d.gpu.queue, &stage.overlay);
        let cmd = renderer.draw(&d.gpu.device, &d.gpu.queue, &target.view, (w, h), stage.camera);
        d.submit(cmd);
        d.present_target(target, None);
    }
}

/// Rugrats: Search for Reptar, ported to the PC.
#[derive(rrt::Args)]
struct Cli {
    /// The disc's CUE sheet (default: the one under work/disc).
    #[arg(positional)]
    disc: Option<PathBuf>,
    #[arg(flatten)]
    app: AppArgs,
}

#[rrt::main(title = "Rugrats: Search for Reptar", hz = rrt::app::rate::NTSC, size = (1024, 720))]
fn main() -> Result<WithArgs<Rugrats>, String> {
    let cli = Cli::parse_env(env!("CARGO_PKG_VERSION"));
    let cue = match cli.disc {
        Some(p) => p,
        None => rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?,
    };
    let game = Rugrats::new(GameDisc::open(&cue)?, cli.app.shot.is_none())?;
    Ok(WithArgs(game, cli.app))
}
