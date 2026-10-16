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
        })
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
            spu.play(song, 0, self.progress.audio().music);
        }
    }

    fn stop_song(&mut self) {
        if let Ok(mut spu) = self.spu.lock() {
            spu.stop_song();
        }
    }

    /// Effects asked for this frame, scaled by the effects setting.
    fn play_effects(&self, effects: impl IntoIterator<Item = Effect>) {
        let Ok(mut spu) = self.spu.lock() else { return };
        let setting = self.progress.audio().effects;
        for e in effects {
            spu.effect(&self.effect_table, e.sound as usize, e.volume * setting / sound::EFFECTS_VOLUME);
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

    /// Stops a movie's or a cinematic's sound.
    fn silence(&self) {
        if let Ok(mut s) = self.soundtrack.lock() {
            *s = Soundtrack::default();
        }
    }

    /// One movie tick, skippable once a frame of it (every other tick);
    /// whether it still plays.
    fn movie(&mut self, movie: &mut Movie, t: &Tick, now: u64) -> bool {
        let skip = now.is_multiple_of(2) && {
            self.pads.update([Some(&t.pad), None]);
            self.pads.find(crate::input::Query::Pressed).is_some()
        };
        let playing = movie.tick(now, skip, &self.disc.mdec);
        self.picture = movie.picture.clone();
        if !playing {
            self.silence();
        }
        playing
    }
}

impl rrt::prelude::Game for Rugrats {
    fn tick(&mut self, t: &mut Tick) {
        if t.pressed().contains(Buttons::SELECT) && t.pad.held(Buttons::START) {
            t.exit();
        }
        let boot = std::mem::replace(&mut self.boot, Boot::FrontEnd);
        self.boot = self.step(boot, t);
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
