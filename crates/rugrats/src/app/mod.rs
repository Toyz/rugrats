//! The program: main's boot (SLUS_006.50:0x800185f8) and chooser
//! (0x80030158) as a state machine on retro_rt's loop, the sound devices,
//! and drawing whichever stage is up.

mod flow;
mod golf_round;
mod play;

use std::sync::{Arc, Mutex};

use rrt::audio::Output;
use rrt::prelude::{Buttons, Draw, Picture, Target, Tick};
use rugrats_render::Renderer;

use crate::audio::sound::{self, Spu};
use crate::audio::speech::Speech;
use crate::disc::GameDisc;
use crate::levels::Level;
use crate::media::cine;
use crate::media::movie::{Movie, Soundtrack};
use crate::media::still;
use crate::rand::Rand;
use crate::save::card::Cards;
use crate::save::progress::Progress;
use crate::stage::Stage;
use crate::world::game::Effect;

use flow::Boot;
use golf_round::GolfRound;

/// The legal screen's still.
const LEGAL512_BS: usize = 234;

pub struct Rugrats {
    disc: GameDisc,
    boot: Boot,
    pads: crate::input::Pads,
    /// What a still or a movie shows.
    picture: Picture,
    soundtrack: Arc<Mutex<Soundtrack>>,
    /// The SPU's voices: music and effects.
    spu: Arc<Mutex<Spu>>,
    /// The song set loaded (the menu's, a level's).
    songs: Option<sound::Loaded>,
    effect_table: Vec<[u8; 3]>,
    /// The game state (0x800c5678), what a save holds.
    progress: Progress,
    /// The level last played (state +0x04), for the hub's return spot.
    level: Option<Level>,
    /// Kept so the sound plays; None headless or with no device.
    _audio: Option<Output>,
    /// The game's rand, which the babies and the chooser share.
    rand: Rand,
    /// Activity mini-golf's round (0x80096a30).
    golf_round: Option<GolfRound>,
    /// The faces the last cinematic left in VRAM, and a count of them
    /// (see `FaceBlocks`).
    faces: (u64, crate::media::face::FaceBlocks),
    /// What draws a 3D screen, made for its stage's generation, the 4:3
    /// picture it draws into, and the VRAM revision it holds.
    world: Option<(Renderer, Target, u64, u64)>,
}

impl Rugrats {
    /// The program on `disc`; `with_audio` opens the sound device.
    pub fn new(disc: GameDisc, with_audio: bool) -> Result<Rugrats, String> {
        let picture = still::picture(&disc.file(LEGAL512_BS)?, &disc.mdec).map_err(|e| e.to_string())?;
        let soundtrack: Arc<Mutex<Soundtrack>> = Default::default();
        let spu: Arc<Mutex<Spu>> = Default::default();
        let effect_table = sound::effect_table(&disc)?;
        let audio = with_audio
            .then(|| Output::open(Arc::new(Mutex::new(sound::Mix { track: soundtrack.clone(), spu: spu.clone() }))))
            .and_then(|r| r.map_err(|e| rrt::tracing::warn!("no audio: {e}")).ok());
        let mut progress = Progress::default();
        if progress.restore_audio(&Cards::default(), &disc.card) {
            rrt::tracing::info!("audio settings from the card: {:?}", progress.audio());
        }
        Ok(Rugrats {
            disc,
            boot: Boot::Legal { until: still::HOLD },
            pads: Default::default(),
            picture,
            soundtrack,
            spu,
            songs: None,
            effect_table,
            level: None,
            progress,
            _audio: audio,
            world: None,
            rand: Default::default(),
            golf_round: None,
            faces: (0, Default::default()),
        })
    }

    /// The boot state's name (for tools and tests).
    pub fn state(&self) -> &'static str {
        match &self.boot {
            Boot::Legal { .. } => "legal",
            Boot::Logo { .. } => "logo",
            Boot::Title { .. } => "title",
            Boot::Demo { .. } => "demo",
            Boot::Opening { .. } => "opening",
            Boot::Scene { .. } => "scene",
            Boot::Card { .. } => "card",
            Boot::Game { .. } | Boot::Fade { .. } => "game",
            Boot::Credits { .. } => "credits",
            Boot::Screen { .. } => "screen",
            Boot::Blank => "blank",
        }
    }

    /// The level in play, if one is (for tools and tests).
    pub fn level_in_play(&self) -> Option<Level> {
        match &self.boot {
            Boot::Game { game } => Some(game.level),
            _ => None,
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
    fn play_song(&mut self, n: usize) {
        let Some(song) = self.songs.as_ref().and_then(|l| l.songs.get(n)) else { return };
        if let Ok(mut spu) = self.spu.lock() {
            spu.play(song, n, 0, self.progress.audio().music);
        }
    }

    /// A room's song (0x8006efcc): crossfaded from the one playing over
    /// 30 ticks.
    fn switch_song(&mut self, n: usize) {
        let Some(song) = self.songs.as_ref().and_then(|l| l.songs.get(n)) else { return };
        if let Ok(mut spu) = self.spu.lock() {
            spu.switch_song(song, n, self.progress.audio().music, 30);
        }
    }

    /// The song as cinematic `n` begins (0x8007424c, its table byte +7):
    /// with neither 8 nor 32 it stops (0x8006ee14), with 32 it pauses
    /// (0x8006ec40), with 8 alone it plays on (the golf cameras).
    fn cine_song_start(&mut self, n: usize) {
        let flags = cine::music_flags(&self.disc, n);
        if flags & 40 == 0 {
            self.stop_song();
        } else if flags & 32 != 0
            && let Ok(mut spu) = self.spu.lock()
        {
            spu.pause_song();
        }
    }

    /// The song as cinematic `n` ends: a paused one goes on, fading in
    /// from silence over 60 ticks (0x8006ed30, 0x8006ef64).
    fn cine_song_end(&mut self, n: usize) {
        if cine::music_flags(&self.disc, n) & 32 != 0
            && let Ok(mut spu) = self.spu.lock()
        {
            spu.resume_song(self.progress.audio().music, 60);
        }
    }

    fn stop_song(&mut self) {
        if let Ok(mut spu) = self.spu.lock() {
            spu.stop_song();
        }
    }

    /// Effects asked for this frame: a world sound at its own volumes,
    /// the rest at the effects setting.
    fn play_effects(&self, effects: impl IntoIterator<Item = Effect>) {
        let Ok(mut spu) = self.spu.lock() else { return };
        let setting = self.progress.audio().effects;
        for e in effects {
            let at = e.at.unwrap_or((setting, setting));
            match e.key {
                Some(key) => spu.key(key, at),
                None => spu.effect(&self.effect_table, e.sound as usize, at),
            }
        }
    }

    /// Speech (0x8008da4c): one line at a time; one asked for while
    /// another plays is dropped.
    fn say(&self, lines: &mut Speech, said: &[u16]) {
        for &n in said {
            if let Ok(mut s) = self.soundtrack.lock()
                && !s.playing()
                && let Some(sound) = lines.line(&self.disc, n)
            {
                *s = Soundtrack::new(sound);
            }
        }
    }

    /// A line at once, cutting off what was said (0x8008df78).
    fn say_now(&self, lines: &mut Speech, n: u16) {
        if let (Ok(mut s), Some(sound)) = (self.soundtrack.lock(), lines.line(&self.disc, n)) {
            *s = Soundtrack::new(sound);
        }
    }

    /// Stops a movie's or a cinematic's sound.
    fn silence(&self) {
        if let Ok(mut s) = self.soundtrack.lock() {
            *s = Soundtrack::default();
        }
    }

    /// One movie tick, skippable once a frame of it (every other tick).
    /// None while it plays; when it ends, whether a button skipped it
    /// (0x8008b448 returns 1 for a skip, 0 for the end).
    fn movie(&mut self, movie: &mut Movie, t: &Tick, now: u64) -> Option<bool> {
        let skip = now.is_multiple_of(2) && {
            self.pads.update([Some(&t.pad), None]);
            self.pads.find(crate::input::Query::Pressed).is_some()
        };
        let playing = movie.tick(now, skip, &self.disc.mdec);
        self.picture = movie.picture.clone();
        if playing {
            return None;
        }
        self.silence();
        Some(skip)
    }
}

impl rrt::prelude::Game for Rugrats {
    fn tick(&mut self, t: &mut Tick) {
        if t.pressed().contains(Buttons::SELECT) && t.pad.held(Buttons::START) {
            t.exit();
        }
        let boot = std::mem::replace(&mut self.boot, Boot::Blank);
        self.boot = self.step(boot, t);
    }

    fn draw(&mut self, d: &mut Draw<'_>) {
        let stage: &Stage = match &self.boot {
            Boot::Opening { cine } => &cine.stage,
            Boot::Scene { cine, .. } => &cine.stage,
            Boot::Card { card, .. } => &card.stage,
            Boot::Screen { screen, .. } => &screen.stage,
            Boot::Title { title } => &title.stage,
            Boot::Game { game } | Boot::Fade { game, .. } => game.view(),
            _ => {
                d.present_picture(&self.picture, None);
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
