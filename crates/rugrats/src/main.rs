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

use rrt::prelude::*;
use rugrats::disc::GameDisc;
use rugrats::still;

/// Files the boot reads, by the game's numbering.
const LEGAL512_BS: usize = 234;

/// Where the boot is.
enum Boot {
    /// The legal screen (main, SLUS_006.50:0x800186cc: `show_still(234)`),
    /// held until the frame given.
    Legal { until: u64 },
    /// The opening cinematic (OPEN.BIN), which needs the 3D engine: not
    /// ported yet.
    Opening,
}

struct Rugrats {
    /// Kept for what the boot reads next.
    #[allow(dead_code)]
    disc: GameDisc,
    boot: Boot,
    screen: Picture,
}

impl Rugrats {
    fn new(disc: GameDisc) -> Result<Rugrats, String> {
        let screen = still::picture(&disc.file(LEGAL512_BS)?, &disc.mdec).map_err(|e| e.to_string())?;
        Ok(Rugrats { disc, boot: Boot::Legal { until: still::HOLD }, screen })
    }
}

impl Game for Rugrats {
    fn tick(&mut self, t: &mut Tick) {
        if t.pressed().contains(Buttons::SELECT) && t.pad.held(Buttons::START) {
            t.exit();
        }
        if let Boot::Legal { until } = self.boot
            && t.frame >= until
        {
            rrt::tracing::info!("boot: the opening cinematic is not ported yet");
            self.boot = Boot::Opening;
            self.screen = Picture::filled(still::WIDTH as u32, still::HEIGHT as u32, [0, 0, 0, 255]);
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
    let game = Rugrats::new(GameDisc::open(&cue)?)?;
    Ok(WithArgs(game, cli.app))
}
