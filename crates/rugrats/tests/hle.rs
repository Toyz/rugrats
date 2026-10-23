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
        // And the floor under it as the settling has it (0x800452f8:
        // 0x8002b240 into +0x13c and +0x140, then +0x148), which a put
        // would leave as it was: the next state's first frame takes its gap
        // to that floor (0x80055904).
        let q = rugrats::world::collide::query(&self.game.col, pos);
        if q.tile.is_some() {
            let pl = rugrats_hle::play::PLAYER;
            self.hle.write_u32(pl + 0x13c, q.floor as u32);
            self.hle.write_u32(pl + 0x140, q.ceiling as u32);
            self.hle.write_u32(pl + 0x148, q.floor as u32);
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

/// The id now (+0x38, as the object set-up gives it: level 5's TV robots
/// are numbered from 13000) of the character placed with LP id `lp`.
fn set_up_id(b: &Both, level: usize, lp: u16) -> u16 {
    let disc = GameDisc::open(&cue().unwrap()).unwrap();
    let file = disc.file(rugrats::media::cine::LP + level).unwrap();
    let placed = rugrats_data::placement::parse_lp(&file);
    let k = placed.iter().position(|p| p.kind == 1 && p.id == lp).expect("the placement");
    b.game.npcs.iter().find(|n| n.placement == Some(k)).expect("its character").id
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

/// Level 5 as Angelica (character 3) where she starts, no robot within
/// reach, the game's rand and the ambience's countdown given the
/// original's, left standing 2000 frames. The ambience (0x80080ba8, last in the frame) says the
/// original's lines on the original's frames: the floating and the other
/// lines (no attack begun).
#[test]
#[ignore]
fn level_5_ambience_as_the_original() {
    let Some(mut b) = Both::open(Boot { level: 5, character: Some(3), ..Default::default() }) else { return };
    // (A console fast enough for no lag frame; where she starts, no robot
    // within reach of her; both played on from the start, the speech lines
    // near her said on both.)
    b.hle.hle.frame_steps = 1_200_000;
    b.script(&[(0, 30)]);
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
    let robot = set_up_id(&b, 5, 2000);
    let (k, o) = sync_npc(&mut b, robot);
    let at = b.game.npcs[k].pos;
    let floor = b.hle.read_i32(o + 0x148);
    b.put([at[0], floor + b.game.player.feet(), at[2] - 1200], 0, 2);
    let (k, o) = sync_npc(&mut b, robot);
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

/// Level 5: Angelica put 4000 short of a sliding door (template 843), the
/// port's trigger pass given the original's place in its list. The pass
/// enters the door (0x8004d078: open, +0x00 bit 0) on the original's frame,
/// five entries a frame from the cursor (the original's read a frame
/// later: its pass runs after the oracle's frame boundary).
#[test]
#[ignore]
fn level_5_door_opens_as_the_original() {
    use rugrats::world::game::Trigger;
    let Some(mut b) = Both::open(Boot { level: 5, character: Some(3), ..Default::default() }) else { return };
    for _ in 0..30 {
        b.hle.frame(0).unwrap();
    }
    // A door on the list with open floor 4000 short of it, and its object.
    let short = |at: [i32; 3]| [at[0], at[1], at[2] - 4000];
    let (k, spot) = b
        .game
        .triggers
        .iter()
        .filter_map(|e| match e.who {
            Trigger::Placement(k) if b.game.set.placements[k].template == 843 => Some(k),
            _ => None,
        })
        .find_map(|k| {
            let q = rugrats::world::collide::query(&b.game.col, short(b.game.set.placements[k].pos));
            (q.kind == 0 && q.tile.is_some()).then(|| (k, q.floor))
        })
        .expect("a door on the list with floor before it");
    let at = b.game.set.placements[k].pos;
    // The list's nodes (+8 its object), the door's the one at its place.
    let list = |h: &mut Play| {
        let mut nodes = Vec::new();
        let mut node = h.read_u32(0x800c_4c24);
        while node != 0 && nodes.len() < 400 {
            nodes.push(node);
            node = h.read_u32(node + 4);
        }
        nodes
    };
    let objs: Vec<u32> = list(&mut b.hle).into_iter().map(|n| b.hle.read_u32(n + 8)).collect();
    let o = objs.into_iter().find(|&o| (0..3).all(|i| b.hle.read_i32(o + 0x18 + 4 * i as u32) == at[i])).expect("the door's object");
    b.put([at[0], spot + b.game.player.feet(), at[2] - 4000], 0, 2);
    let nodes = list(&mut b.hle);
    assert_eq!(nodes.len(), b.game.triggers.len(), "the list's length");
    let cursor = b.hle.read_u32(0x800c_4c20);
    b.game.trigger_next = nodes.iter().position(|&n| n == cursor).map_or(0, |i| i + 1);
    // (The oracle's frame ends between the objects' update and the pass,
    // 0x80072bf0 | 0x80072d38: the original's pass of a frame is read a
    // frame later.)
    let (mut mine, mut theirs) = (None, None);
    let mut f = 0;
    b.script_with(&[(0, 20)], |g, h| {
        f += 1;
        if mine.is_none() && g.trigger_active(Trigger::Placement(k)) == Some(true) {
            mine = Some(f);
        }
        if theirs.is_none() && h.read_u32(o) & 1 != 0 {
            theirs = Some(f);
        }
    });
    eprintln!("door in: port {mine:?} orig {theirs:?}");
    assert!(theirs.is_some() && mine.map(|f| f + 1) == theirs, "the door entered on the original's frame");
}

/// Level 3 as Tommy, put on a shelf top facing 0 with the camera along
/// 2048, Up held: the stick asks a half turn exactly (2048), which the turn
/// makes up through 1024 (0x8005bc80 keeps +2048's sign), playing the turn
/// in place to the right (logical 52, state 64: 0x8005d11c with
/// 0x80059204) and its root's bob. (From the run's first frame on the
/// follow camera parts from the original's.)
#[test]
#[ignore]
fn level_3_turn_in_place_as_the_original() {
    use rugrats::input::{R1, UP};
    let Some(mut b) = Both::open(Boot { level: 3, ..Default::default() }) else { return };
    let feet = b.game.player.feet();
    b.put([21000, feet, 2000], 0, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let (port, orig) = b.script(&[(bits(&[UP]), 8)]);
    let worst = report("turn", &port, &orig);
    assert_eq!(camera_off(&port, &orig), 0, "the original's camera heading");
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    let turned = port.iter().zip(&orig).filter(|(p, o)| p.heading != o.heading).count();
    assert_eq!((worst, turned), (0, 0), "the same as the original's, every frame");
}

/// Level 1 as Tommy, put in the hall, run along it then turned:
/// the shadow's quad (0x8003414c at 0x80034518, before its draw) as the
/// original builds it each frame: the facing from the first part, the
/// six slots' extents with the least reach, and the floor under it.
#[test]
#[ignore]
fn level_1_shadow_as_the_original() {
    use rrt::emu::{BusExt, Cpu};
    use rugrats::input::{LEFT, R1, UP};
    use std::cell::RefCell;
    let Some(mut b) = Both::open(Boot { level: 1, ..Default::default() }) else { return };
    b.put([11000, 779, 19403], 1024, 6);
    b.script(&[(bits(&[R1]), 1), (0, 4)]);
    let hier = b.hle.read_u32(rugrats_hle::play::PLAYER + 0x154);
    let theirs: Rc<RefCell<Option<rugrats::world::game::Quad>>> = Rc::default();
    let t = theirs.clone();
    b.hle.hle.m.hook(0x8003_4518, move |c| {
        if c.cpu.reg(20) as u32 == hier {
            let sp = c.cpu.reg(29) as u32;
            let mut h = |a: u32| c.bus.read_u16(sp + a).unwrap_or(0) as i16 as i32;
            let lo = [h(0x50), h(0x52), h(0x54)];
            let hi = [h(0x58), h(0x5a), h(0x5c)];
            let facing = [h(0x20), h(0x28)];
            let mut w = |a: u32| c.bus.read_u32(sp + a).unwrap_or(0) as i32;
            let at = [w(0x44), w(0x48), w(0x4c)];
            *t.borrow_mut() = Some(rugrats::world::game::Quad { facing, lo, hi, at });
        }
        rrt::emu::HookResult::Continue
    });
    // (The original draws a frame before it updates: its quad is the
    // port's of the frame before.)
    let mut off = Vec::new();
    let mut frame = 0;
    let mut before = b.game.player_shadow();
    for (bits, n) in [(bits(&[UP]), 12), (bits(&[UP, LEFT]), 8), (0, 6)] {
        for _ in 0..n {
            theirs.borrow_mut().take();
            let (port, orig) = b.script(&[(bits, 1)]);
            let orig_quad = *theirs.borrow();
            if before != orig_quad {
                eprintln!("{frame}: port {before:?} orig {orig_quad:?} | at {:?} {:?}", port[0].pos, orig[0].pos);
                off.push(frame);
            }
            before = b.game.player_shadow();
            frame += 1;
        }
    }
    assert!(off.is_empty(), "the original's shadow quad, off at {off:?}");
}

/// Level 1's start cine (7) with its cinematics played: every character's
/// shadow quad (0x8003414c at 0x80034518, by the object's id +0x38 and the
/// cine frame 0x800c4c34) as the port's cine player builds it from the
/// same stream frame: posed from the keys, on the floor under where the
/// stream puts it.
#[test]
#[ignore]
fn level_1_cine_shadows_as_the_original() {
    use rrt::emu::{BusExt, Cpu};
    use rugrats::media::cine::{Set, Shadows, Stream};
    use rugrats::world::game::Quad;
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    let Some(cue) = cue() else { return };
    let disc = GameDisc::open(&cue).unwrap();
    type Seen = BTreeMap<(u32, u16, i16), Quad>;
    let theirs: Rc<RefCell<Seen>> = Rc::default();
    let cines: Rc<RefCell<Vec<u32>>> = Rc::default();
    let drawing: Rc<RefCell<i16>> = Rc::default();
    let (t, d1, d2, c1, c2) = (theirs.clone(), drawing.clone(), drawing.clone(), cines.clone(), cines.clone());
    Play::boot_with(Rc::new(rrt::disc::Image::open(&cue).unwrap()), Boot { level: 1, cinematics: true, ..Default::default() }, move |hle| {
        hle.m.hook(0x8007_424c, move |c| {
            c1.borrow_mut().push(c.cpu.reg(4) as u32);
            rrt::emu::HookResult::Continue
        });
        hle.m.hook(0x8003_414c, move |c| {
            *d1.borrow_mut() = c.bus.read_u16(c.cpu.reg(22) as u32 + 0x38).unwrap_or(0) as i16;
            rrt::emu::HookResult::Continue
        });
        hle.m.hook(0x8003_4518, move |c| {
            let frame = c.bus.read_u16(0x800c_4c34).unwrap_or(0);
            if frame != 0 {
                let sp = c.cpu.reg(29) as u32;
                let at = [0x44u32, 0x48, 0x4c].map(|a| c.bus.read_u32(sp + a).unwrap_or(0) as i32);
                let mut h = |a: u32| c.bus.read_u16(sp + a).unwrap_or(0) as i16 as i32;
                let q = Quad { facing: [h(0x20), h(0x28)], lo: [h(0x50), h(0x52), h(0x54)], hi: [h(0x58), h(0x5a), h(0x5c)], at };
                let n = c2.borrow().last().copied().unwrap_or(0);
                t.borrow_mut().insert((n, frame, *d2.borrow()), q);
            }
            rrt::emu::HookResult::Continue
        });
    })
    .unwrap();
    let n = *cines.borrow().last().unwrap();
    assert_eq!(n, 7, "level 1's start cine");
    let (set, shadows, stream) = (Set::load(&disc, 1).unwrap(), Shadows::load(&disc, 1).unwrap(), Stream::read(&disc, 7).unwrap());
    let theirs = theirs.borrow();
    let (mut same, mut off) = (0, Vec::new());
    for (_, f) in &stream.frames {
        for (id, q) in set.shadow_quads(f, &shadows) {
            match theirs.get(&(n, f.number, id)) {
                Some(o) if *o == q => same += 1,
                Some(o) => off.push((f.number, id, q, *o)),
                None => {}
            }
        }
    }
    eprintln!("{same} the same, {} off, first {:?}", off.len(), off.first());
    assert!(same > 1000 && off.is_empty(), "the original's cine shadows");
}

/// Level 7 as Reptar, standing: Circle breathes fire (state 30, animation
/// 51 at the written 2048), the console fast enough for no lag frame: the
/// tracks as the original's every frame through the breath.
#[test]
#[ignore]
fn level_7_fire_as_the_original() {
    use rugrats::input::CIRCLE;
    let Some(mut b) = Both::open(Boot { level: 7, character: Some(9), ..Default::default() }) else { return };
    b.hle.hle.frame_steps = 1_200_000;
    b.put([25279, -703, -1418], 2048, 6);
    let (port, orig) = b.script(&[(bits(&[CIRCLE]), 1), (0, 50)]);
    let worst = report("fire", &port, &orig);
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 7 as Reptar, standing: Triangle chomps (state 29, animation 49 at
/// the written 2048), no lag frame: tracks and height as the original's.
#[test]
#[ignore]
fn level_7_chomp_as_the_original() {
    use rugrats::input::TRIANGLE;
    let Some(mut b) = Both::open(Boot { level: 7, character: Some(9), ..Default::default() }) else { return };
    b.hle.hle.frame_steps = 1_200_000;
    b.put([25279, -703, -1418], 2048, 6);
    let (port, orig) = b.script(&[(bits(&[TRIANGLE]), 1), (0, 100)]);
    let worst = report("chomp", &port, &orig);
    assert!(b.tracks_off.is_empty(), "the original's animation tracks, off at {:?}", b.tracks_off);
    assert_eq!(height_off(&port, &orig), 0, "the original's height, root and all");
    assert_eq!(worst, 0, "the same as the original's, every frame");
}

/// Level 5 as Angelica put 1200 short of the first TV robot (13000), on its
/// floor, the console fast enough for no lag frame: it sinks to her and
/// bites (mode 5) until she cries and starts again at the respawn. Her
/// health each frame and her place as the original's.
#[test]
#[ignore]
fn level_5_robot_bites_as_the_original() {
    // (With its cinematics: cine 36's end sets the respawn, 0x8007b280.)
    let Some(mut b) = Both::open(Boot { level: 5, character: Some(3), cinematics: true }) else { return };
    b.hle.hle.frame_steps = 1_200_000;
    b.script(&[(0, 30)]);
    let (k, o) = sync_npc(&mut b, 13000);
    let at = b.game.npcs[k].pos;
    let floor = b.hle.read_i32(o + 0x148);
    b.put([at[0], floor + b.game.player.feet(), at[2] - 1200], 0, 2);
    let (k, o) = sync_npc(&mut b, 13000);
    b.game.rand.state = b.hle.read_u32(0x800c_3f94);
    // (Whether it runs, +0x9c, and the trigger pass's place in its list.)
    b.game.npcs[k].scheduled = b.hle.read_u32(o + 0x9c) != 0;
    let mut nodes = Vec::new();
    let mut node = b.hle.read_u32(0x800c_4c24);
    while node != 0 && nodes.len() < 400 {
        nodes.push(node);
        node = b.hle.read_u32(node + 4);
    }
    let cursor = b.hle.read_u32(0x800c_4c20);
    b.game.trigger_next = nodes.iter().position(|&n| n == cursor).map_or(0, |i| i + 1);
    let mut off = Vec::new();
    let mut f = 0;
    b.script_with(&[(0, 400)], |g, h| {
        f += 1;
        let theirs = h.read_i16(0x800e_2b48 + 0x1a4) as i32;
        let place = [0, 4, 8].map(|i| h.read_i32(0x800e_2b48 + 0x18 + i));
        // (Its height with its lift, as +0x1c has it.)
        let mine = [g.player.pos[0], g.player.pos[1] + g.player.root_y, g.player.pos[2]];
        if g.player.health != theirs || mine != place {
            if off.len() < 12 {
                eprintln!("{f}: health port {} orig {theirs} place port {mine:?} orig {place:?} crying {}", g.player.health, g.crying());
            }
            off.push(f);
        }
    });
    assert!(off.is_empty(), "her health and place, off at {off:?}");
}

/// A launch (0x8008864c's record) as compared: flags, place, radius,
/// life, direction, speed, sprite (TEXPOS entry; none for the model kinds
/// 2 and 4, which leave it and the size unset), half size, damage, and the
/// sound (+0x64) and mark (+0x46, +0x68) where it meets something.
type Launched = (u32, [i32; 3], i32, i32, [i32; 3], i32, i32, [i16; 2], i16, (u8, u16, u16));

fn model_kind(flags: u32) -> bool {
    matches!(flags & 15, 2 | 4)
}

/// The class 10 objects (0x8004e0f4) near `put` (or the level's start) in
/// `level` for `frames`: given the original's clocks (+0x64, +0x98's
/// +0x00), schedules (+0x9c), trigger cursor, seeds and the shared sound
/// count, every launch is the original's, frame for frame. Returns how
/// many.
fn emitters_as_the_original(level: u8, put: Option<[i32; 3]>, frames: usize) -> (usize, usize) {
    use rrt::emu::{BusExt, Cpu};
    use rugrats::world::game::Trigger;
    // (As the character the level runs as from the hub.)
    let character = match level {
        4 | 11 => Some(6),
        5 => Some(3),
        6 | 10 => Some(2),
        7 => Some(9),
        17 => Some(5),
        _ => None,
    };
    let Some(mut b) = Both::open(Boot { level, character, ..Default::default() }) else { return (usize::MAX, 0) };
    b.hle.hle.frame_steps = 1_200_000;
    match put {
        Some(at) => b.put(at, 0, 4),
        None => {
            b.script(&[(0, 4)]);
        }
    }
    let objs = objects(&mut b.hle);
    let h = &mut b.hle;
    let g = &mut b.game;
    g.vcount = h.read_u32(0x800b_cb60);
    // (The update list's order, 0x800c43b8 from its head, +0x10 the
    // object: the newest scheduled first.)
    let mut listed = Vec::new();
    let mut node = h.read_u32(0x800c_43b8);
    while node != 0 && listed.len() < 1000 {
        listed.push(h.read_u32(node + 0x10));
        node = h.read_u32(node);
    }
    let placements = g.set.placements.clone();
    let mut scheduled = Vec::new();
    for e in &mut g.emitters {
        let place = placements[e.placement];
        let Some(&o) = objs.iter().find(|&&o| h.read_i16(o + 0x96) as u16 as u32 == place.template && [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i)) == place.pos) else { continue };
        e.last = h.read_u32(o + 0x64);
        let rec = h.read_u32(o + 0x98);
        e.gone = h.read_i16(rec) != 0;
        scheduled.push((e.placement, h.read_u32(o + 0x9c) != 0));
        e.scheduled = listed.iter().position(|&l| l == o).map_or(0, |i| (listed.len() - i) as u64);
    }
    for (k, on) in scheduled {
        g.trigger_set(Trigger::Placement(k), on);
    }
    // (Every other placement's entry: in after its enter, +0x00 bit 0.)
    let others: Vec<usize> = g
        .triggers
        .iter()
        .filter_map(|e| match e.who {
            Trigger::Placement(k) if !g.emitters.iter().any(|e| e.placement == k) => Some(k),
            _ => None,
        })
        .collect();
    for k in others {
        let place = placements[k];
        if let Some(&o) = objs.iter().find(|&&o| h.read_i16(o + 0x96) as u16 as u32 == place.template && [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i)) == place.pos) {
            g.trigger_set(Trigger::Placement(k), h.read_u32(o) & 1 != 0);
        }
    }
    // (The spinners: their turn, last think and last draw.)
    let count = h.read_u32(0x800c_3f24);
    let now = g.frames;
    for sp in &mut g.spinners {
        let place = placements[sp.placement];
        let Some(&o) = objs.iter().find(|&&o| h.read_i16(o + 0x96) as u16 as u32 == place.template && [0, 4, 8].map(|i| h.read_i32(o + 0x18 + i)) == place.pos) else { continue };
        let base = sp.angles();
        for i in 0..3 {
            sp.angle[i] = (sp.angle[i] + h.read_i16(o + 0x24 + 2 * i as u32) as i32 - base[i] as i32).rem_euclid(4096);
        }
        sp.last = h.read_u32(o + 0x64);
        let ago = count.wrapping_sub(h.read_u32(o + 0x60));
        sp.drawn = (ago < 100).then(|| now.wrapping_sub(ago));
    }
    // (The characters' schedules, +0x9c, their objects by id, +0x38.)
    for n in &mut g.npcs {
        if let Some(&o) = objs.iter().find(|&&o| h.read_u32(o + 0x154) != 0 && h.read_i16(o + 0x38) as u16 == n.id && n.id != 0) {
            n.scheduled = h.read_u32(o + 0x9c) != 0;
            // (Its height, the lift in it: the port's lift the rest.)
            n.root.y = h.read_i32(o + 0x1c) - n.pos[1];
        }
    }
    // (The follow camera, the ambience's count.)
    let m = h.read_u32(rugrats_hle::play::PLAYER + 0x224);
    let r3 = |h: &mut Play, a: u32| [0, 4, 8].map(|k| h.read_i32(a + k));
    let state = (r3(h, 0x800d_e530), r3(h, m + 0x98), r3(h, 0x800d_e454), h.read_u32(0x800c_4a3c), h.read_i32(0x800c_4a58), h.read_i32(0x800c_4a5c), h.read_i32(0x800c_4a38));
    g.sync_follow(state);
    g.ambience = h.read_i32(0x800c_4cc0);
    let cursor = h.read_u32(0x800c_4c20);
    let mut node = h.read_u32(0x800c_4c24);
    let mut i = 0;
    while node != 0 && node != cursor && i < 400 {
        node = h.read_u32(node + 4);
        i += 1;
    }
    g.trigger_next = if node == cursor && node != 0 { i + 1 } else { 0 };
    // (When the original's thinks read the count: an object's last
    // read, +0x64, odd against the frame's even count is one early.)
    let thinker = |t: i16| g.set.templates.get(t as u16 as usize).is_some_and(|t| matches!(t.raw[4], 2 | 10));
    let lag = objs.iter().any(|&o| thinker(h.read_i16(o + 0x96)) && h.read_u32(o + 0x9c) != 0 && h.read_u32(o + 0x64) & 1 != 0) as u32;
    g.vsync_lag = if lag == 1 { 1 } else { 2 };
    g.flights.seeds.direction = h.read_u32(0x800c_445c);
    for (k, at) in [0x800c_41a8u32, 0x800c_41ac, 0x800c_41b0, 0x800c_41b4, 0x800c_41bc, 0x800c_41c0, 0x800c_41c4, 0x800c_41c8, 0x800c_41d8, 0x800c_41b8].into_iter().enumerate() {
        g.flights.seeds.emitters[k] = h.read_u32(at);
    }
    g.shelf_sound = h.read_i32(0x800c_4ba0);
    g.rand.state = h.read_u32(0x800c_3f94);
    let flown: Rc<std::cell::RefCell<Vec<Launched>>> = Rc::default();
    {
        let k = flown.clone();
        b.hle.hle.m.hook(0x8008_864c, move |c| {
            let r = c.cpu.reg(4) as u32;
            let mut rd = |a: u32| c.bus.read_u32(r + a).unwrap_or(0) as i32;
            let flags = rd(0) as u32;
            let (at, radius, life) = ([rd(4), rd(8), rd(0xc)], rd(0x10), rd(0x28));
            let (dir, speed) = ([rd(0x2c), rd(0x30), rd(0x34)], rd(0x38));
            let (sprite_at, size_w, damage) = (rd(0x58) as u32, rd(0x5c), (rd(0x68) >> 16) as i16);
            let met = (rd(0x64) as u8, (rd(0x44) >> 16) as u16, rd(0x68) as u16);
            let texpos = c.bus.read_u32(0x800c_4484).unwrap_or(0);
            let model = model_kind(flags);
            let sprite = if sprite_at >= texpos && texpos != 0 && !model { ((sprite_at - texpos) / 12) as i32 } else { -1 };
            let size = if model { [0; 2] } else { [size_w as i16, (size_w >> 16) as i16] };
            k.borrow_mut().push((flags, at, radius, life, dir, speed, sprite, size, damage, met));
            rrt::emu::HookResult::Continue
        });
    }
    // (The original's thinks run past the oracle's frame boundary, so
    // what they launch shows there a frame after: the port's owed one.)
    let (mut same, mut off) = (0, Vec::new());
    let mut owed: Vec<Launched> = Vec::new();
    let mut f = 0;
    b.script_with(&[(0, frames)], |g, _| {
        f += 1;
        let mut theirs = std::mem::take(&mut *flown.borrow_mut());
        let mine = g.flights.launched.iter().map(|l| {
            let model = model_kind(l.flags);
            let met = (l.sound, l.mark.map_or(0, |m| m.sprite), l.mark.map_or(0, |m| m.size));
            (l.flags, l.at, l.radius, l.life, l.dir, l.speed, l.sprite.filter(|_| !model).map_or(-1, |s| s as i32), if model { [0; 2] } else { l.size }, l.damage, met)
        });
        let was = std::mem::take(&mut owed);
        for (late, l) in was.into_iter().map(|l| (true, l)).chain(mine.map(|l| (false, l))) {
            match theirs.iter().position(|t| *t == l) {
                Some(i) => {
                    theirs.remove(i);
                    same += 1;
                }
                None if late => off.push((f, "port", l)),
                None => owed.push(l),
            }
        }
        off.extend(theirs.into_iter().map(|t| (f, "orig", t)));
    });
    off.extend(owed.into_iter().map(|l| (f, "port", l)));
    if !off.is_empty() {
        eprintln!("level {level}: launches off: {off:?}");
    }
    (same, off.len())
}

/// Level 29's bubbles (616, callback 0x80053344, with the shared count's
/// sounds), level 1's spinning models (614, 0x80053090), level 3's drops
/// (611, 0x80052ed8: their sound and mark), level 6's bursts (969,
/// 0x80053ef8: six sprites, a random cone, a delay) and toys (905,
/// 0x80054238, along the object's matrix); the spinners' callback 1
/// (0x80053604, drawn in the last six frames) in levels 3 and 16 with
/// level 16's sparkles; level 5's robots' glow (0x80053b88, each think);
/// level 4's drops (0x800538bc) and their splash where they meet the
/// water (flight hit callback 2, 0x800529cc), as the original's. (Level
/// 5's robots' glow, 0x80053b88 each think, wants the characters' bodies
/// synced as npc_check does them: there all 739 launches are the
/// original's.)
#[test]
#[ignore]
fn emitters_launch_as_the_original() {
    if cue().is_none() {
        return;
    }
    let mut bad = 0;
    for (level, put, frames, least) in [
        (29, None, 150, 10),
        (1, Some([25986, -1690, 6756]), 120, 10),
        (3, Some([103494, -1509, -18403]), 200, 20),
        (6, Some([-29351, -2321, -15276]), 120, 6),
        (6, Some([-51820, -2678, -4743]), 120, 3),
        (3, Some([67070, 701, 6577]), 120, 50),
        (16, Some([13631, -1166, -30143]), 120, 100),
        (4, Some([-59819, -2394, 8746]), 150, 10),
    ] {
        // (EMITONLY=level: that level's places alone.)
        if std::env::var("EMITONLY").ok().and_then(|v| v.parse::<u8>().ok()).is_some_and(|l| l != level) {
            continue;
        }
        let (n, off) = emitters_as_the_original(level, put, frames);
        eprintln!("level {level} at {put:?}: {n} launches the same, {off} off");
        bad += (off != 0 || n < least) as usize;
    }
    assert_eq!(bad, 0, "places with launches off");
}

/// Which of two things rests on the other (0x8006c7a0, a thrown thing's
/// flight done against what it last met): thing `a` moved about thing
/// `b` on a grid, on the port and on the original (its object's +0x18
/// written, the function called), each answer compared. The count of
/// answers the same and not 0, and of those off.
fn stacking_as_the_original(level: u8, pairs: &[(usize, usize)]) -> (usize, usize) {
    let cue = cue().unwrap();
    let disc = GameDisc::open(&cue).unwrap();
    let mut g = Game::open_from(&disc, Level(level), None, &Progress::default()).unwrap();
    let character = Some(g.character as u32).filter(|&c| c != 1);
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).unwrap()), Boot { level, character, cinematics: false }).unwrap();
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    let mut objects = Vec::new();
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                objects.push(p.read_u32(node + 8));
                node = p.read_u32(node + 4);
            }
        }
    }
    let ipos = |v: rrt::glam::Vec3| [v.x as i32, v.y as i32, v.z as i32];
    let (mut same, mut off) = (0, 0);
    for &(a, b) in pairs {
        let of = |p: &mut Play, k: usize| {
            let at = ipos(g.things[k].pos);
            objects
                .iter()
                .map(|&o| ((0..3).map(|i| (p.read_i32(o + 0x18 + 4 * i as u32) - at[i]).abs() as i64).sum::<i64>(), o))
                .min()
                .map_or(0, |(_, o)| o)
        };
        let (oa, ob) = (of(&mut p, a), of(&mut p, b));
        let (start, home) = (g.things[a].pos, ipos(g.things[b].pos));
        let saved = [0, 4, 8].map(|i| p.read_i32(oa + 0x18 + i));
        for dx in (-600..=600).step_by(150) {
            for dz in (-600..=600).step_by(150) {
                for dy in (-1200..=1200).step_by(40) {
                    let at = [home[0] + dx, home[1] + dy, home[2] + dz];
                    g.things[a].pos = rrt::glam::Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
                    for i in 0..3 {
                        p.write_u32(oa + 0x18 + 4 * i as u32, at[i] as u32);
                    }
                    let theirs = p.hle.call_nested(0x8006_c7a0, &[oa, ob]).unwrap();
                    let mine = g.stacks(a, b) as u32;
                    if mine == theirs {
                        same += (mine != 0) as usize;
                    } else {
                        off += 1;
                        if off <= 10 {
                            eprintln!("level {level} things {a} on {b} at {:?}: port {mine} original {theirs}", [dx, dy, dz]);
                        }
                    }
                }
            }
        }
        g.things[a].pos = start;
        for i in 0..3 {
            p.write_u32(oa + 0x18 + 4 * i as u32, saved[i] as u32);
        }
    }
    (same, off)
}

#[test]
#[ignore]
fn things_stack_as_the_original() {
    if cue().is_none() {
        return;
    }
    let mut bad = 0;
    // Level 3's planks (651) and tipped pillars (650); level 6's boxes.
    for (level, pairs) in [(3u8, vec![(7, 10), (10, 7), (7, 6), (10, 11), (9, 12)]), (6, vec![(0, 1), (8, 9)])] {
        let (same, off) = stacking_as_the_original(level, &pairs);
        eprintln!("level {level}: {same} stackings the same, {off} answers off");
        bad += (off != 0 || same < 20) as usize;
    }
    assert_eq!(bad, 0, "levels with stackings off");
}
