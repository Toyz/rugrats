//! The port proved against the original: the same pad script played on the
//! port and on the original running in `rugrats-hle`, the player compared
//! frame by frame. Each boot of the original takes about a minute, so these
//! are ignored by default:
//!
//! ```text
//! cargo test --release -p rugrats --test hle -- --ignored --test-threads 1
//! ```

use std::path::Path;
use std::rc::Rc;

use rugrats::disc::GameDisc;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;
use rugrats_hle::play::{Boot, Play};

fn cue() -> Option<std::path::PathBuf> {
    rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()
}

/// The port and the original in the same level.
struct Both {
    game: Game,
    hle: Play,
    /// The port's pad, held back by the original's latency: a press
    /// reaches the original's player a game frame later (read at the
    /// vertical blank, used by the next frame).
    held: std::collections::VecDeque<u16>,
}

/// The original's input latency, in game frames.
const LAG: usize = 1;

/// One game frame's player, as compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seen {
    pos: [i32; 3],
    heading: i32,
    state: u8,
    camera: i32,
}

impl Both {
    fn open(boot: Boot) -> Option<Both> {
        let cue = cue()?;
        let disc = GameDisc::open(&cue).ok()?;
        let game = Game::open_from(&disc, Level(boot.level), None, &Progress::default()).ok()?;
        let hle = Play::boot(Rc::new(rrt::disc::Image::open(&cue).ok()?), boot).ok()?;
        Some(Both { game, hle, held: std::iter::repeat_n(0, LAG).collect() })
    }

    /// Both players put at `pos` facing `heading`, then `settle` frames
    /// with nothing held.
    fn put(&mut self, pos: [i32; 3], heading: i32, settle: usize) {
        self.hle.put(pos, heading as i16);
        let p = &mut self.game.player;
        p.pos = pos;
        p.heading = heading;
        p.target = heading;
        p.speed = 0;
        self.game.set_camera_yaw(heading as f32);
        self.script(&[(0, settle)]);
    }

    /// Runs `steps` of (pad bits, game frames) on both; the players after
    /// each frame, the port's then the original's.
    fn script(&mut self, steps: &[(u16, usize)]) -> (Vec<Seen>, Vec<Seen>) {
        let (mut port, mut orig) = (Vec::new(), Vec::new());
        let mut pads = Pads::default();
        for &(bits, n) in steps {
            for _ in 0..n {
                self.held.push_back(bits);
                pads.push_bits(0, self.held.pop_front().unwrap_or(0));
                self.game.frame(&pads);
                let p = &self.game.player;
                port.push(Seen { pos: p.pos, heading: p.heading, state: port_state(&self.game), camera: self.game.camera_heading() & 4095 });
                self.hle.frame(bits).unwrap();
                let o = self.hle.player();
                let camera = self.hle.camera_heading();
                if std::env::var_os("HLEVEL").is_some() {
                    let (low, plane) = (self.hle.read_i16(rugrats_hle::play::PLAYER + 0x68), self.hle.read_i32(rugrats_hle::play::PLAYER + 0x13c));
                    eprintln!("    vel {:?} port {:?} speed {} way {:?} heading {} cam {camera} pad {bits:#06x} low {low} plane {plane} flags {:#x} d {:?}", o.velocity, self.game.player.v, o.speed, o.way, o.heading, o.flags, [0, 2, 4].map(|k| self.hle.read_i16(rugrats_hle::play::PLAYER + 0xd8 + k)));
                }
                if std::env::var_os("CAMLOG").is_some() {
                    let r = |h: &mut Play, a: u32| [0, 4, 8].map(|k| h.read_i32(a + k));
                    let (eye, look) = (r(&mut self.hle, 0x800d_e530), r(&mut self.hle, 0x800d_e454));
                    let m = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x224);
                    let mat: Vec<i16> = (0..9).map(|k| self.hle.read_i16(m + 2 * k)).collect();
                    let (cam_eye, smooth) = (r(&mut self.hle, m + 0x14), r(&mut self.hle, 0x800d_e540));
                    let exact = self.game.trig.look_at(cam_eye, look);
                    eprintln!("    eye {eye:?} look {look:?} cam {cam_eye:?} at {smooth:?} m {mat:?} exact {exact:?} | port {:?}", self.game.follow_camera());
                }
                if std::env::var_os("CAMSTATE").is_some() {
                    let r = |h: &mut Play, a: u32| [0, 4, 8].map(|k| h.read_i32(a + k));
                    let m = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x224);
                    let o = (
                        r(&mut self.hle, 0x800d_e530),
                        r(&mut self.hle, m + 0x98),
                        r(&mut self.hle, 0x800d_e454),
                        self.hle.read_u32(0x800c_4a3c),
                        self.hle.read_i32(0x800c_4a58),
                        self.hle.read_i32(0x800c_4a5c),
                        self.hle.read_i32(0x800c_4a38),
                    );
                    let p = self.game.follow_state();
                    let same = o == p;
                    eprintln!("    cam {} orig {o:?}\n        port {p:?}", if same { "same" } else { "DIFF" });
                }
                orig.push(Seen { pos: o.pos, heading: o.heading as i32 & 4095, state: o.state, camera });
            }
        }
        (port, orig)
    }
}

/// The port's state as the original's +0x18c numbers it.
fn port_state(g: &Game) -> u8 {
    use rugrats::actors::player::{State, anim};
    if g.player.action == Some(anim::BUMP) {
        return 9;
    }
    match g.player.state {
        State::Stand => 0,
        State::Walk => 1,
        State::Run => 2,
        State::Jump => 5,
        State::Fall => 3,
        State::Land => 10,
    }
}

fn bits(buttons: &[usize]) -> u16 {
    buttons.iter().fold(0, |a, &b| a | 1 << b)
}

/// Prints the two traces side by side, with the largest distance apart.
fn report(name: &str, port: &[Seen], orig: &[Seen]) -> i32 {
    let mut worst = 0;
    for (k, (p, o)) in port.iter().zip(orig).enumerate() {
        // Across only: the original's y carries the run's root bob (the
        // port keeps it apart, `Player::root_y`).
        let d = [0, 2].iter().map(|&i| (p.pos[i] - o.pos[i]).abs()).max().unwrap();
        worst = worst.max(d);
        eprintln!(
            "{name} {k:3}: port {:?} h{:4} s{:2} c{:4} | orig {:?} h{:4} s{:2} c{:4} | {d}",
            p.pos, p.heading, p.state, p.camera, o.pos, o.heading, o.state, o.camera
        );
    }
    worst
}

/// Level 1, Pickles' Home: a run along the hall and into its wall.
#[test]
#[ignore]
fn hub_run_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    b.put([11000, 779, 19403], 1024, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let hold: usize = std::env::var("HOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let (port, orig) = b.script(&[(bits(&[UP]), hold), (0, 10)]);
    let worst = report("run", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1 from its start, nothing put: Up and Left held (the run across
/// the hall), then Cross (a running jump) into toy 403, met in the air: the
/// bounce, the fall had twice that frame, the landing.
#[test]
#[ignore]
fn hub_start_run_and_jump_as_the_original() {
    use rugrats::input::{CROSS, LEFT, UP};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let (port, orig) = b.script(&[(0, 3), (bits(&[UP, LEFT]), 12), (bits(&[UP, LEFT, CROSS]), 1), (bits(&[UP, LEFT]), 24)]);
    let worst = report("start", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 18: a run at the wall at a slant slides along it (door 213), into
/// the corner past it, and bumps once against the last wall: the same as
/// the original's on every frame.
#[test]
#[ignore]
fn egg_hunt_slide_and_bump_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 18, ..Default::default() }) else { return };
    b.put([28664, 779, 12748], 1433, 6);
    b.script(&[(bits(&[R1]), 1), (0, 2)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 70)]);
    let worst = report("slide", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 11 as Spike: a run at the wall past the first hurdle.
#[test]
#[ignore]
fn spike_run_at_a_wall_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 11, character: Some(6), ..Default::default() }) else { return };
    b.put([25252, 720, 4500], 0, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 12)]);
    let worst = report("spike", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1 from its start, nothing put: a standing jump (Cross alone), up
/// and back down onto the floor.
#[test]
#[ignore]
fn hub_standing_jump_as_the_original() {
    use rugrats::input::CROSS;
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let (port, orig) = b.script(&[(0, 3), (bits(&[CROSS]), 1), (0, 40)]);
    let worst = report("stand", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1: put back from the seat (template 130) at (8945, 779, 11800),
/// Up held: the run into it and the bump.
#[test]
#[ignore]
fn hub_run_at_the_seat_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    b.put([8945, 779, 11800], 0, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 40)]);
    let worst = report("seat", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1 from its start: a standing jump with Cross held (the lift's
/// clock, D_800c4288, runs out on the way up).
#[test]
#[ignore]
fn hub_held_jump_as_the_original() {
    use rugrats::input::CROSS;
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let (port, orig) = b.script(&[(0, 3), (bits(&[CROSS]), 20), (0, 20)]);
    let worst = report("held", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}
