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
use rugrats::front::{Outcome, Title};
use rugrats::stage::Stage;
use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads, Query};
use rugrats::movie::{Movie, Soundtrack};
use rugrats::sound::{self, Spu};
use rugrats::game::Game as Level;
use rugrats::still;
use rugrats_render::Renderer;

/// Files the boot reads, by the game's numbering.
const LEGAL512_BS: usize = 234;

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
    Scene { cine: Box<Cine>, then: usize },
    /// A level in play.
    Game { game: Box<Level> },
    /// Where the port ends for now.
    FrontEnd,
}

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
    /// Levels whose start cinematic has played (state +0x30's bits).
    seen: std::collections::HashSet<usize>,
    /// Kept so the sound plays; None headless or with no device.
    _audio: Option<Output>,
    /// The game's rand, which the babies and the chooser share.
    rand: rugrats::rand::Rand,
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
            seen: Default::default(),
            _audio: audio,
            world: None,
            rand: Default::default(),
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
        self.play_song(0, sound::MUSIC_VOLUME);
        match Title::open(&self.disc, now, &mut self.rand) {
            Ok(title) => Boot::Title { title: Box::new(title) },
            Err(e) => {
                rrt::tracing::error!("the title: {e}");
                self.front_end()
            }
        }
    }

    /// Game mode `mode`: for now its database of the same number.
    /// Level `level`, its start cinematic first unless already seen.
    fn enter(&mut self, level: usize, now: u64) -> Boot {
        let cine = rugrats::game::start_cine(level).filter(|_| !(self.seen.contains(&level) && matches!(level, 1 | 2)));
        self.seen.insert(level);
        match cine {
            Some(n) => self.scene(n, level, level, now),
            None => self.game(level, now),
        }
    }

    /// Cinematic `n` on database `db`, then level `then`.
    fn scene(&mut self, n: usize, db: usize, then: usize, now: u64) -> Boot {
        match Cine::open(&self.disc, n, db, now, &self.soundtrack) {
            Ok(cine) => {
                self.world = None;
                Boot::Scene { cine: Box::new(cine), then }
            }
            Err(e) => {
                rrt::tracing::error!("cine {n}: {e}");
                self.game(then, now)
            }
        }
    }

    fn game(&mut self, mode: usize, _now: u64) -> Boot {
        match Level::open(&self.disc, mode) {
            Ok(game) => {
                self.world = None;
                // The level's song set (0x80017c0c, 0x8006e96c), and for
                // levels 22 and up its song at once (0x8006fea8).
                self.stop_song();
                if let Some((set, song)) = sound::level_set(&self.disc, mode) {
                    self.song_set(set);
                    if let Some(n) = song {
                        self.play_song(n, sound::MUSIC_VOLUME);
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
            Boot::Scene { cine, then } => {
                if cine.tick(now) {
                    self.pads.update([Some(&t.pad), None]);
                    if self.pads.pressed(0, input::TRIANGLE) || cine.over() {
                        if let Ok(mut s) = self.soundtrack.lock() {
                            *s = Soundtrack::default();
                        }
                        let then = *then;
                        self.boot = self.game(then, now);
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
                    if let Ok(mut spu) = self.spu.lock() {
                        for &s in &title.sounds {
                            spu.effect(&self.effect_table, s as usize, sound::EFFECTS_VOLUME);
                        }
                    }
                    if let Some(Outcome::Play(mode)) = outcome {
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
                    game.frame(&self.pads);
                    if let Some(song) = game.song.take().and_then(|n| self.songs.as_ref().and_then(|l| l.songs.get(n))) {
                        if let Ok(mut spu) = self.spu.lock() {
                            spu.play(song, 0, sound::MUSIC_VOLUME);
                        }
                    }
                    if let Some((level, cine)) = game.exit {
                        let here = game.level;
                        self.boot = match cine {
                            Some(n) => {
                                self.seen.insert(level);
                                self.scene(n, here, level, now)
                            }
                            None => self.enter(level, now),
                        };
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
