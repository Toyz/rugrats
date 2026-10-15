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
use rrt::prelude::*;
use rugrats::disc::GameDisc;
use rugrats::input::{Pads, Query};
use rugrats::movie::{Movie, Soundtrack};
use rugrats::still;

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
    /// What main does next: the front end. Not ported yet.
    FrontEnd,
}

struct Rugrats {
    disc: GameDisc,
    boot: Boot,
    pads: Pads,
    screen: Picture,
    soundtrack: Arc<Mutex<Soundtrack>>,
    /// Kept so the sound plays; None headless or with no device.
    _audio: Option<Output>,
}

impl Rugrats {
    fn new(disc: GameDisc, with_audio: bool) -> Result<Rugrats, String> {
        let screen = still::picture(&disc.file(LEGAL512_BS)?, &disc.mdec).map_err(|e| e.to_string())?;
        let soundtrack: Arc<Mutex<Soundtrack>> = Default::default();
        let audio = with_audio
            .then(|| Output::open(soundtrack.clone()))
            .and_then(|r| r.map_err(|e| rrt::tracing::warn!("no audio: {e}")).ok());
        Ok(Rugrats {
            disc,
            boot: Boot::Legal { until: still::HOLD },
            pads: Pads::default(),
            screen,
            soundtrack,
            _audio: audio,
        })
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
        self.pads.update([Some(&t.pad), None]);
        let now = t.frame;
        match &mut self.boot {
            Boot::Legal { until } => {
                if now >= *until {
                    self.boot = self.logo(0, now);
                }
            }
            Boot::Logo { n, movie } => {
                // The skip is looked for once a frame of the movie, every
                // other tick.
                let skip = now.is_multiple_of(2) && self.pads.find(Query::Pressed).is_some();
                let playing = movie.tick(now, skip, &self.disc.mdec);
                self.screen = movie.picture.clone();
                if !playing {
                    let next = *n + 1;
                    if let Ok(mut s) = self.soundtrack.lock() {
                        *s = Soundtrack::default();
                    }
                    self.boot = if next < 3 {
                        self.logo(next, now)
                    } else {
                        rrt::tracing::info!("boot: the front end is not ported yet");
                        self.screen = Picture::filled(320, 240, [0, 0, 0, 255]);
                        Boot::FrontEnd
                    };
                }
            }
            Boot::FrontEnd => {}
        }
    }

    fn draw(&mut self, d: &mut Draw<'_>) {
        d.present_picture(&self.screen, None);
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
