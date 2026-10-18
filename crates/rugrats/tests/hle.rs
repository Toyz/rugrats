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
    /// Frames whose animation tracks (+0xa8, four of 12 bytes) were not
    /// the original's.
    tracks_off: Vec<usize>,
    frames: usize,
    /// The original's camera updates (0x8002953c returning into the
    /// frame, 0x80018db4) so far, and as of the last frame compared: a
    /// frame the original ran late (its camera still under way after two
    /// vertical blanks) has no camera to compare.
    cameras: Rc<std::cell::Cell<u32>>,
    cameras_seen: u32,
}

/// The original's input latency, in game frames.
const LAG: usize = 1;

/// One game frame's player, as compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seen {
    pos: [i32; 3],
    /// The height with the animation's root, as the original's body has
    /// it (the port's position plus the root the step before left).
    y: i32,
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
        let cameras = Rc::new(std::cell::Cell::new(0));
        let mut hle = hle;
        {
            let c = cameras.clone();
            hle.hle.m.hook(0x8001_8db4, move |_| {
                c.set(c.get() + 1);
                rrt::emu::HookResult::Continue
            });
        }
        let mut b = Both {
            game,
            hle,
            held: std::iter::repeat_n(0, LAG).collect(),
            tracks_off: Vec::new(),
            frames: 0,
            cameras,
            cameras_seen: 0,
        };
        b.sync_animation();
        Some(b)
    }

    /// The port's animator given the original's state as it stands (its
    /// tracks, speed, states after, fidget and parts' turns), so both play
    /// on from the same place.
    fn sync_animation(&mut self) {
        let p = rugrats_hle::play::PLAYER;
        let h = &mut self.hle;
        let tracks: [[i16; 6]; 4] = std::array::from_fn(|k| std::array::from_fn(|i| h.read_i16(p + 0xa8 + 12 * k as u32 + 2 * i as u32)));
        let speed = h.read_i16(p + 0xe4) as i32;
        let after = (h.read_u8(p + 0xe6), h.read_u8(p + 0xe7));
        let fidget = (h.read_i16(p + 0xf0), h.read_i16(p + 0xf2) as i32);
        let hier = h.read_u32(p + 0x154);
        let parts = h.read_i16(hier) as u32;
        let header: Vec<u8> = (0..0x34).map(|i| h.read_u8(hier + i)).collect();
        let mut pose = vec![None; 32];
        for (channel, slot) in pose.iter_mut().enumerate() {
            let part = header.get(0x14 + channel).copied().unwrap_or(0xff) as u32;
            if part < parts {
                let q = [0, 4, 8, 12].map(|c| h.read_i32(hier + 0x34 + 40 * part + 4 + c));
                if q != [0; 4] {
                    *slot = Some(q);
                }
            }
        }
        let g = &mut self.game;
        g.player.animator.load_record(tracks, speed, after, fidget, pose, &g.anims);
    }

    /// Both players put at `pos` facing `heading`, the port's animation
    /// given the original's, then `settle` frames with nothing held.
    fn put(&mut self, pos: [i32; 3], heading: i32, settle: usize) {
        self.sync_animation();
        self.hle.put(pos, heading as i16);
        // And the body's matrix (+4, 0x80020d10 of its angles), which the
        // original makes again only as it turns.
        let m = self.game.trig.rotation([heading as i16, 0, 0]);
        for r in 0..3 {
            for c in 0..3 {
                self.hle.write_i16(rugrats_hle::play::PLAYER + 4 + 2 * (3 * r + c) as u32, m[r][c] as i16);
            }
        }
        let p = &mut self.game.player;
        p.pos = pos;
        p.heading = heading;
        p.target = heading;
        p.speed = 0;
        self.game.set_camera_yaw(heading as f32);
        // A room's post camera (taken up in the frame) looks from where the
        // original's look-at is.
        self.script(&[(0, 1)]);
        let look = [0, 4, 8].map(|k| self.hle.read_i32(0x800d_e454 + k));
        self.game.sync_post_look(look);
        self.script(&[(0, settle.saturating_sub(1))]);
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
                port.push(Seen { pos: p.pos, y: p.pos[1] + p.root_y, heading: p.heading, state: port_state(&self.game), camera: self.game.camera_heading() & 4095 });
                self.hle.frame(bits).unwrap();
                let o = self.hle.player();
                // (By the game's own table ratan2, 0x800a0328, on +0x224.)
                let camera = {
                    let m = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x224);
                    let r: [[i32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| self.hle.read_i16(m + 6 * i as u32 + 2 * j as u32) as i32));
                    if m == 0 { 0 } else { self.game.heading_of_rows(&r) }
                };
                if std::env::var_os("HLEVEL").is_some() {
                    let (low, plane) = (self.hle.read_i16(rugrats_hle::play::PLAYER + 0x68), self.hle.read_i32(rugrats_hle::play::PLAYER + 0x13c));
                    eprintln!("    mag {} state {} | port speed {} | vel {:?} port {:?} speed {} way {:?} heading {} cam {camera} pad {bits:#06x} low {low} plane {plane} flags {:#x} d {:?}", self.hle.read_i16(rugrats_hle::play::PLAYER + 0x1b0), o.state, self.game.player.speed, o.velocity, self.game.player.v, o.speed, o.way, o.heading, o.flags, [0, 2, 4].map(|k| self.hle.read_i16(rugrats_hle::play::PLAYER + 0xd8 + k)));
                }
                if std::env::var_os("CAMLOG").is_some() {
                    let r = |h: &mut Play, a: u32| [0, 4, 8].map(|k| h.read_i32(a + k));
                    let (eye, look) = (r(&mut self.hle, 0x800d_e530), r(&mut self.hle, 0x800d_e454));
                    let m = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x224);
                    let mat: Vec<i16> = (0..9).map(|k| self.hle.read_i16(m + 2 * k)).collect();
                    let (cam_eye, smooth) = (r(&mut self.hle, m + 0x14), r(&mut self.hle, 0x800d_e540));
                    let exact = self.game.trig.look_at(cam_eye, look);
                    let (kind, flags) = (self.hle.read_i16(0x800d_e460 + 0x4a), self.hle.read_u32(0x800c_4a3c));
                    eprintln!("    eye {eye:?} look {look:?} cam {cam_eye:?} at {smooth:?} m {mat:?} exact {exact:?} kind {kind} flags {flags:#x} | port {:?} rows {:?} view {:?}", self.game.follow_camera(), self.game.camera_rows(), self.game.view_eye_target());
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
                {
                    let orig: Vec<[i16; 6]> = (0..4u32).map(|k| std::array::from_fn(|i| self.hle.read_i16(rugrats_hle::play::PLAYER + 0xa8 + 12 * k + 2 * i as u32))).collect();
                    let port = self.game.player.animator.record();
                    let same = orig.iter().zip(&port).all(|(o, p)| o[..4] == p[..4] && (o[5] & 0xff) == (p[5] & 0xff));
                    if !same {
                        self.tracks_off.push(self.frames);
                    }
                    if std::env::var_os("TRACKS").is_some() {
                        eprintln!("    tracks {} port {:?}\n                orig {:?}", if same { "same" } else { "DIFF" }, port, orig);
                    }
                    self.frames += 1;
                }
                if std::env::var_os("FOCUS").is_some() {
                    let f = self.hle.read_u32(rugrats_hle::play::PLAYER + 0x1e8);
                    let ot = if f == 0 { None } else { Some(self.hle.read_i16(f + 0x96) as u16 as u32) };
                    let pt = self.game.focus().map(|k| self.game.set.placements[k].template);
                    eprintln!("    focus {} orig {ot:?} port {pt:?}", if ot == pt { "same" } else { "DIFF" });
                }
                if std::env::var_os("YDIFF").is_some() {
                    let p = &self.game.player;
                    eprintln!("    y orig {} port {} + root {} = {} | {}", o.pos[1], p.pos[1], p.root_y, p.pos[1] + p.root_y, o.pos[1] - (p.pos[1] + p.root_y));
                }
                // (A frame the original ran late: its camera not yet done.)
                let done = self.cameras.get() != self.cameras_seen;
                self.cameras_seen = self.cameras.get();
                let camera = if done { camera } else { LATE };
                orig.push(Seen { pos: o.pos, y: o.pos[1], heading: o.heading as i32 & 4095, state: o.state, camera });
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

/// The frames whose camera heading (as the stick reads it, 0x800a0328 on
/// +0x224) is not the original's.
fn camera_off(port: &[Seen], orig: &[Seen]) -> usize {
    port.iter().zip(orig).filter(|(p, o)| o.camera != LATE && p.camera != o.camera).count()
}

/// The camera of a frame the original ran late (its update still under
/// way when the frame was read).
const LATE: i32 = i32::MIN;

/// The largest difference in height (with the root) over the frames.
fn height_off(port: &[Seen], orig: &[Seen]) -> i32 {
    port.iter().zip(orig).map(|(p, o)| (p.y - o.y).abs()).max().unwrap_or(0)
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    assert_eq!(worst, 0, "the same as the original's, every frame");
    assert_eq!(camera_off(&port, &orig), 0, "the follow camera's heading");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
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
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 1: toy 896 picked up (Circle), carried and thrown (Triangle). Each
/// part's turn (the hierarchy's quaternions, +4 of each part, as the
/// samplers leave them) is the original's on every frame once the pick-up
/// has blended out of what the boot left (the oracle stands mid-fidget);
/// the carried toy is on the original's place across (x, z) on every
/// frame, its height too (the hand as last drawn, the body's height with
/// its animation's root);
/// thrown, its flight is the original's on every frame: its start where
/// the hand as last drawn let go, its launch and fall in integers, and
/// its bounces off toys 431 and 430 and the floor (0x80087594).
#[test]
#[ignore]
fn hub_pick_up_as_the_original() {
    use rugrats::input::{CIRCLE, TRIANGLE};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let k = b.game.things.iter().position(|t| t.template == 896).expect("toy 896");
    let t = b.game.things[k].pos;
    b.put([t.x as i32, 779, t.z as i32 - 700], 0, 6);
    // The camera behind on both (R1, 0x80026ddc): the put leaves the
    // original's where it was.
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let p = rugrats_hle::play::PLAYER;
    let placed = placements(&mut b.hle);
    let (mut turns, mut held, mut thrown, mut obj) = (Vec::new(), Vec::new(), Vec::new(), 0u32);
    let (port, orig) = b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[TRIANGLE]), 1), (0, 90)], |g, h| {
        let hier = h.read_u32(p + 0x154);
        let n = h.read_i16(hier) as u32;
        let orig: Vec<[i32; 4]> = (0..n).map(|i| [0, 4, 8, 12].map(|c| h.read_i32(hier + 0x34 + 40 * i + 4 + c))).collect();
        let port: Vec<[i32; 4]> = g.player.animator.turns(&g.player.hierarchy).iter().map(|q| q.unwrap_or([0; 4])).collect();
        turns.push(orig == port);
        let o = h.read_u32(p + 0x1f0);
        if o != 0 {
            obj = o;
        }
        let at = g.things[k].pos;
        let mine = [at.x as i32, at.y as i32, at.z as i32];
        let theirs = [0, 4, 8].map(|i| h.read_i32(obj + 0x18 + i));
        if g.carried().is_some() {
            held.push(mine);
        } else if obj != 0 && h.read_u32(obj + 0x90) != 0 {
            thrown.push((mine, theirs));
        }
    });
    let off = turns.iter().enumerate().filter(|(f, same)| *f >= 8 && !**same).count();
    // From the grab's own frame, as each draw places it.
    let placed = placed.borrow().clone();
    let carried: Vec<([i32; 3], [i32; 3])> = held.iter().copied().zip(placed).collect();
    let across = carried.iter().filter(|(m, t)| m[0] != t[0] || m[2] != t[2]).count();
    let height = carried.iter().map(|(m, t)| (m[1] - t[1]).abs()).max().unwrap_or(0);
    let flown = thrown.iter().filter(|(m, t)| m != t).count();
    eprintln!(
        "turns off {off} of {}, carried {} frames: across off {across}, height off at most {height}; flight {flown} of {} off",
        turns.len(),
        carried.len(),
        thrown.len()
    );
    assert_eq!((off, across), (0, 0), "the original's turns and place");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height, 0, "the original's height");
    assert!(thrown.len() >= 60 && flown == 0, "the flight the original's: {:?}", thrown.iter().find(|(m, t)| m != t));
    assert_eq!(camera_off(&port, &orig), 0, "the follow camera's heading");
}

/// Level 6 as Chuckie (character 2): box 1016 picked up off its shelf,
/// carried into the step ahead (the box held widens his box, 0x80067ba4,
/// and he bumps where the original does), and Circle again. The states whose word has 32 move the body
/// by their root keys across (the pick-up steps him in and back, the
/// put-down back), and the box rides the hand as each draw places it. The put-down
/// plays out without its event (2048), so the box stays in hand (0x80060910
/// lets go only then). Body and box on the original's places every frame.
#[test]
#[ignore]
fn level_6_chuckie_box_as_the_original() {
    use rugrats::input::CIRCLE;
    let Some(mut b) = Both::open(Boot { level: 6, character: Some(2), ..Default::default() }) else { return };
    let k = b.game.things.iter().position(|t| t.template == 1016 && (t.pos.x as i32, t.pos.z as i32) == (42741, 12863)).expect("box 8");
    b.put([42741, -1378, 12163], 0, 6);
    // The camera behind on both (the put leaves the original's where it
    // was).
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let placed = placements(&mut b.hle);
    let mut mine = Vec::new();
    let (port, orig) = b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[rugrats::input::UP]), 30), (bits(&[CIRCLE]), 1), (0, 24)], |g, _| {
        if g.carried().is_some() {
            let at = g.things[k].pos;
            mine.push([at.x as i32, at.y as i32, at.z as i32]);
        }
    });
    // (As each draw places it, from the grab's own frame.)
    let held: Vec<([i32; 3], [i32; 3])> = mine.into_iter().zip(placed.borrow().iter().copied()).collect();
    let worst = report("box", &port, &orig);
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    eprintln!("box held {} frames, {} off", held.len(), held.iter().filter(|(m, t)| m != t).count());
    // (Carried into the step, the box in the body's box bumps him where the
    // original's bumps; the bumps' back-steps drift across by up to 5.)
    let bumps = |seen: &[Seen]| seen.iter().enumerate().filter(|(_, s)| s.state == 9).map(|(f, _)| f).collect::<Vec<_>>();
    assert!(!bumps(&orig).is_empty() && bumps(&port) == bumps(&orig), "the bumps' frames");
    assert!(worst <= 5, "the body's places, {worst} off");
    let far = held.iter().map(|(m, t)| (0..3).map(|k| (m[k] - t[k]).abs()).max().unwrap_or(0)).max().unwrap_or(0);
    assert!(held.len() >= 50 && far <= 5, "the box's places, {far} off: {:?}", held.iter().find(|(m, t)| m != t));
    assert_eq!(b.game.carried(), Some(k), "still in hand");
    let cam = port.iter().zip(&orig).map(|(p, o)| ((p.camera - o.camera + 2048).rem_euclid(4096) - 2048).abs()).max().unwrap_or(0);
    assert!(cam <= 2, "the follow camera's heading, {cam} off");
}

/// Level 9 past cine 17 (where the port starts): ball 3 (template 777)
/// picked up and thrown away from Mr. Friend. Its template's +0x30 of 1
/// makes the throw roll (flag 0x10000000, 0x80086ef0): the segment bent
/// along the floor, the centre 10 over it, friction 15 a frame
/// (0x80086cd0). The ball on the original's place every frame of the
/// roll, and at rest where it stops.
#[test]
#[ignore]
fn level_9_ball_rolls_as_the_original() {
    use rugrats::input::{CIRCLE, TRIANGLE};
    let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..900 {
        b.hle.frame(0).unwrap();
    }
    let k = b
        .game
        .things
        .iter()
        .position(|t| t.template == 777 && (t.pos.x as i32, t.pos.z as i32) == (30003, -3772))
        .expect("ball 3");
    b.put([30003, 7947, -4472], 0, 6);
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let p = rugrats_hle::play::PLAYER;
    let (mut rolled, mut obj) = (Vec::new(), 0u32);
    let (port, orig) = b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[TRIANGLE]), 1), (0, 120)], |g, h| {
        let o = h.read_u32(p + 0x1f0);
        if o != 0 {
            obj = o;
        }
        if g.carried().is_none() && obj != 0 {
            let at = g.things[k].pos;
            rolled.push(([at.x as i32, at.y as i32, at.z as i32], [0, 4, 8].map(|i| h.read_i32(obj + 0x18 + i))));
        }
    });
    let off = rolled.iter().filter(|(m, t)| m != t).count();
    eprintln!("ball rolled {} frames, {off} off; at rest {:?}", rolled.len(), rolled.last());
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert!(rolled.len() >= 60 && off == 0, "the ball's places: {:?}", rolled.iter().find(|(m, t)| m != t));
    assert_eq!(b.game.things[k].state, rugrats::world::game::ThingState::Rest, "at rest");
    assert_eq!(camera_off(&port, &orig), 0, "the follow camera's heading");
}

/// Level 9 past cine 17: ball 3 picked up and put down again (Circle,
/// state 8, 0x80060910). The put-down is a flight as a throw is
/// (0x80062ac4: (0, 0, 4096) turned by the player, speed 10), and the
/// ball's template makes it roll: the ball on the original's place every
/// frame from the put-down to where it stops.
#[test]
#[ignore]
fn level_9_ball_put_down_as_the_original() {
    use rugrats::input::CIRCLE;
    let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..900 {
        b.hle.frame(0).unwrap();
    }
    let k = b
        .game
        .things
        .iter()
        .position(|t| t.template == 777 && (t.pos.x as i32, t.pos.z as i32) == (30003, -3772))
        .expect("ball 3");
    b.put([30003, 7947, -4472], 0, 6);
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let p = rugrats_hle::play::PLAYER;
    let (mut moved, mut obj) = (Vec::new(), 0u32);
    let (port, orig) = b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[CIRCLE]), 1), (0, 90)], |g, h| {
        let o = h.read_u32(p + 0x1f0);
        if o != 0 {
            obj = o;
        }
        if g.carried().is_none() && obj != 0 {
            let at = g.things[k].pos;
            moved.push(([at.x as i32, at.y as i32, at.z as i32], [0, 4, 8].map(|i| h.read_i32(obj + 0x18 + i))));
        }
    });
    let off = moved.iter().filter(|(m, t)| m != t).count();
    eprintln!("ball put down: {} frames, {off} off; at rest {:?}", moved.len(), moved.last());
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert!(moved.len() >= 30 && off == 0, "the ball's places: {:?}", moved.iter().find(|(m, t)| m != t));
    assert_eq!(b.game.things[k].state, rugrats::world::game::ThingState::Rest, "at rest");
    assert_eq!(camera_off(&port, &orig), 0, "the follow camera's heading");
}

/// Level 16: Tommy put in room 4, the booth, starts the pie toss
/// (0x8007e680: on the counter, pie 0 in hand, the six pies' home the
/// pie spot). Its two messages closed, a pie thrown (Triangle): the
/// throw's flight is 0x80061c98's (flags 0x0a070400: gravity 1, done at a
/// hit; (0, 35, 1024) turned by him, 225 with Angelica within 13000) and
/// pie 1 is in his hand at once (0x8004eb10). The pie on the original's
/// place every frame of its flight, done where the original's is.
#[test]
#[ignore]
fn level_16_pie_flies_as_the_original() {
    use rugrats::input::{CROSS, TRIANGLE};
    let Some(mut b) = Both::open(Boot { level: 16, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    b.put([33500, -1269, -8141], 2048, 4);
    let pies: Vec<usize> = (0..b.game.things.len()).filter(|&k| b.game.things[k].template == 564).collect();
    assert_eq!(pies.len(), 6, "six pies");
    let theirs: Vec<u32> = (0..6).map(|i| b.hle.read_u32(0x800e_2770 + 4 * i)).collect();
    assert_eq!(b.game.carried(), Some(pies[0]), "pie 0 in hand");
    // (No pie is in a room's list, the one in hand drawn with the player
    // (0x8003205c: +0x1f0's draw); the port's others are out of the
    // world.)
    let listed = objects(&mut b.hle);
    assert!(theirs.iter().all(|o| !listed.contains(o)), "the original's pies listed");
    assert!(pies.iter().enumerate().all(|(i, &k)| b.game.things[k].hidden == (i != 0)), "the port's pies out of the world");
    let p = rugrats_hle::play::PLAYER;
    b.script(&[(0, 10), (bits(&[CROSS]), 1), (0, 10), (bits(&[CROSS]), 1), (0, 10)]);
    let mut flown = Vec::new();
    let mut hands = Vec::new();
    let mut each = |g: &Game, h: &mut Play| {
        let mine = g.things[pies[0]].pos;
        let o = theirs[0];
        let done = h.read_u8(o + 0x6b) == 255 || g.things[pies[0]].hidden;
        if g.carried() != Some(pies[0]) && h.read_u32(p + 0x1f0) != o {
            flown.push(([mine.x as i32, mine.y as i32, mine.z as i32], [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i)), done));
        }
        hands.push((g.carried().and_then(|k| pies.iter().position(|&x| x == k)), theirs.iter().position(|&x| x == h.read_u32(p + 0x1f0))));
    };
    b.script_with(&[(bits(&[TRIANGLE]), 1), (0, 10)], &mut each);
    // (The throw's messages hold the world until Cross: the original's
    // 0x800c3eb9.)
    for _ in 0..4 {
        b.script_with(&[(0, 10), (bits(&[CROSS]), 1), (0, 2)], &mut each);
    }
    b.script_with(&[(0, 40)], &mut each);
    let off = flown.iter().filter(|(m, t, _)| m != t).count();
    eprintln!("pie flew {} frames, {off} off: {:?}", flown.len(), flown.iter().find(|(m, t, _)| m != t));
    assert!(flown.len() >= 5 && off == 0, "the pie's places: {:?}", flown);
    assert!(hands.iter().all(|(m, t)| m == t), "the pie in hand: {hands:?}");
    // (The original's +0x6b is 255 the frame its flight ends; the port's
    // pie leaves its room.)
    assert!(flown.iter().any(|f| f.2), "done at its hit");
    assert!(b.game.things[pies[0]].hidden, "the pie gone from its room");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
}

/// Level 3: Tommy put into room 19 (cine 9, then the pie toss,
/// 0x8007bc60: on the counter, frozen, pie 0 in hand), given the
/// original's heading (cine 9 leaves the original's otherwise), a pie
/// thrown at Larry and Steve: speed 450 with Larry over 13000 away in 3D,
/// gravity 1, met as a segment of its radius. The pie is the original's
/// on every frame until the original's first lag frame (its time,
/// 0x800c3e98, not 10 on), and a strike on a brother is the original's.
#[test]
#[ignore]
fn level_3_pie_flies_as_the_original() {
    use rugrats::input::TRIANGLE;
    let Some(mut b) = Both::open(Boot { level: 3, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    b.put([41000, 600, -5000], 2048, 1);
    let mut f = 0;
    while b.hle.read_u32(0x800c_56a8) & 8 == 0 && f < 600 {
        b.script(&[(0, 1)]);
        if let Some(n) = b.game.cine.take() {
            b.game.cine_finished(n);
        }
        f += 1;
    }
    assert!(b.game.pie_toss.is_some(), "the port's toss started");
    let p = rugrats_hle::play::PLAYER;
    let h0 = b.hle.read_i16(p + 0x24) as i32 & 4095;
    b.game.player.heading = h0;
    b.game.player.target = h0;
    b.script(&[(0, 40)]);
    // (And its animation, which the skipped cine left elsewhere.)
    b.sync_animation();
    b.game.player.animator.mark_asked(rugrats::actors::player::anim::CARRY_IDLE);
    b.tracks_off.clear();
    let pies: Vec<usize> = (0..b.game.things.len()).filter(|&k| b.game.things[k].template == 564).collect();
    let theirs: Vec<u32> = (0..6).map(|i| b.hle.read_u32(0x800e_2770 + 4 * i)).collect();
    let mut flown = Vec::new();
    let mut clock = b.hle.read_i32(0x800c_3e98);
    let mut lagged = false;
    let (start, mut seen, mut lag_at) = (b.frames, 0, usize::MAX);
    b.script_with(&[(bits(&[TRIANGLE]), 1), (0, 50)], |g, h| {
        let t = h.read_i32(0x800c_3e98);
        if t - clock != 10 && !lagged {
            lag_at = seen;
        }
        lagged |= t - clock != 10;
        clock = t;
        seen += 1;
        let o = theirs[0];
        if !lagged && g.carried() != Some(pies[0]) && h.read_u32(p + 0x1f0) != o {
            let mine = g.things[pies[0]].pos;
            flown.push(([mine.x as i32, mine.y as i32, mine.z as i32], [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i))));
        }
    });
    let off = flown.iter().filter(|(m, t)| m != t).count();
    eprintln!("pie flew {} frames before a lag, {off} off: {:?}", flown.len(), flown.iter().find(|(m, t)| m != t));
    assert!(flown.len() >= 5 && off == 0, "the pie's places: {:?}", flown);
    let before: Vec<usize> = b.tracks_off.iter().copied().filter(|&f| f < start + lag_at).collect();
    assert!(before.is_empty(), "the original's animation tracks before its lag, off at {before:?}");
}

/// The original's objects in the rooms' lists (0x800c4a8c).
/// Where the original's character draw puts the player's carried object
/// each frame (0x8005ff88 from 0x8003205c, read just after the call): a
/// frame that runs past the sample (the hub, level 6) has its object's
/// place as the draw before left it there.
fn placements(p: &mut Play) -> Rc<std::cell::RefCell<Vec<[i32; 3]>>> {
    use rrt::emu::BusExt;
    let placed: Rc<std::cell::RefCell<Vec<[i32; 3]>>> = Rc::default();
    let c = placed.clone();
    p.hle.m.hook(0x8003_23b0, move |m| {
        let o = m.bus.read_u32(rugrats_hle::play::PLAYER + 0x1f0).unwrap_or(0);
        if o != 0 {
            c.borrow_mut().push([0, 4, 8].map(|k| m.bus.read_u32(o + 0x18 + k).unwrap_or(0) as i32));
        }
        rrt::emu::HookResult::Continue
    });
    placed
}

fn objects(p: &mut Play) -> Vec<u32> {
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    let mut out = Vec::new();
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                out.push(p.read_u32(node + 8));
                node = p.read_u32(node + 4);
            }
        }
    }
    out
}

/// Level 9 past cine 17: Tommy put far off, Mr. Friend (46) chases him.
/// Once his first wait is over (he is given the original's place then),
/// each frame his stick (0x80082264) turns him toward Tommy by at most
/// 512 (0x800820e0), looks ahead (0x800825fc: three casts to Tommy, the
/// ways round when blocked; a side open turns him a further 512), sets
/// the way to his speed (row 1 of the table in level 9, 2 doubled past
/// twice his range, by the game's distance 0x8001f57c) and steps along
/// it; then the body turns him by 160. On the original's place and
/// heading every frame.
#[test]
#[ignore]
fn level_9_friend_chases_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..900 {
        b.hle.frame(0).unwrap();
    }
    let k = b.game.npcs.iter().position(|n| n.id == 46).expect("Mr. Friend");
    let at = b.game.npcs[k].pos;
    let mut bodies = Vec::new();
    for o in objects(&mut b.hle) {
        if b.hle.read_u32(o + 0x154) != 0 {
            let d: i32 = (0..3).map(|i| (b.hle.read_i32(o + 0x18 + 4 * i as u32) - at[i]).abs()).sum();
            bodies.push((d, o));
        }
    }
    let o = bodies.into_iter().min().expect("his object").1;
    b.put([30003, 7947, -4472], 0, 6);
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let n = &mut b.game.npcs[k];
    n.pos = [0, 4, 8].map(|i| b.hle.read_i32(o + 0x18 + i));
    n.heading = b.hle.read_i16(o + 0x24) as i32 & 4095;
    n.sync_body(b.hle.read_u8(o + 0x18c), b.hle.read_i32(o + 0x184));
    let mut seen = Vec::new();
    let (port, orig) = b.script_with(&[(0, 520)], |g, h| {
        let n = &g.npcs[k];
        let theirs = ([0, 8].map(|i| h.read_i32(o + 0x18 + i)), h.read_i16(o + 0x24) as i32 & 4095);
        seen.push((([n.pos[0], n.pos[2]], n.heading), theirs));
    });
    let off = seen.iter().filter(|(m, t)| m != t).count();
    report("friend", &port, &orig);
    eprintln!("Mr. Friend chased {} frames, {off} off", seen.len());
    assert_eq!(off, 0, "his place and heading: {:?}", seen.iter().position(|(m, t)| m != t).map(|f| (f, seen[f])));
    assert_eq!(camera_off(&port, &orig), 0, "the follow camera's heading");
}

/// Level 9 past cine 17, Tommy where the cinematic leaves him, within Mr.
/// Friend's bite. His bite cycle (0x80084084): mode 4 stops him and asks
/// mode 5 within half his range with the two bodies level; mode 5 asks the
/// bite (37) and gives the Damage, without a body update; mode 1 plays it
/// out; mode 0 goes back to 4, again without one. Given the original's
/// mode and animation once, his tracks are the original's every frame, and
/// Tommy's health falls on the same frames.
#[test]
#[ignore]
fn level_9_friend_bites_as_the_original() {
    use rugrats::actors::npc::Bite;
    let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..900 {
        b.hle.frame(0).unwrap();
    }
    let k = b.game.npcs.iter().position(|n| n.id == 46).expect("Mr. Friend");
    let at = b.game.npcs[k].pos;
    let mut bodies = Vec::new();
    // (The player's own body, 0x800e2b48, is no character: it has the
    // id 1492 as Tommy has.)
    for o in objects(&mut b.hle) {
        if b.hle.read_u32(o + 0x154) != 0 && o != 0x800e_2b48 {
            let d: i32 = (0..3).map(|i| (b.hle.read_i32(o + 0x18 + 4 * i as u32) - at[i]).abs()).sum();
            bodies.push((d, o));
        }
    }
    let o = bodies.into_iter().min().expect("his object").1;
    let p = rugrats_hle::play::PLAYER;
    let tracks = |h: &mut Play| -> [[i16; 6]; 4] { std::array::from_fn(|j| std::array::from_fn(|i| h.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32))) };
    // His mode, animation, place and heading as the original's stand.
    {
        let t = tracks(&mut b.hle);
        let speed = b.hle.read_i16(o + 0xe4) as i32;
        let after = (b.hle.read_u8(o + 0xe6), b.hle.read_u8(o + 0xe7));
        let fidget = (b.hle.read_i16(o + 0xf0), b.hle.read_i16(o + 0xf2) as i32);
        let hier = b.hle.read_u32(o + 0x154);
        let parts = b.hle.read_i16(hier) as u32;
        let mut pose = vec![None; 32];
        for (channel, slot) in pose.iter_mut().enumerate() {
            let part = b.hle.read_u8(hier + 0x14 + channel as u32) as u32;
            if part < parts {
                let q = [0, 4, 8, 12].map(|c| b.hle.read_i32(hier + 0x34 + 40 * part + 4 + c));
                if q != [0; 4] {
                    *slot = Some(q);
                }
            }
        }
        let mode = b.hle.read_u8(o + 0x699);
        let g = &mut b.game;
        let n = &mut g.npcs[k];
        n.animator.load_record(t, speed, after, fidget, pose, &g.anims);
        n.pos = [0, 4, 8].map(|i| b.hle.read_i32(o + 0x18 + i));
        n.heading = b.hle.read_i16(o + 0x24) as i32 & 4095;
        n.sync_body(b.hle.read_u8(o + 0x18c), b.hle.read_i32(o + 0x184));
        n.bite = match mode {
            5 => Bite::Strike,
            1 => Bite::Recover,
            0 => Bite::Resume,
            _ => Bite::Chase,
        };
        g.player.health = b.hle.read_i16(p + 0x1a4) as i32;
    }
    b.sync_animation();
    let look = [0, 4, 8].map(|i| b.hle.read_i32(0x800d_e454 + i));
    b.game.set_camera_look(look);
    let (mut seen, mut health) = (Vec::new(), Vec::new());
    let (port, orig) = b.script_with(&[(0, 120)], |g, h| {
        let n = &g.npcs[k];
        let theirs = tracks(h);
        let mine = n.animator.record();
        let same = theirs.iter().zip(&mine).all(|(o, p)| o[..4] == p[..4] && (o[5] & 0xff) == (p[5] & 0xff));
        if std::env::var_os("TRACKS").is_some() {
            eprintln!("    friend {} {:?} | {:?} {:?} mode {}", if same { "same" } else { "DIFF" }, theirs[0], mine[0], n.bite, h.read_u8(o + 0x699));
        }
        seen.push(same);
        health.push((g.player.health, h.read_i16(p + 0x1a4) as i32));
    });
    let off: Vec<usize> = seen.iter().enumerate().filter(|(_, s)| !**s).map(|(f, _)| f).collect();
    eprintln!("Mr. Friend's tracks off at {off:?}; Tommy's health {health:?}");
    assert!(off.is_empty(), "his tracks, off at {off:?}");
    assert!(health.iter().all(|(m, t)| m == t), "Tommy's health: {health:?}");
    assert!(b.tracks_off.is_empty(), "Tommy's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(report("bitten", &port, &orig), 0, "Tommy's place");
    assert_eq!(camera_off(&port, &orig), 0, "the camera's heading");
}

/// Mr. Friend (46) in level 9 given the original's place, heading,
/// animation and mode; his object.
fn sync_friend(b: &mut Both) -> (usize, u32) {
    sync_npc(b, 46)
}

/// Character `id` given the original's place, heading, animation, health,
/// body state, mode (a follower's bite, a post walker's posts); its index
/// and its object.
fn sync_npc(b: &mut Both, id: u16) -> (usize, u32) {
    use rugrats::actors::npc::{Bite, Mode};
    let k = b.game.npcs.iter().position(|n| n.id == id).expect("the character");
    let at = b.game.npcs[k].pos;
    // Its own id (+0x38) first, else the body nearest.
    let mut bodies = Vec::new();
    // (The player's own body, 0x800e2b48, is no character: it has the
    // id 1492 as Tommy has.)
    for o in objects(&mut b.hle) {
        if b.hle.read_u32(o + 0x154) != 0 && o != 0x800e_2b48 {
            let d: i32 = (0..3).map(|i| (b.hle.read_i32(o + 0x18 + 4 * i as u32) - at[i]).abs()).sum();
            let other = (b.hle.read_i16(o + 0x38) as u16 != id) as i32;
            bodies.push(((other, d), o));
        }
    }
    let o = bodies.into_iter().min().expect("his object").1;
    let t: [[i16; 6]; 4] = std::array::from_fn(|j| std::array::from_fn(|i| b.hle.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32)));
    let speed = b.hle.read_i16(o + 0xe4) as i32;
    let after = (b.hle.read_u8(o + 0xe6), b.hle.read_u8(o + 0xe7));
    let fidget = (b.hle.read_i16(o + 0xf0), b.hle.read_i16(o + 0xf2) as i32);
    let hier = b.hle.read_u32(o + 0x154);
    let parts = b.hle.read_i16(hier) as u32;
    let mut pose = vec![None; 32];
    for (channel, slot) in pose.iter_mut().enumerate() {
        let part = b.hle.read_u8(hier + 0x14 + channel as u32) as u32;
        if part < parts {
            let q = [0, 4, 8, 12].map(|c| b.hle.read_i32(hier + 0x34 + 40 * part + 4 + c));
            if q != [0; 4] {
                *slot = Some(q);
            }
        }
    }
    let mode = b.hle.read_u8(o + 0x699);
    let (cur, prev) = (b.hle.read_u8(o + 0x695), b.hle.read_u8(o + 0x694));
    let g = &mut b.game;
    let n = &mut g.npcs[k];
    n.animator.load_record(t, speed, after, fidget, pose, &g.anims);
    // What its base state was asked as (the logical animation).
    // (Its after state, +0xe6, playing is standing: the idle asked.)
    if let Some(a) = rugrats::actors::animator::logical_of(t[0][0]).filter(|&a| a != after.0 as i16) {
        n.animator.mark_asked(a);
    }
    let own_y = n.pos[1];
    n.pos = [0, 4, 8].map(|i| b.hle.read_i32(o + 0x18 + i));
    // (The original's height has the root's lift in it; the port's place
    // is the floor (+0x148) and the feet, its lift added where drawn.)
    // (One never stepped keeps its own height there, and a thing that
    // is no body has none.)
    let floor = b.hle.read_i32(o + 0x148);
    if n.unsettled {
        // (Never settled, 0x800452f8: its place keeps its own height, the
        // lift its hover: the original's height less it.)
        // Its root goes on from there as the original's set up does.
        n.root.y = n.pos[1] - own_y;
        n.pos[1] = own_y;
        n.root.set_up(&n.animator, &g.anims, n.root_scale(), b.hle.read_i16(o + 0xde) as i32);
    } else if floor != n.pos[1] && (floor + n.feet() - n.pos[1]).abs() < 1000 {
        n.pos[1] = floor + n.feet();
    }
    n.heading = b.hle.read_i16(o + 0x24) as i32 & 4095;
    n.health = b.hle.read_i16(o + 0x1a4) as i32;
    n.sync_body(b.hle.read_u8(o + 0x18c), b.hle.read_i32(o + 0x184));
    n.bite = match mode {
        5 => Bite::Strike,
        1 => Bite::Recover,
        0 => Bite::Resume,
        _ => Bite::Chase,
    };
    if let Mode::Posts { .. } = n.mode {
        n.mode = Mode::Posts { prev, cur };
    }
    (k, o)
}

/// Level 13 past cine 6: Angelica's race, Tommy left standing. Given the
/// original's Angelica once, she runs her posts as the original does: the
/// shared body's step (a single probe, 0x80064838; a held one lets her by,
/// +0x688 bit 0x1000), the probe ahead and its fall down the stairs
/// (0x800645d8, state 4: the running launch, the character's push 12),
/// the landing, the waits at posts 13 and 5 (mode 7: no body update the
/// frame it starts, the wait's own animation, VSync), the random post (20:
/// rand & 3), to the goal post. Her place and heading every frame to it,
/// and her animation tracks once the sync's first frames are past.
#[test]
#[ignore]
fn level_13_angelica_races_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 13, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..600 {
        b.hle.frame(0).unwrap();
    }
    let (k, o) = sync_npc(&mut b, 600);
    b.sync_animation();
    let mut seen = Vec::new();
    b.script_with(&[(0, 940)], |g, h| {
        let n = &g.npcs[k];
        let theirs: [[i16; 6]; 4] = std::array::from_fn(|j| std::array::from_fn(|i| h.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32)));
        let mine = n.animator.record();
        let tracks = theirs.iter().zip(&mine).all(|(a, c)| a[..4] == c[..4] && (a[5] & 0xff) == (c[5] & 0xff));
        let place = [n.pos[0], n.pos[2], n.heading & 4095] == [h.read_i32(o + 0x18), h.read_i32(o + 0x20), h.read_i16(o + 0x24) as i32 & 4095];
        if std::env::var_os("TRACKS").is_some() && seen.len() % 50 == 25 {
            eprintln!("    angelica {:?} | {:?}", theirs, mine);
        }
        seen.push((place, tracks));
    });
    let place_off: Vec<usize> = seen.iter().enumerate().filter(|(_, s)| !s.0).map(|(f, _)| f).collect();
    let tracks_off: Vec<usize> = seen.iter().enumerate().filter(|(f, s)| *f >= 20 && !s.1).map(|(f, _)| f).collect();
    eprintln!("Angelica: place off at {place_off:?}, tracks off at {tracks_off:?}");
    assert!(place_off.is_empty(), "her place and heading, off at {place_off:?}");
    assert!(tracks_off.is_empty(), "her animation tracks, off at {tracks_off:?}");
}

/// Level 9: Tommy picks up ball 2 and throws it at Mr. Friend coming
/// for him. The aim (0x80059b34), the roll (0x80086ef0) and its meeting
/// with him, his stagger (0x800834b0: logical 8 at 2048, mode 1, his
/// callbacks off) and his chase after. Given the original's Friend once
/// (place, animation, mode), the ball's places, his tracks, place and
/// health, and Tommy, are the original's every frame.
#[test]
#[ignore]
fn level_9_ball_at_friend_as_the_original() {
    use rugrats::input::{CIRCLE, R1, TRIANGLE};
    let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
    for _ in 0..900 {
        b.hle.frame(0).unwrap();
    }
    let ball = b.game.things.iter().position(|t| t.template == 777 && (t.pos.x as i32, t.pos.z as i32) == (31204, -11085)).expect("ball 2");
    b.put([31846, 7947, -10807], 2805, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let (k, o) = sync_friend(&mut b);
    if std::env::var_os("AIMDBG").is_some() {
        use rrt::emu::{BusExt, Cpu};
        b.hle.hle.m.hook(0x8006_1e9c, |c| {
            let sp = c.cpu.reg(29) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let p = rugrats_hle::play::PLAYER;
            let t = w(p + 0x1f4) as u32;
            println!("    AIM orig way [{}, {}, {}] player [{}, {}, {}] target [{}, {}, {}]", w(sp + 16), w(sp + 20), w(sp + 24), w(p + 0x18), w(p + 0x1c), w(p + 0x20), w(t + 0x18), w(t + 0x1c), w(t + 0x20));
            rrt::emu::HookResult::Continue
        });
        b.hle.hle.m.hook(0x8006_1f38, |c| {
            let sp = c.cpu.reg(29) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            println!("    AIM orig dir [{}, {}, {}] angles {:#x}", w(sp + 16), w(sp + 20), w(sp + 24), w(sp + 32));
            rrt::emu::HookResult::Continue
        });
    }
    let p = rugrats_hle::play::PLAYER;
    let (mut rolled, mut friend, mut obj) = (Vec::new(), Vec::new(), 0u32);
    let (port, orig) = b.script_with(&[(bits(&[CIRCLE]), 1), (0, 40), (bits(&[TRIANGLE]), 1), (0, 90)], |g, h| {
        let ob = h.read_u32(p + 0x1f0);
        if ob != 0 {
            obj = ob;
        }
        if g.carried().is_none() && obj != 0 {
            let at = g.things[ball].pos;
            rolled.push(([at.x as i32, at.y as i32, at.z as i32], [0, 4, 8].map(|i| h.read_i32(obj + 0x18 + i))));
            if std::env::var_os("TRACKS").is_some() && rolled.len() <= 8 {
                let fl = h.read_u32(obj + 0x90);
                let v = if fl != 0 { [0, 4, 8].map(|i| h.read_i32(fl + 0x3c + i)) } else { [0; 3] };
                eprintln!("    v port {:?} orig {v:?}", g.things[ball].state);
            }
        }
        let n = &g.npcs[k];
        let theirs: [[i16; 6]; 4] = std::array::from_fn(|j| std::array::from_fn(|i| h.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32)));
        let mine = n.animator.record();
        let same = theirs.iter().zip(&mine).all(|(a, c)| a[..4] == c[..4] && (a[5] & 0xff) == (c[5] & 0xff));
        if std::env::var_os("TRACKS").is_some() {
            eprintln!(
                "    friend {} {:?} | {:?} {:?} mode {} speed {} | {} at {:?} h {} | {:?} h {}",
                if same { "same" } else { "DIFF" },
                theirs[0],
                mine[0],
                n.bite,
                h.read_u8(o + 0x699),
                h.read_i32(o + 0x184),
                n.speed,
                [0, 8].map(|i| h.read_i32(o + 0x18 + i)),
                h.read_i16(o + 0x24),
                [n.pos[0], n.pos[2]],
                n.heading
            );
        }
        let place = [n.pos[0], n.pos[1] + n.root.y, n.pos[2], n.heading] == [h.read_i32(o + 0x18), h.read_i32(o + 0x1c), h.read_i32(o + 0x20), h.read_i16(o + 0x24) as i32 & 4095];
        if !place && std::env::var_os("TRACKS").is_some() {
            eprintln!("    friend place port {:?} {} orig {:?} {}", [n.pos[0], n.pos[1] + n.root.y, n.pos[2]], n.heading, [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i)), h.read_i16(o + 0x24));
        }
        friend.push((same, place, (n.health, h.read_i16(o + 0x1a4) as i32), h.read_u8(o + 0x699)));
    });
    let roll_off = rolled.iter().filter(|(m, t)| m != t).count();
    if std::env::var_os("TRACKS").is_some() {
        for (f, r) in rolled.iter().enumerate().take(8) {
            eprintln!("    ball {f}: {r:?}");
        }
    }
    let first = friend.iter().position(|f| !(f.0 && f.1 && f.2.0 == f.2.1));
    eprintln!("ball {} frames, {roll_off} off; Friend first off {:?}", rolled.len(), first.map(|f| (f, friend[f])));
    assert_eq!(report("thrown", &port, &orig), 0, "Tommy's place");
    assert!(b.tracks_off.is_empty(), "Tommy's tracks, off at {:?}", b.tracks_off);
    assert!(rolled.len() >= 30 && roll_off == 0, "the ball: {:?}", rolled.iter().position(|(m, t)| m != t).map(|f| (f, rolled[f])));
    assert_eq!(first, None, "Mr. Friend: tracks, place, health");
    assert_eq!(camera_off(&port, &orig), 0, "the camera's heading");
}

/// Level 12: the flashlight shone (Triangle held) where cine 66 leaves
/// Tommy. Each frame's light is a shot (0x80091834, type 1) from hand
/// slot 15 turned (0, 0, 350), stepped once along the facing (4096) and
/// drawn (0x80090e70) from its start to its end: both the original's.
#[test]
#[ignore]
fn level_12_light_shot_as_the_original() {
    use rugrats::input::TRIANGLE;
    let Some(mut b) = Both::open(Boot { level: 12, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    b.put([29611, -5365, 15428], 1024, 4);
    // (The flashlight is in hand as +0x1f0 alone: the stand's carry asked,
    // which the sync forgets.)
    b.sync_animation();
    b.game.player.animator.mark_asked(rugrats::actors::player::anim::CARRY_IDLE);
    b.tracks_off.clear();
    let theirs: Rc<std::cell::RefCell<Vec<([i32; 3], [i32; 3])>>> = Rc::default();
    {
        use rrt::emu::{BusExt, Cpu};
        let t = theirs.clone();
        b.hle.hle.m.hook(0x8009_0e70, move |c| {
            let r = c.cpu.reg(5) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            let (from, to) = ([w(r + 0x14), w(r + 0x18), w(r + 0x1c)], [w(r + 4), w(r + 8), w(r + 12)]);
            t.borrow_mut().push((from, to));
            rrt::emu::HookResult::Continue
        });
    }
    let mut mine = Vec::new();
    let drawn = theirs.clone();
    b.script_with(&[(bits(&[TRIANGLE]), 20), (0, 4)], |g, _| {
        // (On the frames the original drew one.)
        if drawn.borrow().len() > mine.len() {
            mine.extend(g.shots.first().copied());
        }
    });
    let theirs = theirs.borrow().clone();
    eprintln!("shots: port {} orig {}\n  port {:?}\n  orig {:?}", mine.len(), theirs.len(), mine.first(), theirs.first());
    let off = mine.iter().zip(&theirs).filter(|(m, t)| m != t).count();
    assert!(theirs.len() >= 18 && mine.len() == theirs.len() && off == 0, "the shots: {:?}", mine.iter().zip(&theirs).find(|(m, t)| m != t));
    assert!(b.tracks_off.is_empty(), "Tommy's tracks, off at {:?}", b.tracks_off);
}

/// Level 12: a Shadow monster in the kitchen (3339) in the light. Tommy
/// put on its floor 2000 short of it, the monster given the original's
/// place; Triangle held. Tommy's aim target (+0x1f4, 0x80059b34) is the
/// monster, so the light goes toward it (0x80075e4c) while it has a size
/// (+0x36): the shot meets its box (0x80090554 -> 0x8006a9cc ->
/// 0x8006a714) and strikes it (0x800511a8 case 41: mode 10, the box
/// zeroed); it shrinks 250 a frame and comes back when its size is spent
/// (0x80084314). Each shot, the size every frame and the frame it comes
/// back are the original's.
#[test]
#[ignore]
fn level_12_light_strikes_as_the_original() {
    use rugrats::input::TRIANGLE;
    let Some(mut b) = Both::open(Boot { level: 12, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    let id: u16 = std::env::var("LIGHT_ID").ok().and_then(|v| v.parse().ok()).unwrap_or(3339);
    let (k, o) = sync_npc(&mut b, id);
    let at = b.game.npcs[k].pos;
    let floor = b.hle.read_i32(o + 0x148);
    let me = [at[0] - 2000, floor + b.game.player.feet(), at[2] + 300];
    b.put(me, 1024, 4);
    b.sync_animation();
    b.game.player.animator.mark_asked(rugrats::actors::player::anim::CARRY_IDLE);
    sync_npc(&mut b, id);
    let theirs: Rc<std::cell::RefCell<Vec<([i32; 3], [i32; 3])>>> = Rc::default();
    {
        use rrt::emu::{BusExt, Cpu};
        let t = theirs.clone();
        b.hle.hle.m.hook(0x8009_0e70, move |c| {
            let r = c.cpu.reg(5) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            t.borrow_mut().push(([w(r + 0x14), w(r + 0x18), w(r + 0x1c)], [w(r + 4), w(r + 8), w(r + 12)]));
            rrt::emu::HookResult::Continue
        });
    }
    let drawn = theirs.clone();
    // (The game's rand as the original's has it: 0x800c3f94.)
    b.game.rand.state = b.hle.read_u32(0x800c_3f94);
    let (mut mine, mut sizes) = (Vec::new(), Vec::new());
    b.script_with(&[(bits(&[TRIANGLE]), 24), (0, 2)], |g, h| {
        if drawn.borrow().len() > mine.len() {
            mine.extend(g.shots.first().copied());
        }
        let n = &g.npcs[k];
        sizes.push(((n.scale, matches!(n.mode, rugrats::actors::npc::Mode::Idle)), (h.read_i16(o + 0x36) as i32, h.read_u8(o + 0x699) == 10)));
    });
    let theirs = theirs.borrow().clone();
    eprintln!("shots {} / {}, sizes {sizes:?}", mine.len(), theirs.len());
    let off = mine.iter().zip(&theirs).filter(|(m, t)| m != t).count();
    assert!(theirs.len() >= 20 && mine.len() == theirs.len() && off == 0, "the shots: {:?}", mine.iter().zip(&theirs).enumerate().find(|(_, (m, t))| m != t));
    assert!(sizes.iter().any(|s| s.1.1) && sizes.iter().any(|s| s.1.0 == 4096 && !s.1.1 && sizes.iter().position(|x| x.1.1).is_some()), "struck and back");
    assert!(sizes.iter().all(|(m, t)| m == t), "the size and mode 10: {:?}", sizes.iter().enumerate().find(|(_, (m, t))| m != t));
}

/// Level 12 standing where cine 66 leaves Tommy, nothing held, the game's
/// rand (0x800c3f94) and the ambience's countdown (0x800c4cc0) given the
/// original's: the ambience (0x800818e0, last in the frame, 0x8001886c)
/// says the original's lines (0x8008da4c) on the original's frames.
#[test]
#[ignore]
fn level_12_ambience_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 12, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    b.put([29611, -5365, 15428], 1024, 4);
    b.game.rand.state = b.hle.read_u32(0x800c_3f94);
    b.game.ambience = b.hle.read_i32(0x800c_4cc0);
    let said: Rc<std::cell::RefCell<Vec<u16>>> = Rc::default();
    {
        use rrt::emu::Cpu;
        let s = said.clone();
        b.hle.hle.m.hook(0x8008_da4c, move |c| {
            s.borrow_mut().push(c.cpu.reg(4) as u16);
            rrt::emu::HookResult::Continue
        });
    }
    let (mut mine, mut theirs) = (Vec::new(), Vec::new());
    let mut f = 0;
    let heard = said.clone();
    b.script_with(&[(0, 1500)], |g, _| {
        f += 1;
        mine.extend(g.speech.iter().map(|&n| (f, n)));
        theirs.extend(heard.borrow_mut().drain(..).map(|n| (f, n)));
    });
    eprintln!("lines: port {mine:?}\n       orig {theirs:?}");
    assert!(theirs.len() >= 2 && mine == theirs, "the lines and their frames");
}

/// Level 12: the flashlight shone walking (Up and Triangle held) from
/// where the level's start leaves Tommy. A
/// frame whose animation event 2048 fired shines at speed 2048
/// (0x8005f4cc), its shot 20 times the length; the body's step adds
/// nothing. Every shot is the original's.
#[test]
#[ignore]
fn level_12_light_walking_as_the_original() {
    use rugrats::input::{TRIANGLE, UP};
    let Some(mut b) = Both::open(Boot { level: 12, ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    // (Where the boot leaves the original's, LP 202, both cameras behind.)
    let o = b.hle.player();
    b.put(o.pos, o.heading as i32 & 4095, 4);
    b.sync_animation();
    b.game.player.animator.mark_asked(rugrats::actors::player::anim::CARRY_IDLE);
    b.tracks_off.clear();
    let theirs: Rc<std::cell::RefCell<Vec<([i32; 3], [i32; 3])>>> = Rc::default();
    {
        use rrt::emu::{BusExt, Cpu};
        let t = theirs.clone();
        b.hle.hle.m.hook(0x8009_0e70, move |c| {
            let r = c.cpu.reg(5) as u32;
            let mut w = |a: u32| c.bus.read_u32(a).unwrap_or(0) as i32;
            t.borrow_mut().push(([w(r + 0x14), w(r + 0x18), w(r + 0x1c)], [w(r + 4), w(r + 8), w(r + 12)]));
            rrt::emu::HookResult::Continue
        });
    }
    let drawn = theirs.clone();
    let mut mine = Vec::new();
    let (port, orig) = b.script_with(&[(bits(&[UP, TRIANGLE]), 24), (0, 2)], |g, _| {
        if drawn.borrow().len() > mine.len() {
            mine.extend(g.shots.first().copied());
        }
    });
    let worst = report("walk", &port, &orig);
    let theirs = theirs.borrow().clone();
    let long = theirs.iter().filter(|(a, b)| (b[0] - a[0]).abs() + (b[2] - a[2]).abs() > 10000).count();
    eprintln!("shots {} / {}, {long} long", mine.len(), theirs.len());
    assert_eq!(worst, 0, "Tommy's place every frame");
    assert!(long > 0 && mine.len() == theirs.len(), "the long shots seen");
    assert!(mine.iter().zip(&theirs).all(|(m, t)| m == t), "the shots: {:?}", mine.iter().zip(&theirs).enumerate().find(|(_, (m, t))| m != t));
}

/// Level 6 as Chuckie: box 9 picked up, then put with it in hand short of
/// box 8 and run at it. The box held widens his box (0x80067ba4), so the
/// box ahead meets box 8: a ledge in reach he cannot climb carrying, run
/// into at full speed (0x800590d4: over the run's factor times 3933 >>
/// 12), bumps him (state 9, 0x8005e1f4); run at it again, the stick at a
/// slant (speed 16, no run) and with no ledge met, box 8 can be picked up
/// (+0x00 bit 0x10000): he stands (state 0). His places and states every
/// frame are the original's.
#[test]
#[ignore]
fn level_6_box_into_a_box_as_the_original() {
    use rugrats::input::{CIRCLE, UP};
    let Some(mut b) = Both::open(Boot { level: 6, character: Some(2), ..Default::default() }) else { return };
    b.put([25377, -1378, -35522], 0, 6);
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4), (bits(&[CIRCLE]), 1), (0, 40)]);
    assert!(b.game.carried().is_some() && b.hle.read_u32(rugrats_hle::play::PLAYER + 0x1f0) != 0, "box 9 in hand");
    b.put([42741, -1378, 11400], 0, 4);
    b.script(&[(bits(&[rugrats::input::R1]), 1), (0, 4)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 40), (0, 20)]);
    let worst = report("box", &port, &orig);
    let turned = port.iter().zip(&orig).map(|(p, o)| ((p.heading - o.heading + 2048).rem_euclid(4096) - 2048).abs()).max().unwrap_or(0);
    assert_eq!(worst, 0, "the body's places");
    assert!(port.iter().zip(&orig).all(|(p, o)| p.state == o.state), "the states");
    assert!(port.iter().any(|p| p.state == 9), "the bump");
    assert!(turned <= 1, "the heading, {turned} off");
}

/// Level 9 past cine 17: Tommy put 600, 1000, 1400 and 2200 short of Mr.
/// Friend facing him, and 600 short facing away (his places the
/// original's), Triangle once. ACTION's touched object (0x800631d8: in
/// the nearby list, a character by its size 8192, 0x8006abbc; within the
/// body's depth and 1800, and a quarter turn) is him or nothing: the kick
/// (state 12) on the original's frame, or none on both.
#[test]
#[ignore]
fn level_9_kick_as_the_original() {
    use rugrats::input::TRIANGLE;
    for (d, heading) in [(600, 0), (1000, 0), (1400, 0), (2200, 0), (600, 2048)] {
        let Some(mut b) = Both::open(Boot { level: 9, cinematics: true, ..Default::default() }) else { return };
        for _ in 0..900 {
            b.hle.frame(0).unwrap();
        }
        let (k, _) = sync_npc(&mut b, 46);
        let at = b.game.npcs[k].pos;
        let floor = b.game.player.pos[1];
        b.put([at[0], floor, at[2] - d], heading, 2);
        let (k, o) = sync_npc(&mut b, 46);
        let (mut mine, mut theirs) = (None, None);
        let mut f = 0;
        b.script_with(&[(bits(&[TRIANGLE]), 1), (0, 12)], |g, h| {
            f += 1;
            if mine.is_none() && matches!(g.player.action, Some(37 | 50)) {
                mine = Some(f);
            }
            if theirs.is_none() && h.read_u8(rugrats_hle::play::PLAYER + 0x18c) == 12 {
                theirs = Some(f);
            }
        });
        eprintln!("d {d} heading {heading}: kick port {mine:?} orig {theirs:?}, him at {:?} / {:?}", b.game.npcs[k].pos, [0, 4, 8].map(|i| b.hle.read_i32(o + 0x18 + i)));
        assert_eq!(mine, theirs, "the kick at {d}, heading {heading}");
    }
}

/// Level 1 from its start: Left held, then Cross mid-turn and Left held
/// through the air, let go as he lands. The jump's state turns no more
/// (its first frame has the turn, then the heading holds through the
/// crouch and the air): every frame the original's.
#[test]
#[ignore]
fn hub_jump_mid_turn_as_the_original() {
    use rugrats::input::{CROSS, LEFT};
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    let (port, orig) = b.script(&[(0, 3), (bits(&[LEFT]), 3), (bits(&[LEFT, CROSS]), 1), (bits(&[LEFT]), 12), (0, 20)]);
    let worst = report("turn", &port, &orig);
    assert!(port.iter().zip(&orig).all(|(p, o)| p.heading == o.heading), "the headings");
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 5 as Angelica (character 3), put 1200 short of the first TV
/// robot (one the original holds still, +0x699 240), the game's rand and
/// the ambience's countdown given the original's, left standing 2000
/// frames. The ambience (0x80080ba8, last in the frame) says the
/// original's lines on the original's frames: the floating and the other
/// lines (no attack begun).
#[test]
#[ignore]
fn level_5_ambience_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 5, character: Some(3), ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    let k = b.game.npcs.iter().position(|n| n.character == 35 && !n.hidden).expect("a robot");
    let id = b.game.npcs[k].id;
    let (k, o) = sync_npc(&mut b, id);
    let at = b.game.npcs[k].pos;
    let floor = b.hle.read_i32(o + 0x148);
    b.put([at[0], floor + b.game.player.feet(), at[2] - 1200], 0, 2);
    sync_npc(&mut b, id);
    b.game.rand.state = b.hle.read_u32(0x800c_3f94);
    b.game.ambience = b.hle.read_i32(0x800c_4cc0);
    let said: Rc<std::cell::RefCell<Vec<u16>>> = Rc::default();
    {
        use rrt::emu::Cpu;
        let s = said.clone();
        b.hle.hle.m.hook(0x8008_da4c, move |c| {
            s.borrow_mut().push(c.cpu.reg(4) as u16);
            rrt::emu::HookResult::Continue
        });
    }
    let (mut mine, mut theirs) = (Vec::new(), Vec::new());
    let mut f = 0;
    let heard = said.clone();
    b.script_with(&[(0, 2000)], |g, _| {
        f += 1;
        mine.extend(g.speech.iter().map(|&n| (f, n)));
        theirs.extend(heard.borrow_mut().drain(..).map(|n| (f, n)));
    });
    eprintln!("lines: port {mine:?}\n       orig {theirs:?}");
    assert!(theirs.len() >= 5 && mine == theirs, "the lines and their frames");
}

/// The trigger list (0x800c4c24) the port builds at a level's load
/// (`Game::trigger_list_placements`) against the original's, by template,
/// in order.
#[test]
#[ignore]
fn trigger_lists_as_the_original() {
    let levels: Vec<u8> = std::env::var("TRIG_LEVELS").map(|v| v.split(',').map(|x| x.parse().unwrap()).collect()).unwrap_or(vec![5, 1, 12, 16, 3, 6, 2, 4, 7, 9, 10, 11, 13, 14, 15, 17, 18, 20, 21]);
    let mut off = Vec::new();
    for level in levels {
        let Some(mut b) = Both::open(Boot { level, character: (level == 5).then_some(3), ..Default::default() }) else { return };
        let mut theirs = Vec::new();
        let mut node = b.hle.read_u32(0x800c_4c24);
        while node != 0 && theirs.len() < 400 {
            let ob = b.hle.read_u32(node + 8);
            theirs.push(b.hle.read_i16(ob + 0x96) as u32);
            if std::env::var_os("TRIGDBG").is_some() {
                eprintln!("    {:#x} template {} 48 {:#x} a0 {} a6 {} id {} at {:?}", ob, b.hle.read_i16(ob + 0x96), b.hle.read_u32(ob + 0x48), b.hle.read_i16(ob + 0xa0) as u16, b.hle.read_u8(ob + 0xa6), b.hle.read_i16(ob + 0x38), [0, 4, 8].map(|i| b.hle.read_i32(ob + 0x18 + i)));
            }
            node = b.hle.read_u32(node + 4);
        }
        let mine = b.game.trigger_templates();
        eprintln!("level {level}: port {} orig {}\n  port {mine:?}\n  orig {theirs:?}", mine.len(), theirs.len());
        if mine != theirs {
            off.push(level);
        }
    }
    assert!(off.is_empty(), "trigger lists not the original's in levels {off:?}");
}

/// Level 5: Angelica put 1200 short of robot 13002, the port's trigger
/// pass given the original's place in the list (the entry after
/// 0x800c4c20's). Five entries a frame (0x80072d38), late in the frame,
/// the robot is woken (scheduled) on the original's frame, and turns to
/// her, 672 a frame, playing the turn in place (state 63). Then it bites
/// twice (modes 5, 1, 0 and 11 again, state 49): for 45 frames its
/// heading, its hover's height, its four tracks and her health are the
/// original's every frame.
#[test]
#[ignore]
fn level_5_robot_wakes_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 5, character: Some(3), ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    let (k, o) = sync_npc(&mut b, 2000);
    let at = b.game.npcs[k].pos;
    let floor = b.hle.read_i32(o + 0x148);
    b.put([at[0], floor + b.game.player.feet(), at[2] - 1200], 0, 2);
    let (k, o) = sync_npc(&mut b, 2000);
    // The list's nodes, front to back, and the one the pass stopped at.
    let mut nodes = Vec::new();
    let mut node = b.hle.read_u32(0x800c_4c24);
    while node != 0 && nodes.len() < 400 {
        nodes.push(node);
        node = b.hle.read_u32(node + 4);
    }
    let cursor = b.hle.read_u32(0x800c_4c20);
    assert_eq!(nodes.len(), b.game.triggers.len(), "the list's length");
    b.game.trigger_next = nodes.iter().position(|&n| n == cursor).map_or(0, |i| i + 1);
    b.game.npcs[k].scheduled = b.hle.read_u32(o + 0x9c) != 0;
    let (mut mine, mut theirs) = (None, None);
    let (h0, o0) = (b.game.npcs[k].heading, b.hle.read_i16(o + 0x24) as i32 & 4095);
    // Each frame: its heading, its height (place and lift, +0x1c), its
    // four tracks, and the player's health.
    let mut frames = Vec::new();
    let mut f = 0;
    b.script_with(&[(0, 45)], |g, h| {
        f += 1;
        if mine.is_none() && g.npcs[k].heading != h0 {
            mine = Some(f);
        }
        if theirs.is_none() && h.read_i16(o + 0x24) as i32 & 4095 != o0 {
            theirs = Some(f);
        }
        let n = &g.npcs[k];
        let t: [[i16; 4]; 4] = std::array::from_fn(|j| std::array::from_fn(|i| h.read_i16(o + 0xa8 + 12 * j as u32 + 2 * i as u32)));
        let m = n.animator.record().map(|r| [r[0], r[1], r[2], r[3]]);
        let phases = n.animator.record().map(|r| r[5] & 0xff) == std::array::from_fn(|j| h.read_u8(o + 0xa8 + 12 * j as u32 + 10) as i16);
        frames.push((
            f,
            (n.heading, n.pos[1] + n.root.y, m, phases, g.player.health),
            (h.read_i16(o + 0x24) as i32 & 4095, h.read_i32(o + 0x1c), t, true, h.read_i16(rugrats_hle::play::PLAYER + 0x1a4) as i32),
        ));
    });
    eprintln!("robot turns: port {mine:?} orig {theirs:?}");
    assert!(theirs.is_some() && mine == theirs, "woken on the original's frame");
    let off: Vec<_> = frames.iter().filter(|(_, m, t)| m != t).collect();
    assert!(off.is_empty(), "frames off the original: {off:?}");
}

/// Level 10: the babies hide at the original's spots. EpisodeStart
/// (0x8007ee54) places Phil, Lil and Tommy in one call (0x8004fe14 for
/// group 0, three objects), a rand each. The original's rand (0x800c3f94)
/// is read as it begins; the port's start is run again from that rand,
/// its hiding spots free, and each baby is where the original's is.
#[test]
fn level_10_babies_hide_as_the_original() {
    use rrt::emu::{BusExt, Cpu};
    use rugrats::levels::chuckies_glasses::ChuckiesGlasses;
    let Some(cue) = cue() else { return };
    let Ok(disc) = GameDisc::open(&cue) else { return };
    let picks: Rc<std::cell::RefCell<Vec<(u32, u16)>>> = Rc::default();
    let p2 = picks.clone();
    let mut p = Play::boot_with(
        Rc::new(rrt::disc::Image::open(&cue).unwrap()),
        Boot { level: 10, character: Some(2), ..Default::default() },
        move |h| {
            h.m.hook(0x8004_fe14, move |c| {
                if c.cpu.reg(5) == 0 {
                    let list = c.cpu.reg(4) as u32;
                    let rand = c.bus.read_u32(0x800c_3f94).unwrap_or(0);
                    for i in 0..c.cpu.reg(6) as u32 {
                        let o = c.bus.read_u32(list + 4 * i).unwrap_or(0);
                        p2.borrow_mut().push((rand, c.bus.read_u16(o + 0x38).unwrap_or(0)));
                    }
                }
                rrt::emu::HookResult::Continue
            });
        },
    )
    .unwrap();
    let picks = picks.borrow().clone();
    let ids: Vec<u16> = picks.iter().map(|p| p.1).collect();
    assert_eq!(ids, [608, 609, 1492], "the order the babies hide in");
    let theirs: Vec<[i32; 2]> = ids
        .iter()
        .map(|&id| {
            let o = objects(&mut p)
                .into_iter()
                .find(|&o| o != 0x800e_2b48 && p.read_u32(o + 0x154) != 0 && p.read_i16(o + 0x38) as u16 == id)
                .expect("the baby");
            [p.read_i32(o + 0x18), p.read_i32(o + 0x20)]
        })
        .collect();
    let mut g = Game::open_from(&disc, Level(10), None, &Progress::default()).unwrap();
    // (Group 0, the hiding spots: up to the first of threshold 100.)
    let end = g.spots.iter().position(|s| s.threshold >= 100).map_or(g.spots.len(), |i| i + 1);
    for s in &mut g.spots[..end] {
        s.used = false;
    }
    g.rand.state = picks[0].0;
    let _ = ChuckiesGlasses::start(&mut g, &disc).unwrap();
    let mine: Vec<[i32; 2]> = ids
        .iter()
        .map(|&id| {
            let n = g.npcs.iter().find(|n| n.id == id).expect("the baby");
            [n.pos[0], n.pos[2]]
        })
        .collect();
    eprintln!("hiding: port {mine:?} orig {theirs:?}");
    assert_eq!(mine, theirs);
}
