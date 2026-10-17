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
        self.script_with(steps, |_, _| {})
    }

    /// [`Self::script`], `each` shown both after every frame.
    fn script_with(&mut self, steps: &[(u16, usize)], mut each: impl FnMut(&Game, &mut Play)) -> (Vec<Seen>, Vec<Seen>) {
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
                if std::env::var_os("TRACKS").is_some() {
                    let orig: Vec<[i16; 6]> = (0..4u32).map(|k| std::array::from_fn(|i| self.hle.read_i16(rugrats_hle::play::PLAYER + 0xa8 + 12 * k + 2 * i as u32))).collect();
                    let port = self.game.player.animator.record();
                    let same = orig.iter().zip(&port).all(|(o, p)| o[..4] == p[..4] && (o[5] & 0xff) == (p[5] & 0xff));
                    eprintln!("    tracks {} port {:?}\n                orig {:?}", if same { "same" } else { "DIFF" }, port, orig);
                }
                if std::env::var_os("FOCUS").is_some() {
                    let f = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x1e8);
                    let ot = if f == 0 { None } else { Some(self.hle.read_i16(f + 0x96) as u16 as u32) };
                    let pt = self.game.focus().map(|k| self.game.set.placements[k].template);
                    eprintln!("    focus {} orig {ot:?} port {pt:?}", if ot == pt { "same" } else { "DIFF" });
                }
                orig.push(Seen { pos: o.pos, heading: o.heading as i32 & 4095, state: o.state, camera });
                each(&self.game, &mut self.hle);
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

/// Level 1 from its start: Up and Left for one frame, then let go: the
/// turn stops where the stick is let go (the target set to the heading
/// under a magnitude of 70, 0x8005c8d8).
#[test]
#[ignore]
fn hub_tap_turn_as_the_original() {
    use rugrats::input::{LEFT, UP};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let (port, orig) = b.script(&[(0, 3), (bits(&[UP, LEFT]), 1), (0, 12)]);
    let worst = report("tap", &port, &orig);
    let turned = port.iter().zip(&orig).filter(|(p, o)| p.heading != o.heading).count();
    assert_eq!((worst, turned), (0, 0), "the same as the original's, every frame");
}

/// Level 1: a run across the playpen (template 1032, flag 8: passed
/// through, the object in focus over it).
#[test]
#[ignore]
fn hub_run_across_the_playpen_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    b.put([5200, 779, 25776], 1024, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 30)]);
    let worst = report("pen", &port, &orig);
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1: toy 896 picked up (Circle), carried and thrown (Triangle). Each
/// part's turn (the hierarchy's quaternions, +4 of each part, as the
/// samplers leave them) is the original's on every frame once the pick-up
/// has blended out of what the boot left (the oracle stands mid-fidget);
/// the carried toy is on the original's place across (x, z) on every
/// frame, and within 1 in height (the root curve, kept apart in the port).
#[test]
#[ignore]
fn hub_pick_up_as_the_original() {
    use rugrats::input::{CIRCLE, TRIANGLE};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let k = b.game.things.iter().position(|t| t.template == 896).expect("toy 896");
    let t = b.game.things[k].pos;
    b.put([t.x as i32, 779, t.z as i32 - 700], 0, 6);
    let p = rugrats_hle::play::PLAYER;
    let (mut turns, mut held, mut obj) = (Vec::new(), Vec::new(), 0u32);
    b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[TRIANGLE]), 1), (0, 12)], |g, h| {
        let hier = h.read_u32(p + 0x154);
        let n = h.read_i16(hier) as u32;
        let orig: Vec<[i32; 4]> = (0..n).map(|i| [0, 4, 8, 12].map(|c| h.read_i32(hier + 0x34 + 40 * i + 4 + c))).collect();
        let port: Vec<[i32; 4]> = g.player.animator.turns(&g.player.hierarchy).iter().map(|q| q.unwrap_or([0; 4])).collect();
        turns.push(orig == port);
        let o = h.read_u32(p + 0x1f0);
        if o != 0 {
            obj = o;
        }
        if let Some(c) = g.carried() {
            let at = g.things[c].pos;
            let mine = [at.x as i32, at.y as i32, at.z as i32];
            let theirs = [0, 4, 8].map(|i| h.read_i32(obj + 0x18 + i));
            held.push((mine, theirs));
        }
    });
    let off = turns.iter().enumerate().filter(|(f, same)| *f >= 8 && !**same).count();
    // The grab's own frame: the carry (0x8005ff88 mode 1) moves it from the
    // next.
    let carried = &held[1..];
    let across = carried.iter().filter(|(m, t)| m[0] != t[0] || m[2] != t[2]).count();
    let height = carried.iter().map(|(m, t)| (m[1] - t[1]).abs()).max().unwrap_or(0);
    eprintln!("turns off {off} of {}, carried {} frames: across off {across}, height off at most {height}", turns.len(), carried.len());
    assert_eq!((off, across), (0, 0), "the original's turns and place");
    assert!(height <= 1, "height within the root curve's 1");
}
