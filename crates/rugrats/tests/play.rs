//! Behaviour the port must keep while it changes: levels played headless
//! with scripted pads, as the play_script example runs them. Skipped
//! without a disc in work/disc.

use std::path::Path;

use rugrats::actors::player::State;
use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::flights;
use rugrats::world::game::Game;

fn disc() -> Option<GameDisc> {
    let cue = rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()?;
    GameDisc::open(&cue).ok()
}

/// Steps of (buttons, frames).
fn run(g: &mut Game, steps: &[(&[usize], usize)], mut each: impl FnMut(&mut Game, usize)) {
    let mut pads = Pads::default();
    let mut frame = 0;
    for &(buttons, n) in steps {
        let bits = buttons.iter().fold(0u16, |a, &b| a | 1 << b);
        for _ in 0..n {
            pads.push_bits(0, bits);
            g.frame(&pads);
            each(g, frame);
            frame += 1;
        }
    }
}

/// Frames with the analog stick at `stick` (x right, y up, 4096 full) and
/// `buttons` down.
fn run_stick(g: &mut Game, stick: [i32; 2], buttons: &[usize], n: usize) {
    let mut pads = Pads::default();
    let bits = buttons.iter().fold(0u16, |a, &b| a | 1 << b);
    for _ in 0..n {
        pads.push_bits(0, bits);
        pads.push_stick(0, stick);
        g.frame(&pads);
    }
}

/// The analog stick that runs toward world heading `h` with the camera
/// where it is.
fn stick_toward(g: &Game, h: i32) -> [i32; 2] {
    let a = (h - g.camera_heading()) as f32 * std::f32::consts::TAU / 4096.0;
    // (A hair over full: the axes are cut to 4096, the length to 4096.)
    [(a.sin() * 4100.0) as i32, (a.cos() * 4100.0) as i32]
}

fn open(disc: &GameDisc, level: u8) -> Game {
    Game::open_from(disc, Level(level), None, &Progress::default()).unwrap()
}

/// A cinematic finished, as the app tells the level once it has played:
/// its last frame leaves the level's objects (`Game::cine_left`), then
/// CineFinish (0x800783d8); kept for `the_whole_game_by_play` to play.
trait Played {
    fn played(&mut self, n: usize);
}

impl Played for Game {
    fn played(&mut self, n: usize) {
        CINES.with(|c| c.borrow_mut().push((self.level, n)));
        if let Some(disc) = disc()
            && let Ok(s) = rugrats::media::cine::Stream::read(&disc, n)
            && let Some((_, last)) = s.frames.last()
        {
            self.cine_left(last);
        }
        self.cine_finished(n);
    }
}

thread_local! {
    /// Every cinematic the tests finished, by level.
    static CINES: std::cell::RefCell<Vec<(Level, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// While set, the player walks (`walk_to`) where a test would put it,
    /// and the places it could not walk to are kept here.
    static WALKING: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
    /// While set, every place a test puts the player is first walked to
    /// from where the player is (`on_foot`), and the misses kept here.
    static ON_FOOT: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}

/// Notes `what` as not reached on foot, when checking.
fn not_on_foot(g: &Game, what: String) {
    let from = g.player.pos;
    ON_FOOT.with(|c| c.borrow_mut().as_mut().map(|v| v.push(format!("level {}: {what} from {from:?}", g.level.0))));
}

fn checking() -> bool {
    ON_FOOT.with(|c| c.borrow().is_some())
}

fn walking() -> bool {
    WALKING.with(|c| c.borrow().is_some())
}

/// Notes a place the walk did not reach.
fn not_walked(g: &Game, what: String) {
    let from = g.player.pos;
    WALKING.with(|c| c.borrow_mut().as_mut().map(|v| v.push(format!("level {}: {what} from {from:?}", g.level.0))));
}

fn put(g: &mut Game, at: [i32; 3], heading: i32) {
    if walking() {
        let feet = at[1] - g.player.feet();
        let land = floor_under(g, at[0], at[2], feet - 1, 0).unwrap_or(feet);
        // Its cell or one within two (a place by a thing is stood beside),
        // as the reachability check takes it.
        let key = (cell(at[0]), cell(at[2]));
        if walk_to(g, |s| (s.0 - key.0).abs() <= 2 && (s.1 - key.1).abs() <= 2 && (s.2 - land).abs() < 300, Some([at[0], at[2]])) {
            // Heading 0 is "any way" to the tests; another is turned to,
            // the camera behind it (the stick's Up runs it).
            if heading != 0 {
                turn_to(g, heading);
                if g.fixed.is_none() {
                    run(g, &[(&[input::R1], 1), (&[], 2)], |_, _| {});
                }
            }
            return;
        }
        not_walked(g, format!("{at:?}"));
    }
    if checking() {
        // The floor the body comes down on there reached (the tests put
        // the player at things in the air too).
        let reach = on_foot(g, g.player.pos);
        let feet = at[1] - g.player.feet();
        let land = floor_under(g, at[0], at[2], feet - 1, 0).unwrap_or(feet);
        // Within two cells of it: a place by a thing is stood beside, not
        // on it.
        let (kx, kz) = (at[0].div_euclid(256), at[2].div_euclid(256));
        let near = (-2..=2).flat_map(|dx| (-2..=2).map(move |dz| (kx + dx, kz + dz)));
        if !near.into_iter().any(|key| reach.get(&key).is_some_and(|fs| fs.iter().any(|&f| (f - land).abs() < 300))) {
            let around: Vec<_> = (-2..=2).flat_map(|dx| (-2..=2).map(move |dz| (kx + dx, kz + dz))).filter_map(|k| reach.get(&k).map(|f| (k, f.clone()))).collect();
            not_on_foot(g, format!("{at:?} (floor {land}; reached about it {around:?}, {} places in all, from {:?} {:?})", reach.len(), Ground::new(g).spot(g.player.pos), g.player.state));
        }
    }
    g.player.pos = at;
    g.player.heading = heading;
    g.player.target = heading;
    g.set_camera_yaw(heading as f32);
}

/// The stick's eight ways (0x8005be24: the direction is turned by the
/// camera's heading).
const STICK: [(i32, &[usize]); 8] = [
    (0, &[input::UP]),
    (512, &[input::UP, input::RIGHT]),
    (1024, &[input::RIGHT]),
    (1536, &[input::DOWN, input::RIGHT]),
    (2048, &[input::DOWN]),
    (2560, &[input::DOWN, input::LEFT]),
    (3072, &[input::LEFT]),
    (3584, &[input::UP, input::LEFT]),
];

/// Turns the player to `h` with the stick, as near as it goes: a stick
/// way's heading is the camera's turned by a multiple of 512, so a snap of
/// 512 round sets the follow camera swinging after the body, and the way
/// whose heading is then within a few of `h` is pressed for a frame: a
/// turn under SNAP_TURN snaps to it (and takes a step), a bigger one turns
/// in place to it. Where the camera does
/// not swing, the nearest way. The error left.
fn turn_to(g: &mut Game, h: i32) -> i32 {
    turn_near(g, h, 16)
}

/// `turn_to`, left as it is when already within `near` of `h`.
fn turn_near(g: &mut Game, h: i32, near: i32) -> i32 {
    let off = |a: i32| (a + 2048).rem_euclid(4096) - 2048;
    if g.player.frozen {
        for _ in 0..30 {
            let keys = steer(g, h);
            if keys.is_empty() {
                break;
            }
            run(g, &[(keys, 1)], |_, _| {});
        }
        run(g, &[(&[], 1)], |_, _| {});
        return off(h - g.player.heading);
    }
    if off(h - g.player.heading).abs() <= near {
        return off(h - g.player.heading);
    }
    // The analog stick pushed a little at `h` for a frame: the aim set,
    // no step (under 241 the speed, 17 x 241 / 4096, is none).
    let [x, z] = stick_toward(g, h);
    run_stick(g, [x * 200 / 4100, z * 200 / 4100], &[], 1);
    // The turn in place runs on to the way's heading (Spike's at 46 a
    // frame takes 45 for half a turn).
    for _ in 0..100 {
        if g.player.heading == g.player.target {
            break;
        }
        run(g, &[(&[], 1)], |_, _| {});
    }
    off(h - g.player.heading)
}

/// The stick this frame to aim a held player (frozen: the stick only
/// turns it, 160 a frame, and letting go stops the turn) at `h`: the way
/// on past `h` on its side while more than 80 off, else none.
fn steer(g: &Game, h: i32) -> &'static [usize] {
    let off = |a: i32| (a + 2048).rem_euclid(4096) - 2048;
    let (c, me) = (g.camera_heading(), g.player.heading);
    let d = off(h - me);
    if d.abs() <= 80 {
        return &[];
    }
    STICK
        .iter()
        .map(|(w, keys)| (off(c + w - me), *keys))
        .filter(|(t, _)| t.signum() == d.signum() && t.abs() >= d.abs())
        .min_by_key(|(t, _)| t.abs())
        .map_or(&[], |(_, keys)| keys)
}

/// Golf's aim (0x8005be24 in state 14): Left and Right turn 25 a frame, 8
/// with Square; as 25 and 8 have no common factor, presses of both reach
/// `to`'s heading exactly.
fn golf_aim(g: &mut Game, to: [i32; 3]) {
    let me = g.player.pos;
    let h = (((to[0] - me[0]) as f32).atan2((to[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU) as i32;
    let d = (h - g.player.heading + 2048).rem_euclid(4096) - 2048;
    // The fewest presses: a of 25 and b of 8 with 25a + 8b = d.
    let (a, b) = (-200..=200)
        .filter_map(|a: i32| {
            let rest = d - 25 * a;
            (rest % 8 == 0).then_some((a, rest / 8))
        })
        .min_by_key(|(a, b)| a.abs() + b.abs())
        .unwrap();
    let way = |n: i32| if n > 0 { input::RIGHT } else { input::LEFT };
    run(g, &[(&[way(a)], a.unsigned_abs() as usize), (&[way(b), input::SQUARE], b.unsigned_abs() as usize)], |_, _| {});
}

/// `steer` toward a place.
fn steer_at(g: &Game, to: [i32; 3]) -> &'static [usize] {
    let me = g.player.pos;
    let h = ((to[0] - me[0]) as f32).atan2((to[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
    steer(g, (h as i32).rem_euclid(4096))
}

/// The buttons that run toward `heading` with the camera where it is.
fn toward(g: &Game, heading: i32) -> &'static [usize] {
    let want = (heading - g.camera_heading()).rem_euclid(4096);
    let off = |d: i32| (d - want + 2048).rem_euclid(4096) - 2048;
    STICK.iter().min_by_key(|(d, _)| off(*d).abs()).unwrap().1
}

/// Level 13's race: Angelica walks her posts to the goal and cine 20.
#[test]
fn level_13_race_reaches_its_goal() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 13);
    let mut cine = None;
    run(&mut g, &[(&[], 3000)], |g, _| cine = cine.or(g.cine.take()));
    assert_eq!(cine, Some(20));
}

/// Every hub door from its spot (or beside it) leads to its level.
#[test]
fn hub_doors_lead_to_their_levels() {
    let Some(disc) = disc() else { return };
    let doors: [([i32; 3], u8); 9] = [
        ([16597, -5365, 14246], 11),
        ([34864, -5365, -5324], 5),
        ([22563, -5365, -6729], 13),
        ([37002, 779, 9866], 2),
        ([26808, 779, -8518], 25),
        ([38751, -5365, 17399], 12),
        ([9330, 779, 20900], 10),
        ([1300, 779, -2713], 18),
        ([22800, 7947, 2900], 9),
    ];
    for (at, level) in doors {
        let mut g = open(&disc, 1);
        let h = g.player.heading;
        put(&mut g, at, h);
        let mut exit = None;
        run(&mut g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 3)], |g, _| exit = exit.or(g.exit.take()));
        assert_eq!(exit.map(|e| e.level), Some(Level(level)), "door at {at:?}");
    }
}

/// The hub's back door slides open and lets Tommy into the yard.
#[test]
fn hub_back_door_opens_to_the_yard() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    put(&mut g, [7155, 779, 22000], 0);
    run(&mut g, &[(&[], 2), (&[input::UP, input::SQUARE], 120)], |_, _| {});
    assert_eq!(g.player.pos[2], 23120, "walking, the shut door stops him");
    put(&mut g, [7155, 779, 22000], 0);
    run(&mut g, &[(&[], 2), (&[input::UP], 40)], |_, _| {});
    // The oracle: run from z 22000, he stops at 23020 (his probe, 250 +
    // the step 170, reaches the panel); walking (Square), at 23120.
    assert_eq!(g.player.pos[2], 23020, "the shut door stops him");
    run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 40), (&[input::UP], 60)], |_, _| {});
    assert!(g.player.pos[2] > 26000, "through the open door: {:?}", g.player.pos);
}

/// The puzzle board goes back to Reptar 2010 once it has begun.
#[test]
fn board_returns_to_level_7() {
    let Some(disc) = disc() else { return };
    let mut p = Progress::default();
    p.set_flags(p.flags() | 512);
    let mut g = Game::open_from(&disc, Level::HUB, None, &p).unwrap();
    put(&mut g, [3103, 779, 1200], 2048);
    let mut exit = None;
    run(&mut g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 3)], |g, _| exit = exit.or(g.exit.take()));
    assert_eq!(exit.map(|e| e.level), Some(Level::REPTAR_2010));
}

/// Golf holes do not hurt (Damage is dropped in 23-31).
#[test]
fn golf_hazards_do_not_hurt() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 25);
    put(&mut g, [31966, -4719, 9241], 0);
    run(&mut g, &[(&[input::UP], 120)], |_, _| {});
    assert_eq!(g.player.health, 255);
}

/// Level 6: five Reptar boxes carried, each put at the foot of a step
/// too high to climb carrying one (a box is 506, the carry's climb at
/// most 521), take Tommy up the Reptar room's steps to the -4608 ledge;
/// the last, against the -6144 block, lets him onto it and down to the
/// switch platform (-4736).
#[test]
fn level_6_boxes_on_the_steps_reach_the_platform() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    assert_eq!(enter_room(&mut g, 11), Some(46));
    g.played(46);
    box_route(&mut g);
    assert!(g.player.pos[2] > 14336 && g.player.pos[1] < -4800, "on the platform: {:?}", g.player.pos);
}

/// The boxes' route (the test above): each box fetched (put beside it,
/// Circle), carried up the steps laid so far with the stick, and put down
/// (Circle) a step's depth short of the next face.
fn box_route(g: &mut Game) {
    let boxes: Vec<usize> = (0..g.things.len()).filter(|&k| g.things[k].template == 1016).collect();
    let up = [
        [44500, 11700],
        [44500, 12900],
        [45500, 12900],
        [46600, 12900],
        [50600, 12900],
        [52000, 12900],
        [52000, 13750],
        [52000, 15000],
        [48700, 15000],
    ];
    // The box, how far up the route it is put down, facing which way:
    // the -2048 step (z 12288), the -3072 block (x 46080), the -3584 block
    // (x 51200), the -4608 ledge (z 14336), the -6144 block (x 48128).
    // The five on the floor or reached by a climb or a running jump (the
    // other three sit on shelves 1536 up: a box under them is the way).
    let laid = [(boxes[6], 1, 0), (boxes[7], 3, 1024), (boxes[3], 5, 1024), (boxes[0], 7, 0), (boxes[4], 9, 3072)];
    for (k, n, h) in laid {
        let b = g.things[k].pos;
        // 900 from it on its own floor, clear of other things (room to turn
        // to it without a step into it).
        let floor = b.y as i32 + 256;
        // (Nearer where a shelf has no room so far off.)
        let room = |x: i32, z: i32| {
            [(0, 0), (300, 0), (-300, 0), (0, 300), (0, -300)]
                .iter()
                .all(|&(dx, dz)| {
                    // (A box may rest a little over the floor, 42 for one;
                    // a shelf's top is a little uneven, 90.)
                    floor_under(g, x + dx, z + dz, floor - 384, 0).is_some_and(|f| (f - floor).abs() < 128) && !Ground::new(g).blocked(x + dx, z + dz, floor)
                })
        };
        let side = [900.0f32, 700.0, 450.0]
            .iter()
            .flat_map(|&r| {
                (0..8).map(move |d| {
                    let a = d as f32 * std::f32::consts::TAU / 8.0;
                    [b.x as i32 + (a.sin() * r) as i32, b.z as i32 + (a.cos() * r) as i32]
                })
            })
            .find(|&[x, z]| room(x, z))
            .unwrap_or([b.x as i32, b.z as i32 - 450]);
        // Circle takes the nearest in reach that is faced. The
        // guard by two of them bites and pushes, and may be stood on: back
        // beside it and again.
        for _ in 0..5 {
            put(g, [side[0], b.y as i32 + 11, side[1]], 0);
            // Up on it (walking into a box climbs it): off it, away.
            for _ in 0..3 {
                let me = g.player.pos;
                if me[1] - g.player.feet() > floor - 300 {
                    break;
                }
                let (dx, dz) = (me[0] as f32 - b.x, me[2] as f32 - b.z);
                let len = dx.hypot(dz).max(1.0);
                stride_to(g, me[0] + (dx * 1200.0 / len) as i32, me[2] + (dz * 1200.0 / len) as i32, 600, 20);
                if std::env::var_os("WALKLOG").is_some() {
                    eprintln!("  OFF box {k} from {me:?} to {:?} {:?} {:?} h{} t{} sp{}", g.player.pos, g.player.state, g.player.action, g.player.heading, g.player.target, g.player.speed);
                    let a = g.player.heading as f32 * std::f32::consts::TAU / 4096.0;
                    let feet = g.player.pos[1] - g.player.feet();
                    for w in [0.0f32, 250.0, -250.0] {
                        let x = me[0] + (a.sin() * 420.0 + a.cos() * w) as i32;
                        let z = me[2] + (a.cos() * 420.0 - a.sin() * w) as i32;
                        let q = rugrats::world::collide::query(&g.col, [x, feet - 64, z]);
                        eprintln!("    probe {w}: ({x},{z}) kind {} floor {} feet {feet}", q.kind, q.floor);
                    }
                    for bx in bodies(g).iter().filter(|bx| bx.covers(me, 700)) {
                        eprintln!("    body t{} {:?} {:?} {:?} y {}..{} char {}", bx.template, bx.pos, bx.lo, bx.hi, bx.min_y, bx.max_y, bx.character);
                    }
                }
                run(g, &[(&[], 10)], |_, _| {});
            }
            // Nearer only if out of reach (1327 in front for Tommy and a
            // box): walking into it climbs it.
            let me = g.player.pos;
            if ((b.x as i64 - me[0] as i64).pow(2) + (b.z as i64 - me[2] as i64).pow(2)) > 1100 * 1100 {
                stride_to(g, b.x as i32, b.z as i32, 1000, 30);
            }
            // Facing it: GRAB takes only what is within a quarter turn of
            // the heading (0x800631d8).
            let me = g.player.pos;
            let h = ((b.x - me[0] as f32).atan2(b.z - me[2] as f32) * 4096.0 / std::f32::consts::TAU) as i32;
            turn_near(g, h.rem_euclid(4096), 256);
            run(g, &[(&[input::CIRCLE], 1), (&[], 40)], |_, _| {});
            if std::env::var_os("WALKLOG").is_some() {
                eprintln!("BOX {k} try from {side:?}: Tommy {:?} heading {} {:?}, carried {:?}", g.player.pos, g.player.heading, g.player.state, g.carried());
            }
            if g.carried() == Some(k) {
                break;
            }
        }
        let p = g.player.pos;
        assert_eq!(g.carried(), Some(k), "box {k} picked up: at {p:?} heading {} {:?} {:?}, the box at {b:?}", g.player.heading, g.player.state, g.player.action);
        put(g, [44500, -1269, 10500], 0);
        run(g, &[(&[], 5)], |_, _| {});
        go(g, &up[..n]);
        // Put down on the floor Tommy stands on, at the next step's foot:
        // where the turn's step took him too close and it came down on the
        // step, up with it again, back a little, and down again.
        let [x, z] = up[n - 1];
        let a = h as f32 * std::f32::consts::TAU / 4096.0;
        // (The turn's own step takes him up to 100 on: a little back first.)
        for back in [150.0, 300.0, 450.0, 600.0] {
            stride_to(g, x - (a.sin() * back) as i32, z - (a.cos() * back) as i32, 48, 40);
            // The box drops ahead from the hands: near the heading does.
            turn_near(g, h, 200);
            let feet = g.player.pos[1] - g.player.feet();
            run(g, &[(&[input::CIRCLE], 1), (&[], 60)], |_, _| {});
            assert_eq!(g.carried(), None, "box {k} put down");
            let t = &g.things[k];
            if std::env::var_os("WALKLOG").is_some() {
                eprintln!("BOX {k} down at {:?}, Tommy {:?} feet {feet} back {back}", t.pos, g.player.pos);
            }
            if (t.pos.y as i32 + 256 - feet).abs() < 100 {
                break;
            }
            let (tp, me) = (g.things[k].pos, g.player.pos);
            stride_to(g, tp.x as i32, tp.z as i32, 400, 20);
            run(g, &[(&[input::CIRCLE], 1), (&[], 40)], |_, _| {});
            assert_eq!(g.carried(), Some(k), "box {k} taken up again: it at {tp:?}, Tommy at {me:?} (feet {feet}), back {back}");
        }
    }
    // Onto the last box, the -6144 block, and down to the platform.
    go(g, &[[47000, 15000], [45000, 15500]]);
}

/// Level 16's pie toss: five hits on Angelica win it. The oracle's run:
/// heading 913, a press every 24 VSyncs (12 frames); a press while a throw
/// is under way is ignored.
#[test]
fn level_16_toss_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 16);
    win_16(&mut g);
}

/// Level 16 won by play (the test above).
fn win_16(g: &mut Game) {
    put(g, [31600, -1750, -8141], 1024);
    // Up onto the spot, then (held there) aimed with the stick at 913 and
    // a press every 12 frames from frame 345.
    // Won, not timed out: the clock goes off with Angelica stopped
    // (mode 0) and her health back to full.
    let mut ended = None;
    let mut hits = 0;
    let mut health = 250;
    let mut pads = Pads::default();
    for f in 0..345 + 80 * 12 {
        let mut keys: Vec<usize> = if f < 15 { vec![input::UP] } else { vec![] };
        if f > 330 {
            keys.extend(steer(g, 913));
        }
        if f >= 345 && (f - 345) % 12 == 0 {
            keys.push(input::TRIANGLE);
        }
        pads.push_bits(0, keys.iter().fold(0u16, |a, &b| a | 1 << b));
        g.frame(&pads);
        if let Some(h) = g.npcs.iter().find(|n| n.id == 600).map(|n| n.health) {
            if h < health {
                hits += 1;
            }
            health = h;
        }
        if ended.is_none() && f > 20 && g.timer.is_none() {
            let a = g
                .npcs
                .iter()
                .find(|n| n.id == 600)
                .map(|n| (matches!(n.mode, rugrats::actors::npc::Mode::Idle), n.health == n.max_health));
            ended = Some((f, a));
        }
    }
    assert!(matches!(ended, Some((_, Some((true, true))))), "toss ended {ended:?} after {hits} hits");
}

/// Every reachable level opens and plays a few seconds of running about.
#[test]
fn every_level_runs() {
    let Some(disc) = disc() else { return };
    for level in [1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 23, 24, 25, 26, 27, 28, 29, 30, 31] {
        let mut g = open(&disc, level);
        run(&mut g, &[(&[], 5), (&[input::UP], 60), (&[input::CROSS], 1), (&[input::LEFT], 40), (&[], 100)], |_, _| {});
    }
}

/// Every level's body has frames for standing, walking, running and
/// jumping once the logical animations are the table's states.
#[test]
fn every_body_has_its_moves() {
    use rugrats::actors::player::anim;
    let Some(disc) = disc() else { return };
    for level in [1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 23, 24, 25, 26, 27, 28, 29, 30, 31] {
        let g = open(&disc, level);
        for a in [anim::IDLE, anim::STAND, anim::RUN, anim::JUMP] {
            let frames = g.player.anm(a).map_or(0, |id| g.anims.frames_of(id).len());
            assert!(frames > 0, "level {level}: animation {a} has no frames");
        }
    }
}

/// Walks onto each pickup `want` names, the nearest left first, as a
/// player would (a jump for one overhead), until the level plays a
/// cinematic.
fn collect(g: &mut Game, want: impl Fn(&rugrats::world::game::Goodie) -> bool) -> Option<usize> {
    // The nearest left each time.
    loop {
        let me = g.player.pos;
        let d = |p: rrt::glam::Vec3| (p.x as i64 - me[0] as i64).pow(2) + (p.z as i64 - me[2] as i64).pow(2) + (p.y as i64 - me[1] as i64).pow(2);
        let Some(k) = (0..g.goodies.len()).filter(|&k| !g.goodies[k].taken && want(&g.goodies[k])).min_by_key(|&k| d(g.goodies[k].pos)) else { break };
        let p = g.goodies[k].pos;
        let (t0, from) = (g.frames, g.player.pos);
        let at = [p.x as i32, p.y as i32 - 100, p.z as i32];
        // Run through it, as a player does: straight at it where the line
        // is clear, else by the way to its place first.
        let feet = at[1] - g.player.feet();
        let land = floor_under(g, at[0], at[2], feet - 1, 0).unwrap_or(feet);
        let key = (cell(at[0]), cell(at[2]));
        let straight = walking() && line_clear(g, at, land);
        let mut cine = None;
        // Straight first where clear; held up on the way (a corner met by
        // the body's side), by the planned way after.
        let tries: &[bool] = if straight { &[false, true] } else { &[true] };
        for &planned in tries {
            if g.goodies[k].taken || cine.is_some() {
                break;
            }
            if planned
                && !(walking() && walk_to(g, |s| (s.0 - key.0).abs() <= 1 && (s.1 - key.1).abs() <= 1 && (s.2 - land).abs() < 300, None))
            {
                put(g, at, 0);
            }
            // At it with the stick until taken. (Put down 100 up, he lands
            // first: the landing plays out, 0x8005cba8 does not move him.)
            let far = ((p.x - g.player.pos[0] as f32).hypot(p.z - g.player.pos[2] as f32) / 100.0) as usize;
            let mut held = 0;
            for _ in 0..40 + far {
                if g.goodies[k].taken || cine.is_some() || held > 12 {
                    break;
                }
                // Over his head: a jump for it (Cross held) once under it.
                let me = g.player.pos;
                let close = ((p.x as i64 - me[0] as i64).pow(2) + (p.z as i64 - me[2] as i64).pow(2)) < 250 * 250;
                if (p.y as i32) < me[1] - 300 && close && matches!(g.player.state, State::Stand | State::Walk | State::Run) {
                    run(g, &[(&[input::CROSS], 20)], |g, _| cine = cine.or(g.cine.take()));
                    continue;
                }
                stride(g, p.x as i32, p.z as i32, false);
                cine = cine.or(g.cine.take());
                let moved = (g.player.pos[0] - me[0]).abs() + (g.player.pos[2] - me[2]).abs();
                held = if moved < 8 && !planned { held + 1 } else { 0 };
            }
        }
        run(g, &[(&[], 2)], |g, _| cine = cine.or(g.cine.take()));
        if std::env::var_os("WALKLOG").is_some() {
            let d = ((p.x - from[0] as f32).hypot(p.z - from[2] as f32)) as i32;
            eprintln!("PICKUP {k}: {} frames for {d} straight ({} a frame)", g.frames - t0, d / (g.frames - t0).max(1) as i32);
        }
        if cine.is_some() {
            return cine;
        }
        if !g.goodies[k].taken {
            let (x, z) = (p.x as i32, p.z as i32);
            let floors: Vec<_> = [-600, -300, 0, 300, 600].iter().map(|d| floor_under(g, x + d, z, p.y as i32 - 600, 0)).collect();
            let q = rugrats::world::collide::query(&g.col, [x, p.y as i32, z]);
            panic!("pickup {k} at {p:?} not taken: Tommy {:?}, floors by x {floors:?}, COL {} {} {}", g.player.pos, q.kind, q.floor, q.ceiling);
        }
    }
    let mut cine = None;
    run(g, &[(&[], 30)], |g, _| cine = cine.or(g.cine.take()));
    cine
}

/// Level 17: all 62 nickels end the clock and win (cine 26).
#[test]
fn level_17_gold_rush_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 17);
    win_17(&mut g);
}

/// Level 17 won by play (the test above).
fn win_17(g: &mut Game) {
    assert_eq!(collect(g, |n| n.template == 446), Some(26));
}

/// Level 18: the 35th egg ends the clock and wins (cine 24).
#[test]
fn level_18_egg_hunt_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 18);
    win_18(&mut g);
}

/// Level 18 won by play (the test above).
fn win_18(g: &mut Game) {
    let t0 = g.frames;
    let cine = collect(g, |e| e.template == 496);
    let left = g.goodies.iter().filter(|e| e.template == 496 && !e.taken).count();
    assert_eq!(cine, Some(24), "{left} eggs left after {} frames", g.frames - t0);
}

/// Level 15: Triangle beside each ? box starts its colour, and its
/// balloons taken fill it; the sixth colour wins (cine 72). Each box
/// breaks into pieces (0x8004cdfc: ten slots at most).
#[test]
fn level_15_mirrorland_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 15);
    win_15(&mut g);
}

/// Level 15 won by play (the test above).
fn win_15(g: &mut Game) {
    let mut won = None;
    for id in 207..=212u16 {
        let b = g.set.placements.iter().find(|p| p.kind == 1 && p.id == id).unwrap().pos;
        // Beside the box, in its room.
        let room = g.room_of(b);
        // (900 off: a walk's last stride at one nearer climbs it.)
        let at = [[900, 0], [-900, 0], [0, 900], [0, -900]]
            .map(|[dx, dz]| [b[0] + dx, b[1], b[2] + dz])
            .into_iter()
            .find(|&p| g.room_of(p) == room)
            .unwrap_or(b);
        put(g, at, 0);
        // Down first (the way may end off a ledge), then nearer if the
        // walk stopped short (its place is had within two cells).
        for _ in 0..90 {
            if !matches!(g.player.state, State::Jump | State::Fall) {
                break;
            }
            run(g, &[(&[], 1)], |_, _| {});
        }
        let me = g.player.pos;
        if ((b[0] - me[0]) as i64).pow(2) + ((b[2] - me[2]) as i64).pow(2) > 1000 * 1000 {
            stride_to(g, b[0], b[2], 950, 40);
        }
        let mut pieces = 0;
        run(g, &[(&[], 4), (&[input::TRIANGLE], 1), (&[], 2)], |g, _| pieces = pieces.max(g.flights.shards().count()));
        assert!(g.is_hidden(id), "box {id} did not open: Tommy at {:?}, the box at {b:?}, put at {at:?}", g.player.pos);
        assert!((1..=10).contains(&pieces), "box {id} broke into {pieces}");
        // Walking for one box can take the last pickups on the way.
        let t0 = g.frames;
        won = won.or(collect(g, |b| (6..=11).contains(&b.kind)));
        if std::env::var_os("WALKLOG").is_some() {
            eprintln!("COLOUR box {id}: {} frames, timer now {:?}", g.frames - t0, g.timer);
        }
    }
    let left: Vec<_> = g.goodies.iter().filter(|b| !b.taken).map(|b| (b.kind, b.pos)).collect();
    assert_eq!(won, Some(72), "left {left:?}, cine {:?}", g.cine);
}

/// Level 9: Mr. Friend kicked down four times; the first brings back all
/// three (cine 18, as the app plays it), the fourth wins (cine 19).
#[test]
fn level_9_mr_friend_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 9);
    win_9(&mut g);
}

/// Level 9 won by play (the test above).
fn win_9(g: &mut Game) {
    // He comes after Tommy: one far off (46 waits at home after cine 18)
    // is gone up to; one near is steered at and kicked (Triangle within
    // 1100 of him).
    let friends = [46u16, 91, 800];
    let nearest = |g: &Game| {
        let me = g.player.pos;
        let d2 = |p: [i32; 3]| ((p[0] - me[0]) as i64).pow(2) + ((p[2] - me[2]) as i64).pow(2);
        g.npcs.iter().filter(|n| friends.contains(&n.id) && !n.hidden).map(|n| (n.id, n.pos, d2(n.pos))).min_by_key(|p| p.2)
    };
    for _ in 0..60 {
        let Some((id, _, d2)) = nearest(g) else { break };
        if d2 > 1500 * 1500 {
            up_to(g, id, 1000);
        }
        let cine = play(g, 30 * 6, |g, f| {
            let Some((_, at, d2)) = nearest(g) else { return vec![] };
            let mut keys = if d2 > 700 * 700 { steer_at(g, at).to_vec() } else { vec![] };
            if d2 < 1000 * 1000 && f % 4 == 0 {
                keys.push(input::TRIANGLE);
            }
            keys
        });
        match cine {
            Some(18) => g.played(18),
            Some(19) => return,
            Some(n) => panic!("cine {n}"),
            None => {}
        }
    }
    panic!("not won: {}", g.script.as_ref().map_or(String::new(), |s| s.describe()));
}

/// Level 4: coming within 2000 of Cynthia at her hiding spot wins (cine
/// 48).
#[test]
fn level_4_cynthia_is_found() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 4);
    win_4(&mut g);
}

/// Level 4 won by play (the test above).
fn win_4(g: &mut Game) {
    // To Cynthia where she waits (the level draws her at a hiding spot,
    // group 0); the clock running out hides her at a spot not used yet, so
    // after her at the newest until found.
    let mut cine = None;
    let mut known: Vec<usize> = Vec::new();
    let mut at = [0; 3];
    for _ in 0..4 {
        for (k, s) in g.spots.iter().enumerate().filter(|(_, s)| s.group == 0 && s.used) {
            if !known.contains(&k) {
                known.push(k);
                at = s.pos;
            }
        }
        // On the floor 1000 short of her (Spike's body stands above its feet).
        let floor = floor_under(g, at[0], at[2] - 1000, at[1] - 1500, 0).expect("a floor by Cynthia");
        put(g, [at[0], floor + g.player.feet(), at[2] - 1000], 0);
        run(g, &[(&[], 10)], |g, _| cine = cine.or(g.cine.take()));
        if cine.is_some() {
            break;
        }
    }
    assert_eq!(cine, Some(48));
}

/// Walks into room `room` (put at its middle, a step), and the cinematic
/// it plays, if any.
fn enter_room(g: &mut Game, room: usize) -> Option<usize> {
    let r = g.set.room_info[room];
    let mid = [0, 1, 2].map(|i| (r.min[i] + r.max[i]) / 2);
    if checking() {
        // Some floor of it reached, or floating, some of it.
        let within = |x: i32, y: i32, z: i32| (0..3).all(|i| [x, y, z][i] >= r.min[i] && [x, y, z][i] <= r.max[i]);
        let inside = if g.player.float.is_some() {
            afloat(g, g.player.pos).iter().any(|&(x, y, z)| within(x, y, z))
        } else {
            on_foot(g, g.player.pos).iter().any(|(&(cx, cz), fs)| fs.iter().any(|&f| within(cx * 256 + 128, f, cz * 256 + 128)))
        };
        if !inside {
            let mut rooms = std::collections::BTreeMap::<usize, usize>::new();
            for (k, fs) in on_foot(g, g.player.pos) {
                for f in fs {
                    if let Some(r) = g.room_of([centre(k.0), f - 100, centre(k.1)]) {
                        *rooms.entry(r).or_default() += 1;
                    }
                }
            }
            not_on_foot(g, format!("room {room} (rooms reached {rooms:?})"));
        }
    }
    if walking() && g.player.float.is_some() {
        if let Some(cine) = float_to(g, room) {
            return cine;
        }
        not_walked(g, format!("room {room}"));
    } else if walking() {
        let inside = |s: Spot| {
            let p = [centre(s.0), s.2, centre(s.1)];
            (0..3).all(|i| p[i] >= r.min[i] && p[i] <= r.max[i])
        };
        if walk_to(g, inside, None) {
            let mut cine = None;
            run(g, &[(&[], 10)], |g, _| cine = cine.or(g.cine.take()));
            return cine;
        }
        not_walked(g, format!("room {room}"));
    }
    // The room is checked, not its middle.
    let check = ON_FOOT.with(|c| c.borrow_mut().take());
    put(g, mid, g.player.heading);
    ON_FOOT.with(|c| *c.borrow_mut() = check);
    let mut cine = None;
    run(g, &[(&[input::UP], 2), (&[], 10)], |g, _| cine = cine.or(g.cine.take()));
    cine
}

/// Level 5's elevator (template 855) from the upper deck: walked up to
/// on the deck beside it, onto its middle with the stick (it starts down
/// once she lands on it), then off at the bottom toward the lower deck.
fn ride_elevator(g: &mut Game) {
    // Its box is in `solid` from a frame on.
    run(g, &[(&[], 1)], |_, _| {});
    let b = *g.solid.iter().find(|b| b.template == 855).expect("the elevator");
    let top = b.min_y;
    let mid = b.pos + b.rot * rrt::glam::Vec3::new((b.lo[0] + b.hi[0]) as f32 / 2.0, 0.0, (b.lo[2] + b.hi[2]) as f32 / 2.0);
    let mid = [mid.x as i32, mid.z as i32];
    let by = |s: Spot| {
        let p = [centre(s.0), top, centre(s.1)];
        !b.covers(p, 0) && b.covers(p, 512) && (s.2 - top).abs() < 300
    };
    if !(walking() && walk_to(g, by, None)) {
        if walking() {
            not_walked(g, "the elevator".into());
        }
        put(g, [mid[0], top + g.player.feet(), mid[1]], g.player.heading);
    }
    let start = g.player.pos[1];
    stride_to(g, mid[0], mid[1], 200, 60);
    // Down to the bottom (it stays there while ridden), and off it toward
    // the lower deck (phase 2's respawn place, (73246, 6697, 9170)).
    for _ in 0..1500 {
        run(g, &[(&[], 1)], |_, _| {});
        if g.player.pos[1] > 5600 {
            stride_to(g, 73246, 9170, 400, 40);
            break;
        }
    }
    assert!(g.player.pos[1] > 6600, "off the elevator at the lower deck from {start}: at {:?}", g.player.pos);
}

/// Level 5: room 30's cine 38 sets Angelica floating through the ducts;
/// the garage, room 38, plays cine 40: won.
#[test]
fn level_5_outer_space_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 5);
    win_5(&mut g);
}

/// Level 5 won by play (the test above).
fn win_5(g: &mut Game) {
    ride_elevator(g);
    assert_eq!(enter_room(g, 30), Some(38));
    g.played(38);
    assert!(g.player.float.is_some(), "floating after cine 38");
    assert_eq!(enter_room(g, 38), Some(40));
    g.played(40);
    assert_eq!(g.won, Some(true));
}

/// Level 6 from the hand-over: room 11's cine 46 gives the level to Tommy,
/// four Reptar boxes stacked by the step take him onto the platform, and
/// Triangle at the switch there plays cine 47: won.
#[test]
fn level_6_toy_palace_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    win_6(&mut g);
}

/// Level 6 won by play (the test above).
fn win_6(g: &mut Game) {
    assert_eq!(enter_room(g, 11), Some(46));
    g.played(46);
    box_route(g);
    let switch = g.set.placements.iter().find(|p| p.kind == 1 && p.cine == 47).expect("the switch").pos;
    let me = g.player.pos;
    let heading = ((switch[0] - me[0]) as f32).atan2((switch[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
    put(g, [switch[0], me[1], switch[2] - 600], heading as i32 & 4095);
    let mut cine = None;
    run(g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 5)], |g, _| cine = cine.or(g.cine.take()));
    assert_eq!(cine, Some(47), "at {:?}, switch {switch:?}", g.player.pos);
    g.played(47);
    assert_eq!(g.won, Some(true));
}

/// Plays with a policy: each frame `pick` chooses the buttons from the
/// game; until a cinematic or `frames` run out.
fn play(g: &mut Game, frames: usize, mut pick: impl FnMut(&mut Game, usize) -> Vec<usize>) -> Option<usize> {
    let mut pads = Pads::default();
    for f in 0..frames {
        let bits = pick(g, f).iter().fold(0u16, |a, &b| a | 1 << b);
        pads.push_bits(0, bits);
        g.frame(&pads);
        if let Some(n) = g.cine.take() {
            return Some(n);
        }
    }
    None
}

/// Levels 20 and 21: eat (Circle) while Angelica is away, lob the box or
/// the bottle (Triangle) to another baby when she comes near; the last
/// cookie or drink wins (cines 64 and 50).
#[test]
fn levels_20_and_21_are_won() {
    let Some(disc) = disc() else { return };
    // The box leaves the hand at the throw's release, so throw before
    // Angelica is in reach of it.
    for (level, won, lob) in [(20, 64, 4500i64), (21, 50, 3300)] {
        let mut g = open(&disc, level);
        win_box_toss(&mut g, won, lob);
    }
}

/// A box toss level (20, 21) won by play: `won` its ending, `lob` the
/// reach at which the box goes.
fn win_box_toss(g: &mut Game, won: usize, lob: i64) {
    let level = g.level.0;
    let mut cine = None;
    for _ in 0..3 {
        cine = play(g, 6000, |g, f| {
            let me = g.player.pos;
            let d2 = |p: [i32; 3], q: [i32; 3]| {
                let (dx, dz) = ((p[0] - q[0]) as i64, (p[2] - q[2]) as i64);
                dx * dx + dz * dz
            };
            let Some(angelica) = g.npcs.iter().find(|n| n.id == 600).map(|n| n.pos) else { return vec![] };
            if d2(angelica, me) > lob * lob {
                return if f % 2 == 0 { vec![input::CIRCLE] } else { vec![] };
            }
            // Turn to the baby farthest from her with the stick and lob.
            let babies = [1776u16, 609, 608, 1492];
            let far = g.npcs.iter().filter(|n| babies.contains(&n.id) && !n.hidden).max_by_key(|n| d2(n.pos, angelica));
            if let Some(b) = far {
                let h = ((b.pos[0] - me[0]) as f32).atan2((b.pos[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
                let h = (h as i32).rem_euclid(4096);
                // Out of the target's cone (512): a tap of the stick toward
                // it (a step), else the lob.
                if ((h - g.player.heading + 2048).rem_euclid(4096) - 2048).abs() > 384 {
                    return toward(g, h).to_vec();
                }
            }
            if f % 2 == 0 { vec![input::TRIANGLE] } else { vec![] }
        });
        if cine == Some(won) {
            break;
        }
        // A lost round: its cinematic, then the next.
        if let Some(n) = cine {
            g.played(n);
        }
    }
    assert_eq!(cine, Some(won), "level {level}: {}", g.script.as_ref().map_or(String::new(), |s| s.describe()));
}

/// Level 20's lob (0x80057bc4) carries the box to a baby near or far: the
/// catch swaps the player into it.
#[test]
fn the_lob_reaches_the_baby() {
    let Some(disc) = disc() else { return };
    for d in [2500, 5000, 8000] {
        let mut g = open(&disc, 20);
        play(&mut g, 30, |_, _| vec![]);
        let baby = g.npcs.iter().find(|n| n.id == 609).map(|n| n.pos).unwrap();
        let me = g.player.pos;
        put(&mut g, [baby[0], me[1], baby[2] + d], 0);
        face(&mut g, baby);
        let before = g.character;
        play(&mut g, 200, |_, f| if f == 1 { vec![input::TRIANGLE] } else { vec![] });
        assert_ne!(g.character, before, "{d} away: {}", g.script.as_ref().map_or(String::new(), |s| s.describe()));
    }
}

/// The player's table states as the oracle shows them on level 3
/// (0x800e2b48 +0xa8): running alternates 3 and 4, walking 1 and 2, a
/// jump rises in 5 and comes down in 6, stopped is 0.
#[test]
fn the_animations_follow_the_original_states() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    let seen = |g: &mut Game, steps: &[(&[usize], usize)]| {
        let mut states = Vec::new();
        run(g, steps, |g, _| {
            let s = g.player.track().0;
            if states.last() != Some(&s) {
                states.push(s);
            }
        });
        states
    };
    seen(&mut g, &[(&[], 20)]);
    let run_states = seen(&mut g, &[(&[input::DOWN], 60)]);
    assert!(run_states.windows(2).any(|w| w == [3, 4]) && run_states.windows(2).any(|w| w == [4, 3]), "{run_states:?}");
    assert_eq!(seen(&mut g, &[(&[], 30)]).last(), Some(&0));
    let walk = seen(&mut g, &[(&[input::SQUARE, input::DOWN], 60)]);
    assert!(walk.windows(2).any(|w| w == [1, 2]), "{walk:?}");
    seen(&mut g, &[(&[], 30)]);
    let jump = seen(&mut g, &[(&[input::CROSS], 4), (&[], 40)]);
    assert!(jump.windows(2).any(|w| w == [5, 6]), "{jump:?}");
    // Standing still: 1200 dt, a fidget (23), standing, the other (24).
    let idle = seen(&mut g, &[(&[], 400)]);
    assert!(idle.windows(3).any(|w| w == [23, 0, 24] || w == [24, 0, 23]), "{idle:?}");
}

/// A standing jump sounds 6 when its rise gives way to its fall (state 6)
/// and 11 on landing (state 8): 0x800562bc on entering them.
#[test]
fn a_jump_sounds_at_its_states() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    run(&mut g, &[(&[], 20)], |_, _| {});
    let mut heard = Vec::new();
    run(&mut g, &[(&[input::CROSS], 2), (&[], 60)], |g, f| {
        heard.extend(g.effects.iter().map(|e| (f, e.sound)));
    });
    let at = |n: u8| heard.iter().find(|h| h.1 == n).map(|h| h.0);
    let (fall, land) = (at(6), at(11));
    assert!(fall.is_some() && land.is_some() && fall < land, "{heard:?}");
    assert!(fall > Some(2), "the 6 is not the take-off: {heard:?}");
}

/// Level 9's phase 2 (0x8007cac4 after cine 18): 46 is home still in
/// state 19 and waits there while the other two chase; Tommy within a
/// bite wakes him and he chases from then on.
#[test]
fn level_9_mr_friend_waits_after_his_return() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 9);
    g.played(18);
    let home = |g: &Game| g.npcs.iter().find(|n| n.id == 46).map(|n| n.pos).unwrap();
    let at = home(&g);
    put(&mut g, [at[0] + 6000, at[1], at[2]], 0);
    run(&mut g, &[(&[], 120)], |_, _| {});
    assert_eq!(home(&g), at, "46 waits at home");
    put(&mut g, [at[0] + 300, at[1], at[2]], 0);
    let health = g.player.health;
    run(&mut g, &[(&[], 60)], |_, _| {});
    assert!(g.player.health < health, "46 bites");
    let me = [at[0] + 6000, at[1], at[2]];
    put(&mut g, me, 0);
    run(&mut g, &[(&[], 60)], |_, _| {});
    assert_ne!(home(&g), at, "46 chases");
}

/// Level 9: a can (template 716, bounce 3200) thrown at Mr. Friend hurts
/// him 30, bounces off him and comes back into Tommy, hurting him 30
/// (0x80087594; the oracle's 100 -> 70 and 255 -> 225).
#[test]
fn level_9_a_can_bounces_back() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 9);
    let Some(can) = g.things.iter().position(|t| t.template == 716) else { panic!("no can") };
    let k = g.npcs.iter().position(|n| n.id == 46).unwrap();
    let p = g.npcs[k].pos;
    put(&mut g, [p[0], p[1], p[2] - 1500], 0);
    g.npcs[k].mode = rugrats::actors::npc::Mode::Idle;
    g.give(can);
    let (his, mine) = (g.npcs[k].health, g.player.health);
    run(&mut g, &[(&[], 5), (&[input::TRIANGLE], 1), (&[], 60)], |g, _| {
        let n = g.npcs.iter().position(|n| n.id == 46).unwrap();
        g.npcs[n].mode = rugrats::actors::npc::Mode::Idle;
    });
    assert_eq!(g.npcs[k].health, his - 30, "Mr. Friend hit");
    assert_eq!(g.player.health, mine - 30, "the can came back");
}

/// After a cry the follow camera comes back (0x800290a4), but levels 9,
/// 20 and 21 keep a room's fixed camera (0x8002953c).
#[test]
fn the_camera_after_a_cry() {
    let Some(disc) = disc() else { return };
    let fixed_after_cry = |level: u8| -> Option<bool> {
        let mut g = open(&disc, level);
        let room = (0..g.set.room_info.len()).find(|&k| {
            let r = g.set.room_info[k];
            r.flags & 4 != 0 && r.camera == 3 && r.flags & 2 == 0
        })?;
        enter_room(&mut g, room);
        g.fixed_camera()?;
        g.cry();
        run(&mut g, &[(&[], 400)], |_, _| {});
        Some(g.fixed_camera().is_some())
    };
    assert_eq!(fixed_after_cry(9), Some(true), "level 9 keeps it");
    let other = [2u8, 3, 6, 10, 11, 12, 13, 15, 17, 18].into_iter().find_map(fixed_after_cry);
    assert_eq!(other, Some(false), "elsewhere the follow camera");
}

/// Golf: a ball rolling near the cup is watched from its room's fixed
/// camera once the golf camera's delay is out (0x8002953c, 0x800259a0).
#[test]
fn golf_watches_the_ball_near_the_cup() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 27);
    let cup = g.set.placements.iter().find(|p| p.kind == 1 && p.template == 789).map(|p| p.pos).unwrap();
    let b = ball(&g).unwrap();
    let a = ((cup[0] - b[0]) as f64).atan2((cup[2] - b[2]) as f64) - 256.0 * std::f64::consts::TAU / 4096.0;
    let aim = [b[0] + (a.sin() * 4000.0) as i32, b[1], b[2] + (a.cos() * 4000.0) as i32];
    put(&mut g, [b[0], b[1] - 100, b[2] - 700], 0);
    run(&mut g, &[(&[], 60)], |_, _| {});
    golf_aim(&mut g, aim);
    let posts: Vec<[i32; 3]> = g.posts.iter().map(|p| p.pos).collect();
    let mut watched = false;
    run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 70), (&[input::TRIANGLE], 1), (&[], 300)], |g, _| {
        let e = g.view.0;
        watched |= posts.iter().any(|p| (p[0] as f32 - e.x).abs() < 1.0 && (p[2] as f32 - e.z).abs() < 1.0);
    });
    assert!(watched, "no post camera on the way to the cup");
}

/// Level 5's elevator (template 855): ridden from the top it goes down its
/// 12288 at 60 a frame (the oracle: -4578 to about 7710 in 410 VSyncs,
/// 205 frames, from its arming by the landing), carrying Tommy.
#[test]
fn level_5_the_elevator_goes_down() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 5);
    let k = g.movers.iter().position(|m| m.template == 855).expect("the elevator");
    let at = g.movers[k].pos;
    put(&mut g, [at[0], at[1] - 1500, at[2]], 0);
    let (mut lowest, mut moving) = ((0, 0), 0);
    run(&mut g, &[(&[], 260)], |g, f| {
        let y = g.movers[k].pos[1];
        if y > lowest.0 {
            lowest = (y, f);
        }
        if moving == 0 && y != at[1] {
            moving = f;
        }
    });
    // 205 steps of 60 pass the range by 12 (the think does not clamp).
    assert_eq!(lowest.0, at[1] + 12300, "down by its range");
    assert!((204..=206).contains(&(lowest.1 - moving)), "in about 205 frames: {lowest:?} from {moving}");
}

/// The hub's fetch (docs/levels/level-01.md, the oracle's run): toy 679
/// thrown from (44000, 700, 36000) facing +z; Spike gets up, runs to it,
/// carries it in his mouth (model 1214), brings it back and puts it down
/// (speech 190).
#[test]
fn hub_spike_fetches_a_toy() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    let toy = g.things.iter().position(|t| t.template == 679).expect("toy 679");
    put(&mut g, [44000, 700, 36000], 0);
    g.give(toy);
    run(&mut g, &[(&[], 90)], |_, _| {});
    let spike = g.npcs.iter().position(|n| n.id == 14).unwrap();
    let mouth = |g: &Game| {
        let n = &g.npcs[spike];
        n.hierarchy.header.get(0x24).and_then(|&p| n.hierarchy.parts.get(p as usize)).map(|p| p.model)
    };
    let (mut carried, mut put_down) = (false, false);
    run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 600)], |g, _| {
        carried |= mouth(g) == Some(1214);
        put_down |= carried && g.speech.contains(&190);
    });
    assert!(carried, "in his mouth");
    assert!(put_down, "brought back");
    assert_eq!(mouth(&g), Some(79), "his own mouth again");
}

/// The throw (logical 18, table 26, over the base on track 1) at speed
/// 0x2000 lets go at its event 16 and ends with its track.
#[test]
fn a_throw_runs_at_its_speed() {
    use rugrats::actors::player::anim::THROW;
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    run(&mut g, &[(&[], 5)], |_, _| {});
    g.player.act_at(THROW, rugrats::world::game::THROW_ANIM_SPEED, &g.anims);
    let (mut released, mut frames) = (None, 0);
    for f in 0..200 {
        g.player.animator.step(&g.anims);
        if released.is_none() && g.player.animator.events & 16 != 0 {
            released = Some(f);
        }
        if !g.player.animator.playing(THROW) {
            frames = f;
            break;
        }
    }
    // Raw durations 30, 10, 10, 10, 10, 10 (x 5 in the port's units) at
    // 20 a frame: the release after 25 of 400, the end at 400.
    assert_eq!(released, Some(12));
    assert_eq!(frames, 19);
}

/// Turns the player to face `to` with the stick (`turn_to`).
fn face(g: &mut Game, to: [i32; 3]) {
    let me = g.player.pos;
    let h = ((to[0] - me[0]) as f32).atan2((to[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
    turn_to(g, (h as i32).rem_euclid(4096));
}

/// Level 13: Tommy into the kitchen (room 20) before Angelica reaches her
/// last post plays cine 21: won.
#[test]
fn level_13_cookie_race_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 13);
    win_13(&mut g);
}

/// Level 13 won by play (the test above).
fn win_13(g: &mut Game) {
    run(g, &[(&[], 30)], |_, _| {});
    assert_eq!(enter_room(g, 20), Some(21));
}

/// Level 2: the gap in the fence (room 69, cine 3), the ride after the
/// goose to the court (room 44, cine 4), then pucks kicked at the goose
/// (Triangle, facing it) until it falls: cine 5. Each hit on the goose
/// throws feathers (0x800509d8: sprite 276, kind 5).
#[test]
fn level_2_grandpas_teeth_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 2);
    win_2(&mut g);
}

/// Level 2 won by play (the test above).
fn win_2(g: &mut Game) {
    assert_eq!(enter_room(g, 69), Some(3));
    g.played(3);
    assert_eq!(enter_room(g, 44), Some(4));
    g.played(4);
    // A clear shot: the goose in range, Chuckie off the line to it.
    let feathers = std::cell::Cell::new(0);
    let cine = play(g, 30 * 130, |g, f| {
        let n = g.flights.iter().filter(|f| f.kind() == 5 && f.skin == flights::Skin::Sprite(276)).count();
        feathers.set(feathers.get().max(n));
        let pos = |id: u16| g.npcs.iter().find(|n| n.id == id).map(|n| n.pos);
        let (Some(goose), Some(chuckie)) = (pos(11), pos(1776)) else { return vec![] };
        let me = g.player.pos;
        let (gx, gz) = ((goose[0] - me[0]) as f64, (goose[2] - me[2]) as f64);
        let (cx, cz) = ((chuckie[0] - me[0]) as f64, (chuckie[2] - me[2]) as f64);
        let len = gx.hypot(gz).max(1.0);
        let along = (cx * gx + cz * gz) / len;
        let off = (cx * gz - cz * gx).abs() / len;
        let clear = off > 900.0 || along < 0.0 || along > len + 600.0;
        let mut keys = steer_at(g, goose).to_vec();
        if f % 4 == 0 && len < 14000.0 && clear {
            keys.push(input::TRIANGLE);
        }
        keys
    });
    let goose = g.npcs.iter().find(|n| n.id == 11).map(|n| (n.pos, n.health, n.hidden, format!("{:?}", n.mode)));
    let chuckie = g.npcs.iter().find(|n| n.id == 1776).map(|n| (n.pos, format!("{:?}", n.mode)));
    let court = g.set.room_info.get(44).map(|r| (r.min, r.max));
    assert_eq!(cine, Some(5), "goose {goose:?}, chuckie {chuckie:?}, court {court:?}, Tommy at {:?}", g.player.pos);
    assert!(feathers.get() >= 5, "{} feathers", feathers.get());
}

/// Puts the player 600 short of `at`, facing it, and presses Triangle;
/// the cinematic that follows, if any.
fn use_at(g: &mut Game, at: [i32; 3]) -> Option<usize> {
    use_from(g, at, [0, -600])
}

/// Triangle at `at` from the side `(dx, dz)` off it.
fn use_from(g: &mut Game, at: [i32; 3], side: [i32; 2]) -> Option<usize> {
    put(g, [at[0] + side[0], at[1], at[2] + side[1]], 0);
    face(g, at);
    let mut cine = None;
    run(g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 5)], |g, _| cine = cine.or(g.cine.take()));
    cine
}

/// Level 12: Triangle at the refrigerator (cine 67) puts Spike somewhere
/// in the dark house; Triangle beside him plays cine 68: won.
#[test]
fn level_12_let_there_be_light_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 12);
    win_12(&mut g);
}

/// Level 12 won by play (the test above).
fn win_12(g: &mut Game) {
    let fridge = g.set.placements.iter().find(|p| p.kind == 1 && p.id == 602).expect("the refrigerator").pos;
    // Its door faces -x.
    assert_eq!(use_from(g, fridge, [-600, 0]), Some(67));
    g.played(67);
    let spike = g.npcs.iter().find(|n| n.id == 14 && !n.hidden).expect("Spike out").pos;
    assert_eq!(use_at(g, spike), Some(68));
}

/// Level 3: the balloon (cine 8) to the bakery; the pie toss,
/// throwing at whichever of Larry and Steve is nearer, until four hits
/// (cine 13); the aisles, then the boss's room (36, cine 86); Triangle at
/// the switch: cine 14, won.
#[test]
fn level_3_aisle_seven_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    win_3(&mut g);
}

/// Level 3 won by play (the test above).
fn win_3(g: &mut Game) {
    // Room 19 (the cart, cine 9) is behind a ledge too high to climb, in
    // the original too: the balloon.
    let balloon = g.set.placements.iter().find(|p| p.id == 81).expect("the balloon").pos;
    assert_eq!(use_at(g, balloon), Some(8));
    g.played(8);
    let cine = play(g, 30 * 120, |g, f| {
        let me = g.player.pos;
        let near = g.npcs.iter().filter(|n| matches!(n.id, 75 | 76) && !n.hidden).min_by_key(|n| {
            let (dx, dz) = ((n.pos[0] - me[0]) as i64, (n.pos[2] - me[2]) as i64);
            dx * dx + dz * dz
        });
        let mut keys = near.map_or(vec![], |n| steer_at(g, n.pos).to_vec());
        if f % 15 == 0 {
            keys.push(input::TRIANGLE);
        }
        keys
    });
    assert_eq!(cine, Some(13), "the toss");
    g.played(13);
    assert_eq!(enter_room(g, 36), Some(86));
    g.played(86);
    // The switch, LP 278 (+0x44 16, which 0x8004691c makes cine 14).
    let switch = g.set.placements[278].pos;
    // The boss may bite him twice on the way: a cry, back to the aisles,
    // and room 36's cine 86 again on the way back in.
    for _ in 0..3 {
        match use_at(g, switch) {
            Some(14) => return,
            Some(86) => g.played(86),
            c => panic!("the switch: {c:?}"),
        }
    }
    panic!("the switch never reached");
}

/// Level 7: Reptar walks the city to City Hall (room 6): cine 85, the end
/// of the game.
#[test]
fn level_7_reptar_2010_reaches_city_hall() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 7);
    win_7(&mut g);
}

/// Level 7 won by play (the test above).
fn win_7(g: &mut Game) {
    assert_eq!(enter_room(g, 6), Some(85));
    g.played(85);
    assert!(g.ending);
}

/// Level 10: Chuckie tags each hiding baby (Triangle beside it), then waits
/// in the playpen, which catches it as it runs home (cines 75-77); the
/// third catch plays cine 78: won.
#[test]
fn level_10_chuckies_glasses_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 10);
    win_10(&mut g);
}

/// Level 10 won by play (the test above).
fn win_10(g: &mut Game) {
    // Home is the playpen. A baby on his floor first; a race lost
    // (cine 41-43) hides that baby again elsewhere and costs a try, three
    // lost end the level.
    // The playpen (template 1032 at (5926, -1387, 21212)), on its floor:
    // cine 44 leaves Chuckie outside it.
    let pen = [5648, 670, 19228];
    let mut left = vec![608u16, 609, 1492];
    let mut lost = 0;
    while !left.is_empty() {
        // Those on the pen's floor first (the stairs are slow), the nearest
        // of them.
        let d = |p: [i32; 3]| ((p[1] - pen[1]).abs() > 1000, ((p[0] - pen[0]) as i64).pow(2) + ((p[2] - pen[2]) as i64).pow(2));
        let id = *left.iter().min_by_key(|&&id| g.npcs.iter().find(|n| n.id == id).map_or((true, i64::MAX), |n| d(n.pos))).unwrap();
        // Tagged within 900 (0x800831a4).
        up_to(g, id, 850);
        let baby = g.npcs.iter().find(|n| n.id == id).expect("a baby").pos;
        // Roughly toward her is enough for the tag; the race starts at it.
        let me = g.player.pos;
        let h = ((baby[0] - me[0]) as f32).atan2((baby[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
        turn_near(g, (h as i32).rem_euclid(4096), 300);
        run(g, &[(&[input::TRIANGLE], 1), (&[], 2)], |_, _| {});
        let mode = g.npcs.iter().find(|n| n.id == id).map(|n| format!("{:?}", n.mode));
        let tagged = g.frames;
        put(g, pen, 0);
        let mut cine = g.cine.take();
        if std::env::var_os("WALKLOG").is_some() {
            eprintln!("BABY {id} at {baby:?}: home in {}, after Triangle {mode:?}, me {:?}, cine {cine:?}, now {:?} at {:?}", g.frames - tagged, g.player.pos,
                g.npcs.iter().find(|n| n.id == id).map(|n| format!("{:?}", n.mode)), g.npcs.iter().find(|n| n.id == id).map(|n| n.pos));
        }
        run(g, &[(&[], 30)], |g, _| cine = cine.or(g.cine.take()));
        match cine {
            Some(75..=77) => left.retain(|&b| b != id),
            Some(41..=43) => lost += 1,
            c => panic!("baby {id}: {c:?}"),
        }
        assert!(lost < 3, "three races lost");
        g.played(cine.unwrap());
    }
    assert_eq!(g.cine.take(), Some(78));
}

/// Finishes cinematic `n` and any the level chains after it (an intro
/// that follows a win), as the app plays them in turn.
fn finish(g: &mut Game, mut n: usize) {
    loop {
        g.played(n);
        match g.cine.take() {
            Some(next) => n = next,
            None => return,
        }
    }
}

/// Level 11, all three rings: Spike eats the twelve bones stand by stand
/// (cine 54); the tricycle into the ramp with Cross pressed in the middle
/// band of distances lands in the net (cine 57); Angelica's meter stopped
/// in both zones launches Chuckie true (cine 39, then 62): won.
#[test]
fn level_11_circus_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 11);
    win_11(&mut g);
}

/// Level 11 won by play (the test above).
fn win_11(g: &mut Game) {
    // Ring 1: the bone moves to the next stand each time; Spike jumps
    // through it ("jumps or walks into the bone above" its stand).
    let mut cine = None;
    let feet = g.player.feet();
    let p = g.player.pos;
    let ground_y = floor_under(g, p[0], p[2], p[1] - feet - 400, 0).expect("the ring's floor") + feet;
    // A jump that misses leaves the bone where it is: again until twelve
    // (cine 54), within the ring's 110 s.
    for k in 0..30 {
        if cine.is_some() {
            break;
        }
        let b = g.goodies.iter().find(|b| b.kind == 14 && !b.taken).expect("a bone").pos;
        let bone = [b.x as i32, b.z as i32];
        // A running jump through it (Spike's rises past the bone's height
        // from about 800 to 1750 along): the takeoff 1250 short, on the
        // floor, from a side with its run-up clear.
        // The side whose run-up starts nearest Spike first.
        let me = g.player.pos;
        let mut sides: Vec<i32> = (0..16).map(|d| d * 256).collect();
        sides.sort_by_key(|&h| {
            let a = h as f32 * std::f32::consts::TAU / 4096.0;
            let back = [bone[0] - (a.sin() * 1762.0) as i32, bone[1] - (a.cos() * 1762.0) as i32];
            ((back[0] - me[0]) as i64).pow(2) + ((back[1] - me[2]) as i64).pow(2)
        });
        let takeoff = sides.into_iter().find_map(|h| {
            let a = h as f32 * std::f32::consts::TAU / 4096.0;
            let at = |t: f32| [bone[0] - (a.sin() * t) as i32, bone[1] - (a.cos() * t) as i32];
            let clear = [1250.0, 1500.0, 1750.0].iter().all(|&t| {
                let [x, z] = at(t);
                floor_under(g, x, z, ground_y - feet - 384, 0) == Some(ground_y - feet) && !Ground::new(g).blocked(x, z, ground_y - feet)
            });
            // Nothing to hold the run up before it (the step's probes from
            // the takeoff, Spike's 1000 deep).
            let [tx, tz] = at(1250.0);
            let from = [tx, ground_y, tz];
            let probes = g.player.probes([a.sin(), a.cos()], 200.0);
            // (The bone's own stand aside: jumped at.)
            let stand = |b: &&rugrats::actors::player::Box3| b.covers([bone[0], 0, bone[1]], 400);
            let free = !g.bodies.iter().filter(|b| !b.character && b.min_y < ground_y - feet - 384 && !stand(b)).any(|b| {
                let hy = g.player.probe_height(b, from);
                probes.iter().any(|&(px, pz)| b.segment_hit([tx, hy, tz], [tx + px, hy, tz + pz]).is_some())
            });
            (clear && free).then_some((at(1250.0), h))
        });
        let Some(([x, z], h)) = takeoff else {
            for d in 0..16 {
                let a = (d * 256) as f32 * std::f32::consts::TAU / 4096.0;
                let [x, z] = [bone[0] - (a.sin() * 1250.0) as i32, bone[1] - (a.cos() * 1250.0) as i32];
                eprintln!("d {d}: floor {:?} blocked {}", floor_under(g, x, z, ground_y - feet - 384, 0), Ground::new(g).blocked(x, z, ground_y - feet));
            }
            panic!("no takeoff for the bone at {b:?}, ground {ground_y} feet {feet}");
        };
        // To the run-up's start (`leap_by_play` backs up there).
        let a = h as f32 * std::f32::consts::TAU / 4096.0;
        // Up to 700 behind the run-up, then along the line to it: there
        // facing the bone, with no turn to make.
        let t0 = g.frames;
        near(g, [x - (a.sin() * 1212.0) as i32, ground_y, z - (a.cos() * 1212.0) as i32], h);
        if std::env::var_os("WALKLOG").is_some() {
            eprintln!("  near took {}", g.frames - t0);
        }
        let t1 = g.frames;
        // Within 64 (the turn's 41 off took a jump 380 wide of the bone,
        // its box 375 each way).
        leap_within(g, x, z, h, 64);
        if std::env::var_os("WALKLOG").is_some() {
            eprintln!("  leap took {}", g.frames - t1);
        }

        run(g, &[(&[], 4)], |g, _| cine = cine.or(g.cine.take()));
        cine = cine.or(g.cine.take());
        if std::env::var_os("WALKLOG").is_some() {
            eprintln!("bone {k} at {b:?}: takeoff {x},{z} h {h}, heading {} me {:?}", g.player.heading, g.player.pos);
        }
    }
    assert_eq!(cine, Some(54), "ring 1");
    finish(g, 54);
    // Ring 2: along +x into the ramp's zone (x 15000..22700), Cross 6700
    // in.
    let y = g.player.pos[1];
    put(g, [13500, y, 34300], 1024);
    let mut from = None;
    let cine = play(g, 30 * 20, |g, _| {
        let x = g.player.pos[0];
        if x >= 15000 && from.is_none() {
            from = Some(x);
        }
        match from {
            Some(f) if x - f >= 6700 => vec![input::UP, input::CROSS],
            _ => vec![input::UP],
        }
    });
    assert_eq!(cine, Some(57), "ring 2 at {:?}", g.player.pos);
    finish(g, 57);
    // Ring 3: the meter rises 6 a frame from 6; stop it at 174 (28
    // frames), then the second from 46 at 172 (21 frames).
    let presses = [0usize, 28, 49];
    let cine = play(g, 120, |_, f| if presses.contains(&f) { vec![input::TRIANGLE] } else { vec![] });
    assert_eq!(cine, Some(39), "ring 3's leap");
    g.played(39);
    let cine = play(g, 30 * 10, |_, _| vec![]);
    assert_eq!(cine, Some(62), "ring 3");
}

/// The golf ball's position, from the script's describe line.
fn ball(g: &Game) -> Option<[i32; 3]> {
    let d = g.script.as_ref()?.describe();
    let at = d.split("ball [").nth(1)?.split(']').next()?;
    let v: Vec<i32> = at.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    (v.len() == 3).then(|| [v[0], v[1], v[2]])
}

fn stage(g: &Game) -> String {
    g.script.as_ref().map_or(String::new(), |s| s.describe())
}

/// One stroke: walk up to the ball, face `aim`, start the meter, stop it
/// at about `power` (3 a frame from 4), and wait for the ball to rest.
/// The level's exit or cinematic if the stroke ends the hole.
fn stroke(g: &mut Game, aim: [i32; 3], power: i32) -> Option<usize> {
    let b = ball(g)?;
    put(g, [b[0], b[1] - 100, b[2] - 700], 0);
    let mut out = None;
    run(g, &[(&[], 60)], |g, _| out = out.or(g.cine.take()));
    golf_aim(g, aim);
    let wait = ((power - 4).max(0) / 3) as usize;
    run(g, &[(&[input::TRIANGLE], 1), (&[], wait), (&[input::TRIANGLE], 1)], |g, _| out = out.or(g.cine.take()));
    for _ in 0..600 {
        run(g, &[(&[], 1)], |g, _| out = out.or(g.cine.take()));
        if out.is_some() || g.exit.is_some() || stage(g).contains("roll None") && stage(g).contains("Free") {
            break;
        }
    }
    out.or(g.exit.take().map(|e| e.level.0 as usize + 1000))
}

/// A stroke aimed `offset` off the cup's bearing, at `power`.
#[derive(Clone, Copy, Debug)]
struct Shot {
    offset: i32,
    power: i32,
}

/// Replays `shots` on a fresh hole: the result, if one ended it, and the
/// game after.
fn replay(disc: &GameDisc, level: u8, shots: &[Shot]) -> (Option<usize>, Game) {
    let mut g = open(disc, level);
    let cup = g.set.placements.iter().find(|p| p.kind == 1 && p.template == 789).map(|p| p.pos).unwrap();
    for s in shots {
        let Some(b) = ball(&g) else { break };
        let bearing = ((cup[0] - b[0]) as f64).atan2((cup[2] - b[2]) as f64);
        let a = bearing + s.offset as f64 * std::f64::consts::TAU / 4096.0;
        let aim = [b[0] + (a.sin() * 4000.0) as i32, b[1], b[2] + (a.cos() * 4000.0) as i32];
        if let Some(r) = stroke(&mut g, aim, s.power) {
            return (Some(r), g);
        }
    }
    (None, g)
}

/// The distance to the cup along the fairway: a shortest-path field over
/// the COL grid (cells of 250, eight ways), through floor no higher than
/// the cup's own by more than 60.
struct Field {
    origin: [i32; 2],
    w: usize,
    d: Vec<f64>,
}

const CELL: i32 = 250;

impl Field {
    fn new(g: &Game, cup: [i32; 3]) -> Field {
        let (x0, z0) = (g.col.min[0], g.col.min[2]);
        let (w, h) = (((g.col.max[0] - x0) / CELL + 1) as usize, ((g.col.max[2] - z0) / CELL + 1) as usize);
        let cup_floor = rugrats::world::collide::query(&g.col, [cup[0], cup[1] - 50, cup[2]]).floor;
        let open: Vec<bool> = (0..w * h)
            .map(|i| {
                let (x, z) = (x0 + (i % w) as i32 * CELL, z0 + (i / w) as i32 * CELL);
                let q = rugrats::world::collide::query(&g.col, [x, cup[1] - 50, z]);
                q.kind != 2 && q.tile.is_some() && q.floor >= cup_floor - 60
            })
            .collect();
        let mut d = vec![f64::MAX; w * h];
        let at = |x: i32, z: i32| ((z - z0) / CELL) as usize * w + ((x - x0) / CELL) as usize;
        let start = at(cup[0], cup[2]);
        d[start] = 0.0;
        let mut heap = std::collections::BinaryHeap::new();
        heap.push((std::cmp::Reverse(0u64), start));
        while let Some((std::cmp::Reverse(_), i)) = heap.pop() {
            let (cx, cz) = ((i % w) as i64, (i / w) as i64);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if nx < 0 || nz < 0 || nx >= w as i64 || nz >= h as i64 {
                    continue;
                }
                let j = nz as usize * w + nx as usize;
                if !open[j] {
                    continue;
                }
                let nd = d[i] + if dx != 0 && dz != 0 { 1.414 } else { 1.0 } * CELL as f64;
                if nd < d[j] {
                    d[j] = nd;
                    heap.push((std::cmp::Reverse(nd as u64), j));
                }
            }
        }
        Field { origin: [x0, z0], w, d }
    }

    /// The distance to go from `p` (huge off the fairway).
    fn at(&self, p: [i32; 3]) -> f64 {
        let (x, z) = ((p[0] - self.origin[0]) / CELL, (p[2] - self.origin[1]) / CELL);
        if x < 0 || z < 0 || x as usize >= self.w {
            return f64::MAX;
        }
        self.d.get(z as usize * self.w + x as usize).copied().unwrap_or(f64::MAX)
    }
}

/// Plays a hole by search: each stroke the one of a fan of aims and
/// powers that holes out, else ends nearest the cup along the fairway.
fn solve_hole(disc: &GameDisc, level: u8, par: usize, fan: i32) -> Option<(usize, Vec<Shot>)> {
    let mut shots: Vec<Shot> = Vec::new();
    let field = {
        let g = open(disc, level);
        let cup = g.set.placements.iter().find(|p| p.kind == 1 && p.template == 789).map(|p| p.pos).unwrap();
        Field::new(&g, cup)
    };
    while shots.len() < par {
        let mut best: Option<(i64, Shot, Option<usize>)> = None;
        for offset in (-fan..=fan).step_by(128) {
            for power in (30..=218).step_by(17) {
                let s = Shot { offset, power };
                let mut tried = shots.clone();
                tried.push(s);
                let (r, g) = replay(disc, level, &tried);
                if r.is_some_and(|r| r != 0) && !stage(&g).contains("Over") {
                    return Some((r.unwrap(), tried));
                }
                let Some(b) = ball(&g) else { continue };
                let d = field.at(b).min(1e12) as i64;
                if best.is_none_or(|(bd, ..)| d < bd) {
                    best = Some((d, s, r));
                }
            }
        }
        shots.push(best?.1);
    }
    None
}

/// Finds each hole's shots by search (slow: run with --ignored); what it
/// printed is golf_course_is_played's list.
#[test]
#[ignore]
fn golf_course_search() {
    let Some(disc) = disc() else { return };
    let pars = [4usize, 5, 4, 5, 5, 5, 4, 5, 3];
    for (k, level) in [25u8, 28, 24, 29, 23, 30, 26, 31, 27].into_iter().enumerate() {
        let r = solve_hole(&disc, level, pars[k], 2048);
        eprintln!(
            "SEARCH {level}: {:?}",
            r.map(|(r, s)| (r, s.iter().map(|s| (s.offset, s.power)).collect::<Vec<_>>()))
        );
    }
}

/// Mini golf, the nine holes in course order, each played stroke by stroke
/// (walk up to the ball, aim, the power meter stopped, the roll) at or
/// under par; each holes out to the next hole, the ninth to cine 35.
/// The shots are golf_course_search's.
#[test]
fn golf_course_is_played() {
    let Some(disc) = disc() else { return };
    win_golf(&disc);
}

/// The golf course played hole by hole at par or under (the shot list
/// golf_course_search found).
fn win_golf(disc: &GameDisc) {
    let course: [(u8, usize, &[(i32, i32)], usize); 9] = [
        (25, 4, &[(-128, 183), (-896, 200)], 1028),
        (28, 5, &[(0, 217), (0, 64)], 1024),
        (24, 4, &[(-256, 183), (0, 217), (-1536, 115)], 1029),
        (29, 5, &[(256, 217), (0, 115)], 1023),
        (23, 5, &[(256, 217), (512, 217), (-1152, 132)], 1030),
        (30, 5, &[(-384, 200), (-640, 149)], 1026),
        (26, 4, &[(0, 200), (640, 200), (-1664, 64)], 1031),
        (31, 5, &[(-384, 217), (-512, 217), (-384, 200), (-768, 115)], 1027),
        (27, 3, &[(-256, 217)], 35),
    ];
    for (level, par, shots, next) in course {
        let shots: Vec<Shot> = shots.iter().map(|&(offset, power)| Shot { offset, power }).collect();
        assert!(shots.len() <= par);
        let (r, g) = replay(disc, level, &shots);
        assert_eq!(r, Some(next), "hole {level}: {}", stage(&g));
    }
}

/// The cry says 2040 + the character at once (0x8007f920, 0x8008df78):
/// Tommy's 2041 in level 2, a line its speech set holds.
#[test]
fn the_cry_has_its_line() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 2);
    g.cry();
    assert_eq!(g.say_now, Some(2041));
    assert!(g.lines.line(&disc, 2041).is_some_and(|s| !s.is_empty()));
}

/// A lobster on Tommy bites every 180 dt (36 VSyncs in the oracle's
/// level 16), here in level 3's aisles.
#[test]
fn the_lobster_bites_every_36_vsyncs() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    g.played(13);
    let Some(k) = g
        .npcs
        .iter()
        .position(|n| matches!(n.mode, rugrats::actors::npc::Mode::Prowl { .. }) && !n.hidden && n.damage > 0)
    else {
        panic!("no lobster")
    };
    let at = g.npcs[k].pos;
    put(&mut g, [at[0], at[1], at[2] - 300], 0);
    let mut hits = Vec::new();
    let mut last = g.player.health;
    run(&mut g, &[(&[], 120)], |g, f| {
        if g.player.health < last {
            hits.push(f);
        }
        last = g.player.health;
        let n = &g.npcs[k];
        let p = n.pos;
        g.player.pos = [p[0], p[1], p[2] - 300];
    });
    let gaps: Vec<usize> = hits.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(hits.len() >= 4 && gaps.iter().all(|&d| (17..=20).contains(&d)), "bites at {hits:?}");
}

/// The room lookup (0x8002b660) tries the room the player was in, then the
/// rooms its portals lead to, and only then the first room whose bounds
/// hold the point. Level 2's rooms overlap (19 pairs): where two overlap,
/// seen from a neighbour of the later one, the lookup gives the later one.
#[test]
fn rooms_are_found_through_portals() {
    let Some(disc) = disc() else { return };
    let g = open(&disc, 2);
    let rooms = &g.set.room_info;
    let inside = |k: usize, p: [i32; 3]| (0..3).all(|i| rooms[k].min[i] <= p[i] && p[i] <= rooms[k].max[i]);
    let mut seen = 0;
    for a in 0..rooms.len() {
        for b in a + 1..rooms.len() {
            let p: [i32; 3] = std::array::from_fn(|i| {
                (rooms[a].min[i].max(rooms[b].min[i]) + rooms[a].max[i].min(rooms[b].max[i])) / 2
            });
            if !inside(a, p) || !inside(b, p) {
                continue;
            }
            for h in (0..rooms.len()).filter(|&h| g.set.neighbours[h].contains(&b) && !inside(h, p)) {
                let want = g.set.neighbours[h].iter().copied().find(|&k| inside(k, p));
                assert_eq!(g.room_from(p, Some(h)), want, "from {h} at {p:?}");
                if want == Some(b) && g.room_of(p) == Some(a) {
                    seen += 1;
                }
            }
        }
    }
    assert!(seen > 0, "no overlap reached through a portal");
}

/// A thrown thing meets objects (0x80061f88's flags 0x9070700: the world,
/// objects and their touches): dropped over one, it comes to rest on its
/// top, not on the floor under it.
#[test]
fn a_thrown_thing_lands_on_an_object() {
    use rugrats::world::game::ThingState;
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 2)], |_, _| {});
    let k = g.things.iter().position(|t| !t.hidden && !t.world_only).expect("a thing in the hub");
    // A wide object whose top is well above the floor under it.
    let b = g
        .solid
        .iter()
        .find(|b| {
            let c = [b.pos.x as i32, b.min_y, b.pos.z as i32];
            let q = rugrats::world::collide::query(&g.col, [c[0], b.max_y + 100, c[2]]);
            b.hi[0] - b.lo[0] > 600
                && b.hi[2] - b.lo[2] > 600
                && b.covers(c, 0)
                && q.kind == 0
                && q.floor - b.min_y > 300
        })
        .cloned()
        .expect("an object to land on");
    let shape = g.things[k].shape;
    let bottom = g.set.models.get(shape).and_then(|m| m.boxes.iter().map(|x| x.1[1]).max()).unwrap_or(0) as f32;
    g.things[k].pos = rrt::glam::Vec3::new(b.pos.x, b.min_y as f32 - 1500.0, b.pos.z);
    g.things[k].state = ThingState::Flying([0; 3]);
    run(&mut g, &[(&[], 60)], |_, _| {});
    let t = &g.things[k];
    assert_eq!(t.state, ThingState::Rest);
    assert!((t.pos.y + bottom - b.min_y as f32).abs() < 2.0, "rests at {} + {bottom}, the top is {}", t.pos.y, b.min_y);
}

/// Objects are drawn from their rooms' lists; a character placed in no
/// room (the hub's Angelica, id 600; level 7's babies) is in none and is
/// never drawn, while one in a room is.
#[test]
fn characters_in_no_room_are_not_drawn() {
    let Some(disc) = disc() else { return };
    for (level, gone) in [(1u8, &[600u16][..]), (7, &[1492, 1776, 608, 609][..])] {
        let mut g = open(&disc, level);
        run(&mut g, &[(&[], 2)], |_, _| {});
        for &id in gone {
            let k = g.npcs.iter().position(|n| n.id == id).expect("placed");
            assert_eq!(g.npc_rooms[k], None, "level {level} id {id}");
        }
        assert!(level != 1 || g.npc_rooms.iter().any(|r| r.is_some()), "level {level}: no character in a room");
    }
}

/// Hole 26's Reptar (id 9999, mode 8) makes a noise every 1000 dt
/// (0x80085168): sound 144 or 145 at him, or line 1134.
#[test]
fn hole_26_reptar_roars() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 26);
    let mut noises = 0;
    run(&mut g, &[(&[], 250)], |g, _| {
        noises += g.effects.iter().filter(|e| matches!(e.sound, 144 | 145)).count();
        noises += g.speech.iter().filter(|&&n| n == 1134).count();
    });
    assert!(noises >= 2, "{noises} noises in 250 frames");
}

/// A climb is carried by its animation's root keys (0x80055904): Tommy
/// onto the hub's template 130 (a rise of about 850, the 900 band). The
/// oracle's body, every frame of the climb (VSyncs 2706-2734 with Tommy put
/// at (8945, 779, 12500) and the stick held toward it): its height, and
/// its distance along from where the climb began.
#[test]
fn a_climb_moves_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 1)], |_, _| {});
    put(&mut g, [8945, 779, 12500], 0);
    let oracle_y = [725, 683, 653, 475, 358, 303, 314, 256, 126, 25, -50, -97, -84, -92];
    let oracle_along = [0, 1, 1, 52, 79, 79, 73, 94, 141, 294, 366, 353, 401, 419];
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::UP], 15)], |g, _| {
        if g.player.action.is_some() {
            let root = g.player.parts(&g.anims, &g.trig)[0].1;
            seen.push((root.y as i32, root.z as i32));
        }
    });
    assert_eq!(seen.len(), oracle_y.len(), "climb frames {seen:?}");
    let z0 = seen[0].1;
    for (k, &(y, z)) in seen.iter().enumerate() {
        assert!((y - oracle_y[k]).abs() <= 3, "frame {k}: y {y} against {}", oracle_y[k]);
        assert!((z - z0 - oracle_along[k]).abs() <= 4, "frame {k}: along {} against {}", z - z0, oracle_along[k]);
    }
}

/// A standing jump as the original's: the crouch (table state 5, its root
/// keys taking the body 89 down) before it leaves the floor at state 6
/// (0x800562bc, 0x8005ec9c), the arc, and the landing's sink of 144 under
/// the floor taken back 23 a frame. The oracle's body height, a game frame
/// apart, from the press at the hub's start (VSyncs 2706-2746).
#[test]
fn a_jump_moves_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 3)], |_, _| {});
    let oracle =
        [783, 794, 810, 827, 843, 859, 868, 868, 814, 668, 544, 442, 362, 304, 268, 253, 260, 289, 340, 413, 508];
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::CROSS], 1), (&[], 20)], |g, _| seen.push(g.player.parts(&g.anims, &g.trig)[0].1.y as i32));
    for (k, (&y, &o)) in seen.iter().zip(&oracle).enumerate() {
        assert!((y - o).abs() <= 2, "frame {k}: {y} against {o} ({seen:?})");
    }
}

/// Running bobs by the run's root keys, table states 3 and 4 in turn (the
/// oracle's body from the hub's start, the stick held: VSyncs 2716-2750).
#[test]
fn running_bobs_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 3)], |_, _| {});
    let oracle = [746, 739, 756, 761, 769, 780, 751, 731, 720, 753, 773, 778, 747, 739, 756, 761, 769, 780];
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::UP, input::LEFT], 18)], |g, _| seen.push(g.player.parts(&g.anims, &g.trig)[0].1.y as i32));
    for (k, (&y, &o)) in seen.iter().zip(&oracle).enumerate() {
        assert!((y - o).abs() <= 2, "frame {k}: {y} against {o} ({seen:?})");
    }
}

/// A running jump (logical 28, table state 41) has no crouch: one frame in
/// state 41, then off, the same arc as the standing jump's (the oracle's
/// steps from VSync 2742: -146, -124, -102, -80, -58, -36, -15, +7), and
/// the landing played out, the stick held, before the run goes on.
#[test]
fn a_running_jump_moves_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 3), (&[input::UP, input::LEFT], 12)], |_, _| {});
    let mut seen = Vec::new();
    let mut states = Vec::new();
    run(&mut g, &[(&[input::UP, input::LEFT, input::CROSS], 1), (&[input::UP, input::LEFT], 20)], |g, _| {
        seen.push(g.player.parts(&g.anims, &g.trig)[0].1.y as i32);
        states.push(g.player.state);
    });
    let steps: Vec<i32> = seen.windows(2).map(|w| w[1] - w[0]).collect();
    assert_eq!(&steps[..8], &[-146, -124, -102, -80, -58, -36, -15, 7], "{seen:?}");
    let landing = states.iter().position(|s| *s == State::Land).expect("a landing");
    assert!(states[landing..landing + 6].iter().all(|s| *s == State::Land), "{states:?}");
}

/// Chuckie's climb in level 10, on the same seat (template 130): the
/// player's body is 0x800c5680's own character, so its root motion is not
/// scaled. The oracle (level 10 booted as Chuckie, put at (8945, 670,
/// 12500)): his height through the climb (table state 13).
#[test]
fn chuckies_climb_moves_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 10);
    run(&mut g, &[(&[], 1)], |_, _| {});
    put(&mut g, [8945, 670, 12500], 0);
    let oracle = [
        661, 652, 639, 622, 605, 502, 398, 282, 154, 25, 6, -14, -22, -17, -12, -23, -34, -50, -72, -95, -114, -133,
        -152,
    ];
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::UP], 30)], |g, _| {
        if g.player.action.is_some() {
            seen.push(g.player.parts(&g.anims, &g.trig)[0].1.y as i32);
        }
    });
    assert!(seen.len() >= oracle.len(), "{seen:?}");
    for (k, (&y, &o)) in seen.iter().zip(&oracle).enumerate() {
        assert!((y - o).abs() <= 2, "frame {k}: {y} against {o} ({seen:?})");
    }
}

/// The door in the hub to `level` (a placement whose LP +0x42 names it,
/// 0x8004f78c): Tommy put before it, Triangle; where the exit leads.
fn through_door(g: &mut Game, level: u8) -> Option<Level> {
    let spots: Vec<[i32; 3]> =
        g.set.placements.iter().filter(|p| p.kind == 1 && p.level == level as i16).map(|p| p.pos).collect();
    for at in spots {
        for (dx, dz) in [(0, -600), (0, 600), (-600, 0), (600, 0)] {
            let from = [at[0] + dx, at[1], at[2] + dz];
            let q = rugrats::world::collide::query(&g.col, [from[0], from[1] - 2000, from[2]]);
            if q.kind != 0 {
                continue;
            }
            put(g, [from[0], q.floor + g.player.feet(), from[2]], 0);
            face(g, at);
            let mut exit = None;
            run(g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 3)], |g, _| exit = exit.or(g.exit.take()));
            if let Some(e) = exit {
                return Some(e.level);
            }
        }
    }
    None
}

/// Level `level` opened as the game opens it (from the hub, with the
/// progress so far) and won by play.
fn win_level(disc: &GameDisc, level: Level, progress: &Progress) {
    if level.is_hole() || level == Level::GOLF_DOOR {
        win_golf(disc);
        return;
    }
    // The level's start cinematic (0x80074958) plays before it.
    if let Some(n) = rugrats::world::game::start_cine(level) {
        CINES.with(|c| c.borrow_mut().push((level, n)));
    }
    let mut g = Game::open_from(disc, level, Some(Level::HUB), progress).unwrap();
    match level.0 {
        2 => win_2(&mut g),
        3 => win_3(&mut g),
        4 => win_4(&mut g),
        5 => win_5(&mut g),
        6 => win_6(&mut g),
        7 => win_7(&mut g),
        9 => win_9(&mut g),
        10 => win_10(&mut g),
        11 => win_11(&mut g),
        12 => win_12(&mut g),
        13 => win_13(&mut g),
        15 => win_15(&mut g),
        16 => win_16(&mut g),
        17 => win_17(&mut g),
        18 => win_18(&mut g),
        20 => win_box_toss(&mut g, 64, 4500),
        21 => win_box_toss(&mut g, 50, 3300),
        n => panic!("no way to win level {n}"),
    }
}

/// The whole game from a new one, by play: each of the hub's doors walked
/// to and used (Triangle), its level won by playing it, the progress kept
/// (pieces, the count that opens the bonus levels, Mirrorland's doors),
/// every bonus level a win opens played at once, and with all twelve
/// pieces the board's Reptar 2010 to City Hall; then every cinematic it
/// came to played through to its end.
#[test]
fn the_whole_game_by_play() {
    let Some(disc) = disc() else { return };
    let mut progress = Progress::default();
    let mut rand = rugrats::rand::Rand::default();
    let mut played = Vec::new();
    // A new game's first entry to the hub: Reptar Puzzle (+0x30 bit 4)
    // gives the first piece (0x80094bfc(1)), as the app does then.
    progress.set_flags(progress.flags() | 4);
    assert!(progress.give_piece(Level::HUB, &mut rand));
    // Training, from the title's menu (activities aside): won by play
    // too, before the hub.
    win_level(&disc, Level::TRAINING, &progress);
    played.push(Level::TRAINING.0);
    // Doors 3, 4 and 6 open with Mirrorland won (+0x30 bit 16), which the
    // count opens after the third main level.
    for door in [2u8, 5, 9, 10, 11, 12, 25, 3, 4, 6, 13, 18] {
        let mut hub = Game::open_from(&disc, Level::HUB, None, &progress).unwrap();
        run(&mut hub, &[(&[], 3)], |_, _| {});
        assert_eq!(through_door(&mut hub, door), Some(Level(door)), "the door to {door}, after {played:?}");
        let mut next = Some(Level(door));
        while let Some(level) = next.filter(|&l| l != Level::HUB) {
            progress.enter(level);
            win_level(&disc, level, &progress);
            played.push(level.0);
            let record = if level == Level::GOLF_DOOR { Level::GOLF } else { level };
            next = progress.finish(record, true, &mut rand).next;
        }
    }
    assert_eq!(progress.pieces(), 12, "after {played:?}");
    win_level(&disc, Level::REPTAR_2010, &progress);
    for l in [2, 3, 4, 5, 6, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21] {
        assert!(played.contains(&l), "level {l} not played: {played:?}");
    }
    // Every cinematic the campaign came to, played whole as the app plays
    // it (Cine::tick at 60 ticks a second, nothing pressed): each shows
    // frames and reaches its end.
    let mut cines = CINES.with(|c| c.borrow().clone());
    cines.sort_by_key(|c| (c.0.0, c.1));
    cines.dedup();
    let audio = Default::default();
    let mut frames = 0;
    let count = cines.len();
    for (level, n) in cines {
        let mut cine = rugrats::media::cine::Cine::open(&disc, n, level.index(), 0, &audio).unwrap();
        let mut shown = 0;
        let mut now = 0;
        while !cine.over() && now < 60 * 600 {
            now += 1;
            shown += cine.tick(now) as usize;
        }
        assert!(cine.over() && shown > 0, "cine {n} of level {}: {shown} frames by tick {now}", level.0);
        frames += shown;
    }
    eprintln!("{count} cinematics played, {frames} frames");
}

/// The running jump with Cross held, frame by frame from the press: how
/// far along and how high (y down) the body is, by the port's air
/// (`Player`'s: the crouch frame's run at the body's gait, then
/// `moving_jump`'s launch, gravity
/// 14 less 5 while rising with Cross held, the drag of 32/4096), to 1500
/// under the takeoff. Tommy's matches the port's to the crouch's root keys
/// (54 higher at the first frame).
fn jump_shape(g: &Game) -> Vec<(i32, i32)> {
    const DT: i32 = 10;
    // The body's run speed (17, Spike's 20 in level 11).
    let run = g.player.gait.0;
    let (up, factor) = g.player.moving_jump;
    let (mut along, mut y) = (run * DT, 0);
    let mut out = vec![(along, y)];
    let (mut vh, mut vy) = ((factor * run) as f32, up);
    let mut first = true;
    // The stick held on the way: its push against the drag
    // (`Player::air_across`).
    let push = rugrats::actors::player::AIR_PUSH;
    while y < 1500 {
        let a = if !first && vy < 0 { 9 } else { 14 };
        first = false;
        let acc = push - vh * 32.0 / 4096.0;
        along += ((vh * DT as f32 + acc * (DT * DT) as f32 / 2.0) / 64.0) as i32;
        vh += acc * DT as f32;
        y += (vy * DT + a * DT * DT / 2) >> 6;
        vy = (vy + a * DT).min(4096);
        out.push((along, y));
    }
    out
}

/// The boxes the body meets (`Game::bodies`; before the first frame, the
/// objects').
fn bodies(g: &Game) -> &[rugrats::actors::player::Box3] {
    if g.bodies.is_empty() { &g.solid } else { &g.bodies }
}

/// Goes up to character `id` until within `reach` of it, as a player
/// does: walked to a place within reach of it on its floor, then steered at
/// it with the stick (it may move); twice. Whether it got there.
fn up_to(g: &mut Game, id: u16, reach: i64) -> bool {
    let pos = |g: &Game| g.npcs.iter().find(|n| n.id == id && !n.hidden).map(|n| n.pos);
    let d2 = |g: &Game, p: [i32; 3]| ((p[0] - g.player.pos[0]) as i64).pow(2) + ((p[2] - g.player.pos[2]) as i64).pow(2);
    for _ in 0..2 {
        let Some(at) = pos(g) else { return false };
        if d2(g, at) > reach * reach {
            let land = floor_under(g, at[0], at[2], at[1] - 1500, 0).unwrap_or(at[1] - g.player.feet());
            let r = (reach - 128).pow(2);
            let near = |s: Spot| {
                ((centre(s.0) - at[0]) as i64).pow(2) + ((centre(s.1) - at[2]) as i64).pow(2) < r && (s.2 - land).abs() < 300
            };
            if !(walking() && walk_to(g, near, None)) {
                put(g, beside(g, at, 600.0), 0);
            }
        }
        for _ in 0..90 {
            let Some(at) = pos(g) else { return false };
            if d2(g, at) < reach * reach {
                return true;
            }
            let keys = steer_at(g, at).to_vec();
            run(g, &[(&keys, 1)], |_, _| {});
        }
    }
    false
}

/// To about `at` (the leap that follows backs up to its own run-up): put
/// there facing `heading`, or walked to within two cells of it on its
/// floor.
fn near(g: &mut Game, at: [i32; 3], heading: i32) {
    if walking() {
        let feet = at[1] - g.player.feet();
        let land = floor_under(g, at[0], at[2], feet - 1, 0).unwrap_or(feet);
        // Straight there where the line is clear (the ring's floor).
        if line_clear(g, at, land) && stride_to(g, at[0], at[2], 600, 200) {
            return;
        }
        let key = (cell(at[0]), cell(at[2]));
        if walk_to(g, |s| (s.0 - key.0).abs() <= 2 && (s.1 - key.1).abs() <= 2 && (s.2 - land).abs() < 300, None) {
            return;
        }
    }
    put(g, at, heading);
}

/// Whether the straight line from the player to `at` keeps to his floor
/// `land` with nothing on it.
fn line_clear(g: &Game, at: [i32; 3], land: i32) -> bool {
    let me = g.player.pos;
    let here = me[1] - g.player.feet();
    let ground = Ground::new(g);
    let n = (((at[0] - me[0]) as f32).hypot((at[2] - me[2]) as f32) / 128.0) as i32 + 1;
    (here - land).abs() < 64
        && (0..=n).all(|k| {
            let (x, z) = (me[0] + (at[0] - me[0]) * k / n, me[2] + (at[2] - me[2]) * k / n);
            ground.floor(x, z, land - 384).is_some_and(|f| (f - land).abs() < 64) && !ground.blocked(x, z, land)
        })
}

/// The place `r` short of `at` on the player's side, on the floor there
/// (a character is gone up to from where the player is, not round him).
fn beside(g: &Game, at: [i32; 3], r: f64) -> [i32; 3] {
    let me = g.player.pos;
    let (dx, dz) = ((me[0] - at[0]) as f64, (me[2] - at[2]) as f64);
    let len = dx.hypot(dz).max(1.0);
    let (x, z) = (at[0] + (dx * r / len) as i32, at[2] + (dz * r / len) as i32);
    let y = floor_under(g, x, z, at[1] - 1500, 0).map_or(at[1], |f| f + g.player.feet());
    [x, y, z]
}

/// The floor under (x, z) from height `y` down: the COL's or the top of an
/// object or a thing at rest within `near` of it, the higher; None in a
/// wall.
fn floor_under(g: &Game, x: i32, z: i32, y: i32, near: i32) -> Option<i32> {
    let q = rugrats::world::collide::query(&g.col, [x, y, z]);
    if q.kind != 0 || q.tile.is_none() {
        return None;
    }
    // Doors (opened as the player comes to them) are no floor.
    let door = |b: &rugrats::actors::player::Box3| {
        g.doors.iter().map(|d| g.set.placements[d.placement].pos).any(|d| (b.pos.x - d[0] as f32).abs() < 2.0 && (b.pos.z - d[2] as f32).abs() < 2.0)
    };
    let top = bodies(g)
        .iter()
        .filter(|b| !b.character && b.covers([x, y, z], near) && b.min_y >= y && b.min_y < q.floor && !door(b))
        .map(|b| b.min_y)
        .min();
    Some(top.unwrap_or(q.floor))
}

/// A place on foot: its cell (256) and its floor.
type Spot = (i32, i32, i32);

/// How a place was reached from the one before: a step into the next cell
/// (a walk, a climb or a drop), a jump up onto the next cell's top, or a
/// running jump toward a heading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Way {
    Step,
    Hop,
    Jump(i32),
}

const FOOT: i32 = 256;

fn cell(v: i32) -> i32 {
    v.div_euclid(FOOT)
}

fn centre(c: i32) -> i32 {
    c * FOOT + FOOT / 2
}

/// The level's ground as the player can take it, by the player's rules: a
/// step up to 384, a climb up to the body's `climb_max` (521 carrying)
/// where the COL or a climbable object stands in the way, a jump onto the
/// next cell up to the moving jump's rise, any drop, and running jumps
/// (`jump_shape`) over gaps. Object tops are floors; objects in the way
/// are walls. Doors open as the player comes (hub_doors_...); movers are
/// where they are.
struct Ground<'a> {
    g: &'a Game,
    /// Moves (cell to cell) that failed in play.
    ban: std::collections::HashSet<[i32; 4]>,
    /// Characters stand in the way (a walk planned now); without, the
    /// level as it lies.
    characters: bool,
    /// A step's way checked by its middle probe alone (a way the side
    /// probes catch on is slid along in play, slower).
    narrow: bool,
    shape: Vec<(i32, i32)>,
    jump: i32,
    most: i32,
    /// The body's half width (the hierarchy box's x, `Player::ground_step`'s
    /// side probes): the COL must be open that far about a place.
    side: i32,
    /// The boxes the body meets (doors left out: opened as the player
    /// comes), by 1024 square: each in every square within 512 of its reach;
    /// those the walk starts inside (a cart stood in) left out too.
    boxes: Vec<rugrats::actors::player::Box3>,
    grid: std::collections::HashMap<(i32, i32), Vec<usize>>,
}

impl<'a> Ground<'a> {
    /// The boxes that may cover (x, z) within 512.
    fn near(&self, x: i32, z: i32) -> impl Iterator<Item = &rugrats::actors::player::Box3> {
        self.grid.get(&(x.div_euclid(1024), z.div_euclid(1024))).into_iter().flatten().map(|&k| &self.boxes[k])
    }

    /// The floor under (x, z) from `y` down: the COL's or the top of a box
    /// (not a character's) within `near` of it, the higher; None in a wall.
    fn floor_at(&self, x: i32, z: i32, y: i32, near: i32) -> Option<i32> {
        let q = rugrats::world::collide::query(&self.g.col, [x, y, z]);
        if q.kind != 0 || q.tile.is_none() {
            return None;
        }
        let top = self.near(x, z).filter(|b| !b.character && b.covers([x, y, z], near) && b.min_y >= y && b.min_y < q.floor).map(|b| b.min_y).min();
        Some(top.unwrap_or(q.floor))
    }

    fn new(g: &'a Game) -> Self {
        Self::with_shut(g, &[])
    }

    /// `new`, the doors at `shut` (used and still shut) walls.
    fn with_shut(g: &'a Game, shut: &[[i32; 3]]) -> Self {
        // Carrying (not a kept thing), no jump and climbs under 522
        // (0x80058da4).
        let shape = jump_shape(g);
        let carrying = g.player.carrying && g.player.carry_climbs;
        let jump = if carrying { 0 } else { -shape.iter().map(|s| s.1).min().unwrap_or(0) };
        // Level 10 refuses type 14 (906 up and more, 0x80056e44).
        let most = if carrying { 521.min(g.player.climb_max) } else if g.level == Level::CHUCKIES_GLASSES { 905 } else { g.player.climb_max };
        // The doors' places (their boxes carry no template).
        // Only a shut door at rest is passed (opened on the way); an open
        // or swinging one stands where it is.
        let doors: Vec<[i32; 3]> = g
            .doors
            .iter()
            .filter(|d| !d.open && !d.moving)
            .map(|d| g.set.placements[d.placement].pos)
            .filter(|p| !shut.contains(p))
            .collect();
        // Level 5's sliding doors (template 843) open as she comes within
        // 5000 of them.
        let sliding = |b: &rugrats::actors::player::Box3| g.level == Level::OUTER_SPACE && b.template == 843;
        let boxes: Vec<_> = bodies(g)
            .iter()
            .filter(|b| !sliding(b))
            .filter(|b| !doors.iter().any(|d| (b.pos.x - d[0] as f32).abs() < 2.0 && (b.pos.z - d[2] as f32).abs() < 2.0))
            .cloned()
            .collect();
        let me = g.player.pos;
        let body = [me[0], me[1] - g.player.feet() - 300, me[2]];
        let boxes: Vec<_> = boxes.into_iter().filter(|b| !b.covers(body, 0)).collect();
        let mut grid: std::collections::HashMap<(i32, i32), Vec<usize>> = Default::default();
        for (k, b) in boxes.iter().enumerate() {
            let r = b.lo.iter().chain(b.hi.iter()).map(|v| v.abs()).max().unwrap_or(0) + 512;
            let (x, z) = (b.pos.x as i32, b.pos.z as i32);
            for gx in (x - r).div_euclid(1024)..=(x + r).div_euclid(1024) {
                for gz in (z - r).div_euclid(1024)..=(z + r).div_euclid(1024) {
                    grid.entry((gx, gz)).or_default().push(k);
                }
            }
        }
        let h = &g.player.hierarchy.header;
        let half = |at: usize| h.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32).abs();
        let side = half(0x04).max(half(0x0c));
        Ground { g, ban: Default::default(), characters: true, narrow: false, shape, jump, most, side, boxes, grid }
    }

    /// The floor under (x, z) from `y` down. Objects within half a cell
    /// count: a narrow one (level 11's stands) between the cells' middles
    /// is stood on too.
    fn floor(&self, x: i32, z: i32, y: i32) -> Option<i32> {
        self.floor_at(x, z, y, FOOT / 2)
    }

    /// An object (not a door) where the body would be, standing on `floor`:
    /// within its radius (`player::RADIUS`, and 32 for the stick's eight
    /// ways) at its middle's height, above a step and below the head.
    fn blocked(&self, x: i32, z: i32, floor: i32) -> bool {
        // The characters as they stand now (the plan is made again as they
        // move) and the things at rest, as the body meets them.
        self.near(x, z)
            .filter(|b| self.characters || !b.character)
            .any(|b| b.covers([x, floor - 300, z], rugrats::actors::player::RADIUS + 32) && b.min_y < floor - 384 && b.max_y > floor - 600)
    }

    /// Whether a step from cell (cx, cz) on `floor` along (dx, dz) has its
    /// probes clear of things (`Player::probes`: the body's depth and the
    /// step ahead, and at its half-width each side, at each thing's own
    /// middle height, 0x80063d3c); things low enough to step onto aside.
    fn probes_clear(&self, (cx, cz): (i32, i32), floor: i32, dir: [f32; 2]) -> bool {
        self.probes_clear_by((cx, cz), floor, dir, FOOT as f32)
    }

    /// `probes_clear` for a step of `step` on.
    fn probes_clear_by(&self, (cx, cz): (i32, i32), floor: i32, dir: [f32; 2], step: f32) -> bool {
        let p = &self.g.player;
        let (x, z) = (centre(cx), centre(cz));
        let at = [x, floor + p.feet(), z];
        // (All three, as the ground move's: a way the side ones meet a
        // thing on holds the body there; narrow, the middle one.)
        p.probes(dir, step).iter().take(if self.narrow { 1 } else { 3 }).all(|&(px, pz)| {
            self.near(x, z).chain(self.near(x + px, z + pz)).filter(|b| !b.character && b.min_y < floor - 384).all(|b| {
                let h = p.probe_height(b, at);
                b.segment_hit([x, h, z], [x + px, h, z + pz]).is_none()
            })
        })
    }

    /// The places one step on and how (a step or a hop), and whether some
    /// way on is not a plain step (an edge, where a jump is worth trying).
    fn steps(&self, (cx, cz, floor): Spot) -> (Vec<(Spot, Way)>, bool) {
        let mut out = Vec::new();
        let mut edge = false;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (x, z) = (centre(cx + dx), centre(cz + dz));
            // Room for the body: the COL open its half width about it, and
            // half of that (a wall thinner than the body is between).
            let (w, h) = (self.side, self.side / 2);
            let room = |f: i32| {
                [(0, 0), (w, 0), (-w, 0), (0, w), (0, -w), (h, 0), (-h, 0), (0, h), (0, -h)]
                    .iter()
                    .all(|&(ox, oz)| {
                        // Open, or a ledge low enough to climb (the probe
                        // leaves that to the climb).
                        let col = &self.g.col;
                        rugrats::world::collide::query(col, [x + ox, f - 384, z + oz]).kind == 0 || {
                            let up = rugrats::world::collide::query(col, [x + ox, f - self.most, z + oz]);
                            up.kind == 0 && up.floor >= f - self.most
                        }
                    })
            };
            // A thin box between the two middles (a board, a rail) is crossed
            // by the step (`Box3::crosses`, 0x80069c40).
            let from = [centre(cx), floor, centre(cz)];
            let across = |f: i32| {
                // (Not one the step starts inside: the way out of it.)
                self.near(x, z).filter(|b| self.characters || !b.character).any(|b| {
                    b.crosses(from, [x, f, z]) && !b.covers(from, 0) && b.min_y < f.min(floor) - 384 && b.max_y > f.min(floor) - 600
                })
            };
            match self.floor(x, z, floor - 384) {
                Some(f) if !self.blocked(x, z, f.min(floor)) && room(f.min(floor)) && !across(f) && self.probes_clear((cx, cz), floor, [dx as f32, dz as f32]) => {
                    edge |= f > floor + 384;
                    out.push(((cx + dx, cz + dz, f), Way::Step));
                }
                _ => {
                    edge = true;
                    // Over it, onto a top higher than a step with room on
                    // it: a jump onto any, a climb onto the COL's or a
                    // climbable object's (template flag 64, 0x80058da4).
                    // The climb first: no jump needed where it reaches.
                    for (reach, way) in [(self.most, Way::Step), (self.jump, Way::Hop)] {
                        let Some(f) = self.floor(x, z, floor - reach - 1) else { continue };
                        let on = self.g.solid.iter().find(|b| b.min_y == f && b.covers([x, f, z], FOOT / 2));
                        let climbable = way == Way::Hop || on.is_none_or(|b| b.climb);
                        if f >= floor - reach && f < floor - 384 && !self.blocked(x, z, f) && climbable {
                            out.push(((cx + dx, cz + dz, f), way));
                            break;
                        }
                    }
                }
            }
        }
        (out, edge)
    }

    /// Where a running jump (state 41, Cross held) from (x0, z0) on `floor`
    /// toward heading `h` comes down: along `jump_shape`, the body's top
    /// (500 up) in the open and nothing in its way; the feet over a floor
    /// (in a wall the port puts the body back over the floor it came from,
    /// 0x8005f1b4), landing when the next frame would reach it. It needs a
    /// run-up of two cells behind.
    fn leap(&self, x0: i32, z0: i32, floor: i32, h: i32) -> Option<Spot> {
        if self.jump == 0 {
            return None;
        }
        let a = h as f32 * std::f32::consts::TAU / 4096.0;
        let (sx, sz) = (a.sin(), a.cos());
        let (x0, z0) = (x0 as f32, z0 as f32);
        let run_up = [256.0, 512.0].iter().all(|&n| {
            let (x, z) = ((x0 - sx * n) as i32, (z0 - sz * n) as i32);
            self.floor(x, z, floor - 384).is_some_and(|f| (f - floor).abs() < 64) && !self.blocked(x, z, floor)
        });
        if !run_up {
            return None;
        }
        for (i, &(along, dy)) in self.shape.iter().enumerate() {
            let (x, z) = ((x0 + sx * along as f32) as i32, (z0 + sz * along as f32) as i32);
            let y = floor + dy;
            // In the air an object stops the body as on the ground: its top
            // more than a step over the feet, its bottom under the head.
            let open = |h: i32| self.floor_at(x, z, h, 0).is_some_and(|f| f >= h);
            if !open(y - 500) || self.blocked(x, z, y) {
                return None;
            }
            let f = self.floor(x, z, y - 1)?;
            let next = self.shape.get(i + 1).map_or(i32::MAX, |n| n.1 - dy);
            let falling = self.shape.get(i + 1).is_some_and(|n| n.1 > dy);
            if falling && f - y <= next {
                return Some((cell(x), cell(z), f));
            }
        }
        None
    }

    /// The floor the body at `pos` stands on.
    fn spot(&self, pos: [i32; 3]) -> Spot {
        let feet = pos[1] - self.g.player.feet();
        (cell(pos[0]), cell(pos[2]), self.floor(pos[0], pos[2], feet - 1).unwrap_or(feet))
    }

    /// Every place reached from `from`, cheapest first (a step costs 1, a
    /// jump 6 and its cells), with the way into each, until `goal` holds
    /// for one (returned).
    fn ways(&self, from: [i32; 3], goal: impl Fn(Spot) -> bool) -> (Places, Option<Spot>) {
        use std::cmp::Reverse;
        let start = self.spot(from);
        let mut places = Places::default();
        let mut todo = std::collections::BinaryHeap::from([(Reverse(0), start, start, Way::Step)]);
        while let Some((Reverse(cost), here, prev, way)) = todo.pop() {
            let (cx, cz, floor) = here;
            let seen = places.floors.entry((cx, cz)).or_default();
            if seen.iter().any(|&f| (f - floor).abs() < 64) || seen.len() > 8 {
                continue;
            }
            seen.push(floor);
            places.came.insert(here, (prev, way));
            if goal(here) {
                return (places, Some(here));
            }
            // A move, or a place the body was held out of from any side.
            let banned = |n: &Spot| self.ban.contains(&[cx, cz, n.0, n.1]) || self.ban.contains(&[i32::MIN, i32::MIN, n.0, n.1]);
            let (next, _) = self.steps(here);
            for (n, way) in next.into_iter().filter(|n| !banned(&n.0)) {
                todo.push((Reverse(cost + if way == Way::Hop { 10 } else { 1 }), n, here, way));
            }
            // A running jump where something is in the way within 1024
            // ahead (a drop, a rise, a wall or an object), the takeoff
            // here or a few cells back.
            for d in 0..8 {
                let a = (d * 512) as f32 * std::f32::consts::TAU / 4096.0;
                let ahead = [256.0, 512.0, 768.0, 1024.0].iter().any(|&t| {
                    let (x, z) = (centre(cx) + (a.sin() * t) as i32, centre(cz) + (a.cos() * t) as i32);
                    !self.floor(x, z, floor - 384).is_some_and(|f| (f - floor).abs() < 64) || self.blocked(x, z, floor)
                }) || !self.probes_clear_by((cx, cz), floor, [a.sin(), a.cos()], 1024.0);
                if !ahead {
                    continue;
                }
                if let Some(n) = self.leap(centre(cx), centre(cz), floor, d * 512).filter(|n| (n.0, n.1) != (cx, cz) && !banned(n)) {
                    let far = (n.0 - cx).abs().max((n.1 - cz).abs());
                    todo.push((Reverse(cost + 6 + far), n, here, Way::Jump(d * 512)));
                }
            }
        }
        (places, None)
    }

    /// The ways from the start to `to`.
    fn path(places: &Places, to: Spot) -> Vec<(Spot, Way)> {
        let mut path = Vec::new();
        let mut at = to;
        while let Some(&(prev, way)) = places.came.get(&at) {
            if prev == at {
                break;
            }
            path.push((at, way));
            at = prev;
        }
        path.reverse();
        path
    }
}

/// What a flood reached: the floors by cell, and the way into each place.
#[derive(Default)]
struct Places {
    floors: std::collections::HashMap<(i32, i32), Vec<i32>>,
    came: std::collections::HashMap<Spot, (Spot, Way)>,
}

/// One frame of the stick toward (x, z): running, or walking (Square) when
/// `slow`.
fn stride(g: &mut Game, x: i32, z: i32, slow: bool) {
    let me = g.player.pos;
    let h = (((x - me[0]) as f32).atan2((z - me[2]) as f32) * 4096.0 / std::f32::consts::TAU) as i32;
    // The analog stick straight at it (a DualShock's, as the original
    // reads it), Square to walk.
    let stick = stick_toward(g, h);
    let keys: &[usize] = if slow { &[input::SQUARE] } else { &[] };
    run_stick(g, stick, keys, 1);
    // Well off it: let go while the heading turns to the way's (at its
    // rate; held on, the body runs a wide circle), then on.
    let off = |a: i32| (a + 2048).rem_euclid(4096) - 2048;
    if off(h - g.player.heading).abs() > 600 && !g.player.frozen {
        for _ in 0..40 {
            if g.player.heading == g.player.target {
                break;
            }
            run(g, &[(&[], 1)], |_, _| {});
        }
    }
}

/// One frame of the stick along the line from `from` to `to`: at the place
/// on it 600 on from the body's nearest (a narrow ramp is kept to; aiming
/// at the end alone, with the stick's eight ways, drifts off it).
fn stride_along(g: &mut Game, from: [i32; 2], to: [i32; 2]) {
    let me = g.player.pos;
    let (lx, lz) = ((to[0] - from[0]) as f32, (to[1] - from[1]) as f32);
    let len = lx.hypot(lz);
    if len < 1.0 {
        return stride(g, to[0], to[1], false);
    }
    let along = (lx * (me[0] - from[0]) as f32 + lz * (me[2] - from[1]) as f32) / len;
    let t = ((along + 600.0) / len).min(1.0);
    stride(g, from[0] + (lx * t) as i32, from[1] + (lz * t) as i32, false);
}

/// Walks with the stick to within `near` of (x, z) in `frames` at most,
/// slowing to a walk for the last 400.
fn stride_to(g: &mut Game, x: i32, z: i32, near: i32, frames: usize) -> bool {
    for _ in 0..frames {
        let me = g.player.pos;
        let d2 = ((x - me[0]) as i64).pow(2) + ((z - me[2]) as i64).pow(2);
        if d2 <= (near as i64).pow(2) {
            return true;
        }

        stride(g, x, z, d2 < 400 * 400);
    }
    false
}

/// A running jump toward `h` taking off at (tx, tz): back 512 from it, the
/// stick turned to `h` (`turn_to`), the camera put behind (R1), the way
/// ahead held, and Cross held from the takeoff until the body is down.
fn leap_by_play(g: &mut Game, tx: i32, tz: i32, h: i32) -> bool {
    leap_within(g, tx, tz, h, 48)
}

/// `leap_by_play`, the turn left as it is within `near` of `h`.
fn leap_within(g: &mut Game, tx: i32, tz: i32, h: i32, near: i32) -> bool {
    let a = h as f32 * std::f32::consts::TAU / 4096.0;
    let back = [(tx as f32 - a.sin() * 512.0) as i32, (tz as f32 - a.cos() * 512.0) as i32];
    stride_to(g, back[0], back[1], 64, 60);
    run(g, &[(&[], 4)], |_, _| {});
    turn_near(g, h, near);
    if g.fixed.is_none() {
        run(g, &[(&[input::R1], 1), (&[], 2)], |_, _| {});
    }
    let ahead = g.player.heading;
    let b = ahead as f32 * std::f32::consts::TAU / 4096.0;
    let mut jumped = false;
    let mut last = g.player.pos;
    for _ in 0..80 {
        let p = &g.player;
        // Down (Cross let go as it lands: held on, it jumps again).
        if jumped && p.state != State::Jump && matches!(p.state, State::Stand | State::Walk | State::Run | State::Land) && p.v[1] == 0 {
            run(g, &[(&[], 6)], |_, _| {});
            return true;
        }
        let past = (p.pos[0] - tx) as f32 * b.sin() + (p.pos[2] - tz) as f32 * b.cos();
        // Held up short of the takeoff (a thing the step's probes meet,
        // up to the body's depth and a step ahead): over it from there.
        let held = p.state == State::Run && (p.pos[0] - last[0]).abs() + (p.pos[2] - last[2]).abs() < 8 && past > -600.0;
        if held && !jumped && std::env::var_os("WALKLOG").is_some() {
            eprintln!("  held at {:?}, {past:.0} short of the takeoff ({tx}, {tz})", p.pos);
        }
        last = p.pos;
        if std::env::var_os("LEAPLOG").is_some() {
            eprintln!("    leap {:?} {:?} past {past:.0} v {:?} floor {}", p.pos, p.state, p.v, p.floor);
        }
        let stick = stick_toward(g, ahead);
        // (Within half a run's step of it: a takeoff at a shelf's edge is
        // otherwise run off.)
        if jumped || past >= -85.0 || held {
            jumped = true;
        }
        run_stick(g, stick, if jumped { &[input::CROSS] } else { &[] }, 1);
    }
    false
}

/// Walks by play to a place where `goal` holds (`Ground`'s way, followed
/// with the stick and Cross, planned again from wherever the player gets
/// to), then to within 64 of `end`. True when there.
fn walk_to(g: &mut Game, goal: impl Fn(Spot) -> bool + Copy, end: Option<[i32; 2]>) -> bool {
    let log = std::env::var_os("WALKLOG").is_some();
    let mut ban = std::collections::HashSet::new();
    // Doors used once on the way; those that stayed shut are walls.
    let mut used: Vec<[i32; 3]> = Vec::new();
    let mut hopped = 0;
    let mut unbanned = false;
    let mut downs: Vec<(i32, i32)> = Vec::new();
    // (Level 4's way to Cynthia's farthest spot is some 800 places and a
    // dozen jumps, each landing a little off it to be planned on from.)
    for _ in 0..80 {
        let shut: Vec<[i32; 3]> = used.iter().copied().filter(|p| g.doors.iter().any(|d| g.set.placements[d.placement].pos == *p && !d.open && !d.moving)).collect();
        let mut ground = Ground::with_shut(g, &shut);
        ground.ban = ban.clone();
        let start = ground.spot(g.player.pos);
        let (mut places, mut found) = ground.ways(g.player.pos, goal);
        // No way the body passes cleanly: one it slides through.
        if found.is_none() {
            ground.narrow = true;
            (places, found) = ground.ways(g.player.pos, goal);
        }
        // Characters across every way (level 2's geese in the maze): past
        // them, as a player goes.
        if found.is_none() {
            ground.characters = false;
            (places, found) = ground.ways(g.player.pos, goal);
        }
        let Some(to) = found else {
            if log {
                eprintln!("no way from {:?} (spot {start:?}, steps {:?}, ban {ban:?})", g.player.pos, ground.steps(start));
            }
            // Every way banned by failures: things move (level 11's carts),
            // so once, all of them tried again.
            if !ban.is_empty() && !unbanned {
                unbanned = true;
                ban.clear();
                continue;
            }
            // Up on something the plan has no way off (a rail come down on
            // after a jump), or in a corner it allows the body no room in
            // (it slides along walls): off it ahead, back, to either side.
            if hopped < 6 {
                let turn = [0, 2048, 1024, -1024, 0, 2048][hopped];
                hopped += 1;
                let ahead = toward(g, (g.player.heading + turn).rem_euclid(4096)).to_vec();
                run(g, &[(&ahead, 12), (&[], 6)], |_, _| {});
                continue;
            }
            return false;
        };
        let path = Ground::path(&places, to);
        if path.is_empty() {
            // To the spot's own place on that floor first (a straight line
            // to it may run off a shelf's edge), then the spot itself, or
            // as near as a thing there lets him.
            if let Some([x, z]) = end {
                let me = g.player.pos;
                let key = (cell(x), cell(z));
                if ((x - me[0]) as i64).pow(2) + ((z - me[2]) as i64).pow(2) > 256 * 256 {
                    let mut again = Ground::with_shut(g, &shut);
                    again.ban = ban.clone();
                    let (places, found) = again.ways(me, |s| (s.0, s.1) == key && (s.2 - to.2).abs() < 300);
                    if let Some(t) = found {
                        let path = Ground::path(&places, t);
                        if !path.is_empty() {
                            follow(g, start, &path, &mut used);
                        }
                    }
                }
            }
            let from = g.player.pos;
            // (A thing there holds the body its probes' reach off: its depth
            // and a step, 0x80063d3c.)
            let there = end.is_none_or(|[x, z]| stride_to(g, x, z, 64, 90) || {
                let me = g.player.pos;
                ((x - me[0]) as i64).pow(2) + ((z - me[2]) as i64).pow(2) < 512 * 512
            });
            if log && !there {
                eprintln!("short of {end:?}: from {from:?} to {:?} ({:?})", g.player.pos, g.player.state);
            }
            return there;
        }
        if log {
            eprintln!("t{} from {:?}: {} ways to {:?}, jumps {:?}", g.frames, g.player.pos, path.len(), (centre(to.0), centre(to.1), to.2), path.iter().filter(|p| p.1 != Way::Step).collect::<Vec<_>>());
        }
        if std::env::var_os("WALKPATH").is_some() {
            let cells: Vec<_> = path.iter().step_by(25).map(|p| (centre(p.0.0), centre(p.0.1), p.0.2)).collect();
            eprintln!("  path {cells:?}");
        }
        match follow(g, start, &path, &mut used) {
            Some(failed) => {
                ban.insert(failed);
            }
            // Down off the way again where it was last time: its first
            // move is no way from here.
            None => {
                let here = (cell(g.player.pos[0]), cell(g.player.pos[2]));
                if downs.contains(&here) {
                    ban.insert([start.0, start.1, path[0].0.0, path[0].0.1]);
                }
                downs.push(here);
            }
        }
    }
    false
}

/// Follows `path` from `start` until it is done or the player is off it:
/// steps aimed two places on (counting off those passed), jumps from their
/// takeoff. The move that failed (cell to cell), if one did.
fn follow(g: &mut Game, start: Spot, path: &[(Spot, Way)], used: &mut Vec<[i32; 3]>) -> Option<[i32; 4]> {
    let mut i = 0;
    let mut stuck = 0;
    // Caught on a corner: back to the middle of its cell, then straight at
    // the next place for a while.
    let mut was = g.player.pos;
    let mut moved = was;
    let mut close = 0;
    // The aim and where it was taken.
    let mut line: Option<(Spot, [i32; 2])> = None;
    // Frames on the ground since a place on the way was passed.
    let mut stalled = 0;
    let mut last_frame = g.frames;
    // A budget: 40 frames a place on the way and 300 more.
    let mut budget = 40 * path.len() + 300;
    while i < path.len() {
        budget = budget.saturating_sub(1);
        if budget == 0 {
            if std::env::var_os("WALKLOG").is_some() {
                eprintln!("  out of time at {:?} ({:?} {:?} v {:?}) going to {:?}", g.player.pos, g.player.state, g.player.action, g.player.v, path[i]);
                if g.player.state == State::Fall {
                    run(g, &[(&[], 30)], |_, _| {});
                    eprintln!("  let go 30: {:?} {:?} v {:?} heading {} speed {}", g.player.pos, g.player.state, g.player.v, g.player.heading, g.player.speed);
                }
            }
            let from = if i == 0 { start } else { path[i - 1].0 };
            return Some([from.0, from.1, path[i].0.0, path[i].0.1]);
        }
        if stuck > 0 && stuck % 10 == 0 {
            let me = g.player.pos;
            if (me[0] - was[0]).abs() + (me[2] - was[2]).abs() < 40 {
                stride_to(g, centre(cell(me[0])), centre(cell(me[2])), 40, 10);
                close = 20;
            }
            was = g.player.pos;
        }
        let here = Ground::new(g).spot(g.player.pos);
        let at = |s: &Spot| (s.0, s.1) == (here.0, here.1) && (here.2 - s.2).abs() < 300;
        let ahead = path[i..].iter().take(8).take_while(|(_, w)| *w == Way::Step).count();
        if let Some(j) = (0..ahead).rev().find(|&j| at(&path[i + j].0)) {
            i += j + 1;
            stalled = 0;
            continue;
        }
        if let (spot, Way::Hop) = path[i] {
            // Up onto the next cell's top: the takeoff short of it from which
            // the running jump (`Ground::leap`) comes down on it, if one is
            // found; else run at it, Cross held from 480 short of its middle
            // (or once its side stops the body) until down on it.
            let off = if i == 0 { start } else { path[i - 1].0 };
            let h = ((((spot.0 - off.0) as f32).atan2((spot.1 - off.1) as f32) * 4096.0 / std::f32::consts::TAU) as i32).rem_euclid(4096);
            let a = h as f32 * std::f32::consts::TAU / 4096.0;
            let ground = Ground::new(g);
            let takeoff = (0..90)
                .map(|k| 64.0 + 32.0 * k as f32)
                .map(|t| ((centre(spot.0) as f32 - a.sin() * t) as i32, (centre(spot.1) as f32 - a.cos() * t) as i32))
                .find(|&(x, z)| {
                    ground.floor(x, z, off.2 - 384).is_some_and(|f| (f - off.2).abs() < 64)
                        && ground.leap(x, z, off.2, h).is_some_and(|l| (l.0, l.1) == (spot.0, spot.1))
                });
            if let Some((x, z)) = takeoff {
                leap_by_play(g, x, z, h);
                let here = Ground::new(g).spot(g.player.pos);
                if (here.0, here.1) != (spot.0, spot.1) || (here.2 - spot.2).abs() >= 300 {
                    return Some([off.0, off.1, spot.0, spot.1]);
                }
                i += 1;
                continue;
            }
            let mut jumped = false;
            let mut last = g.player.pos;
            for _ in 0..60 {
                let me = g.player.pos;
                let near = ((centre(spot.0) - me[0]) as i64).pow(2) + ((centre(spot.1) - me[2]) as i64).pow(2) < 480 * 480
                    || (me[0] - last[0]).abs() + (me[2] - last[2]).abs() < 8;
                last = me;
                if jumped && matches!(g.player.state, State::Stand | State::Walk | State::Run) {
                    break;
                }
                let h = (((centre(spot.0) - me[0]) as f32).atan2((centre(spot.1) - me[2]) as f32) * 4096.0 / std::f32::consts::TAU) as i32;
                let mut keys = toward(g, h.rem_euclid(4096)).to_vec();
                if near || jumped {
                    jumped = true;
                    keys.push(input::CROSS);
                }
                run(g, &[(&keys, 1)], |_, _| {});
            }
            let here = Ground::new(g).spot(g.player.pos);
            if (here.0, here.1) != (spot.0, spot.1) || (here.2 - spot.2).abs() >= 300 {
                let from = if i == 0 { start } else { path[i - 1].0 };
                return Some([from.0, from.1, spot.0, spot.1]);
            }
            i += 1;
            continue;
        }
        if let (spot, Way::Jump(h)) = path[i] {
            let off = if i == 0 { start } else { path[i - 1].0 };
            if !leap_by_play(g, centre(off.0), centre(off.1), h) {
                return Some([off.0, off.1, spot.0, spot.1]);
            }
            let here = Ground::new(g).spot(g.player.pos);
            if (here.0, here.1) != (spot.0, spot.1) || (here.2 - spot.2).abs() >= 300 {
                if std::env::var_os("WALKLOG").is_some() {
                    eprintln!("  jump {h} from {:?} to {:?} came down at {:?}", (centre(off.0), centre(off.1)), (centre(spot.0), centre(spot.1), spot.2), g.player.pos);
                }
                return Some([off.0, off.1, spot.0, spot.1]);
            }
            i += 1;
            continue;
        }
        // Down off the way (off a ramp's side) and on the ground below it:
        // a new plan from here.
        let last = if i == 0 { start } else { path[i - 1].0 };
        if matches!(g.player.state, State::Stand | State::Walk | State::Run)
            && here.2 - last.2 >= 300
            && here.2 - path[i].0.2 >= 300
        {
            if std::env::var_os("WALKLOG").is_some() {
                eprintln!("  t{} down off the way at {:?} going to {:?}", g.frames, g.player.pos, path[i]);
            }
            return None;
        }
        // Stuck: no way on for 45 frames, not climbing nor in the air.
        let me = g.player.pos;
        let busy = g.player.action.is_some() || !matches!(g.player.state, State::Stand | State::Walk | State::Run);
        if busy || (me[0] - moved[0]).abs() + (me[2] - moved[2]).abs() + (me[1] - moved[1]).abs() > 96 {
            moved = me;
            stuck = 0;
        }
        stuck += 1;
        // Back and forth by a wall (each way more than the 96 above), in
        // frames (a step may take several, the turn waited out).
        let spent = g.frames - last_frame;
        last_frame = g.frames;
        if busy {
            stalled = 0;
        } else {
            stalled += spent;
        }
        if stalled > 300 {
            stuck = 46;
        }
        // A shut door on the way just ahead (or one that holds the body
        // up): hinged and sliding ones open with a use (Triangle, 0x8004f4a0
        // / 0x8004f6d8), facing it, before going on through.
        {
            let me = g.player.pos;
            let near = |p: [i32; 3], q: [i32; 2], r: i64| ((p[0] - q[0]) as i64).pow(2) + ((p[2] - q[1]) as i64).pow(2) < r * r;
            let soon: Vec<[i32; 2]> = path[i..(i + 4).min(path.len())].iter().map(|s| [centre(s.0.0), centre(s.0.1)]).collect();
            // Its box in `solid` (by its place; the hinge, not its middle).
            let shut = g.doors.iter().filter(|d| !d.open && !d.moving).map(|d| g.set.placements[d.placement].pos).filter(|p| !used.contains(p)).find(|&p| {
                let body = g.solid.iter().find(|b| (b.pos.x - p[0] as f32).abs() < 2.0 && (b.pos.z - p[2] as f32).abs() < 2.0);
                let on_way = body.is_some_and(|b| {
                    soon.iter().any(|&c| b.covers([c[0], b.max_y - 10, c[1]], 400)) && b.covers([me[0], b.max_y - 10, me[2]], 900)
                });
                let _ = on_way;
                stuck == 12 && near(p, [me[0], me[2]], 1500)
            });
            if let Some(p) = shut {
                // Up to it (a use needs it touched, +0xf4), its middle
                // faced, then Triangle.
                used.push(p);
                let mid = g
                    .solid
                    .iter()
                    .find(|b| (b.pos.x - p[0] as f32).abs() < 2.0 && (b.pos.z - p[2] as f32).abs() < 2.0)
                    .map(|b| {
                        let c = b.pos + b.rot * rrt::glam::Vec3::new((b.lo[0] + b.hi[0]) as f32 / 2.0, 0.0, (b.lo[2] + b.hi[2]) as f32 / 2.0);
                        [c.x as i32, p[1], c.z as i32]
                    })
                    .unwrap_or(p);
                stride_to(g, mid[0], mid[2], 200, 25);
                face(g, mid);
                run(g, &[(&[input::TRIANGLE], 1), (&[], 8)], |_, _| {});
            }
        }
        if stuck == 30 && std::env::var_os("WALKLOG").is_some() {
            let me = g.player.pos;
            for d in &g.doors {
                let p = g.set.placements[d.placement].pos;
                if ((p[0] - me[0]) as i64).pow(2) + ((p[2] - me[2]) as i64).pow(2) < 4000 * 4000 {
                    let b = g.solid.iter().find(|b| (b.pos.x - p[0] as f32).abs() < 2.0 && (b.pos.z - p[2] as f32).abs() < 2.0);
                    eprintln!("    door {} at {p:?} open {} moving {} box {:?}", d.template, d.open, d.moving, b.map(|b| (b.lo, b.hi, b.rot)));
                }
            }
            let near: Vec<_> = g.bodies.iter().filter(|b| b.covers([me[0], me[1], me[2]], 300)).map(|b| (b.template, b.character, b.pos, b.min_y, b.max_y, b.lo, b.hi, b.rot)).collect();
            eprintln!("    held at {me:?} by {near:?}");
        }
        if stuck > 45 {
            if std::env::var_os("WALKLOG").is_some() {
                eprintln!("  t{} stuck at {:?} going to {:?}", g.frames, g.player.pos, path[i]);
            }
            // Held out of a place on the same floor, next to it: the place
            // is too narrow for the body, whichever side it comes from.
            let to = path[i].0;
            if path[i].1 == Way::Step && (to.2 - here.2).abs() < 300 && (to.0 - here.0).abs() + (to.1 - here.1).abs() == 1 {
                return Some([i32::MIN, i32::MIN, to.0, to.1]);
            }
            let from = if i == 0 { start } else { path[i - 1].0 };
            return Some([from.0, from.1, path[i].0.0, path[i].0.1]);
        }
        // The farthest place on (up to 24, no jump between) whose straight
        // line from here keeps to the way's own cells: a straight run.
        let me = g.player.pos;
        let cells: std::collections::HashSet<(i32, i32)> =
            path[i.saturating_sub(1)..(i + ahead).min(path.len())].iter().map(|p| (p.0.0, p.0.1)).chain([(here.0, here.1)]).collect();
        let straight = |to: &Spot| {
            let (dx, dz) = ((centre(to.0) - me[0]) as f32, (centre(to.1) - me[2]) as f32);
            let n = (dx.abs().max(dz.abs()) / 96.0).ceil().max(1.0) as i32;
            (0..=n).all(|k| {
                let t = k as f32 / n as f32;
                cells.contains(&(cell(me[0] + (dx * t) as i32), cell(me[2] + (dz * t) as i32)))
            })
        };
        let far = if close > 0 { 0 } else { (0..ahead.min(24)).rev().find(|&j| straight(&path[i + j].0)).unwrap_or(0) };
        if std::env::var_os("WALKTRACE").is_some() {
            eprintln!("    t{} at {:?} {:?} i {i} aim {:?}", g.frames, me, g.player.state, (centre(path[i + far].0.0), centre(path[i + far].0.1), path[i + far].0.2));
        }
        close -= 1;
        let (goal, _) = path[i + far];
        let before = g.player.pos;
        // Along the line from where this aim was taken (the straight run
        // checked from there).
        if line.is_none_or(|(to, _)| to != goal) {
            line = Some((goal, [before[0], before[2]]));
        }
        let from = line.map_or([before[0], before[2]], |l| l.1);
        // Up or down a slope (level 6's narrow ramp): turned to the way's
        // heading (from the last place to the aim), the camera put behind
        // (R1), so the stick's Up runs it exactly; then along the line.
        let slope = (goal.2 - here.2).abs() > 128;
        if slope && close <= 0 && stuck <= 3 {
            let off = |a: i32| (a + 2048).rem_euclid(4096) - 2048;
            let last = if i == 0 { start } else { path[i - 1].0 };
            let h = ((((centre(goal.0) - centre(last.0)) as f32).atan2((centre(goal.1) - centre(last.1)) as f32) * 4096.0 / std::f32::consts::TAU) as i32).rem_euclid(4096);
            if off(h - g.player.heading).abs() > 48 && matches!(g.player.state, State::Stand | State::Walk | State::Run) {
                turn_to(g, h);
                if g.fixed.is_none() {
                    run(g, &[(&[input::R1], 1), (&[], 2)], |_, _| {});
                }
                line = Some((goal, [g.player.pos[0], g.player.pos[2]]));
            }
        }
        // A run into a thing bumps (state 9, some 14 frames held): by
        // things, within a run's probe of one, walked (Square).
        let me = g.player.pos;
        let (dx, dz) = ((centre(goal.0) - me[0]) as f32, (centre(goal.1) - me[2]) as f32);
        let len = dx.hypot(dz).max(1.0);
        // Along the aim and the heading (it turns to the aim at its rate).
        let a = g.player.heading as f32 * std::f32::consts::TAU / 4096.0;
        let mut probes = g.player.probes([dx / len, dz / len], 340.0);
        probes.extend(g.player.probes([a.sin(), a.cos()], 340.0));
        let feet = me[1] - g.player.feet();
        let by_thing = g.bodies.iter().filter(|b| !b.character && b.min_y < feet - 384 && b.covers(me, 1600)).any(|b| {
            let h = g.player.probe_height(b, me);
            probes.iter().any(|&(px, pz)| b.segment_hit([me[0], h, me[2]], [me[0] + px, h, me[2] + pz]).is_some())
        });
        if close > 0 || stuck > 3 || !slope {
            stride(g, centre(goal.0), centre(goal.1), by_thing);
        } else {
            stride_along(g, from, [centre(goal.0), centre(goal.1)]);
        }
        if std::env::var_os("WALKTRACE").is_some() {
            let p = g.player.pos;
            eprintln!("      moved {:.0} heading {} target {} speed {} {:?}", ((p[0] - before[0]) as f64).hypot((p[2] - before[2]) as f64), g.player.heading, g.player.target, g.player.speed, g.player.state);
        }
    }
    None
}

/// The feet's heights a walk from `from` reaches, cell by cell (`Ground`,
/// the characters left out: where they stand is a moment's).
fn on_foot(g: &Game, from: [i32; 3]) -> std::collections::HashMap<(i32, i32), Vec<i32>> {
    let mut ground = Ground::new(g);
    ground.characters = false;
    ground.narrow = true;
    // (A place put at may be too tight for the plan's body: from the
    // nearest about it that is not.)
    [(0, 0), (256, 0), (-256, 0), (0, 256), (0, -256), (256, 256), (-256, 256), (256, -256), (-256, -256)]
        .iter()
        .map(|&(dx, dz)| ground.ways([from[0] + dx, from[1], from[2] + dz], |_| false).0.floors)
        .find(|f| f.len() > 1)
        .unwrap_or_default()
}

/// Floats into room `room` (state 21): the way through the open cubes
/// (`afloat`, side to side and up and down), each cube's middle gone to
/// with the stick across, Cross up and Circle down. The cinematic the room
/// plays (`Some(None)` for none), or None if no way was had.
fn float_to(g: &mut Game, room: usize) -> Option<Option<usize>> {
    let r = g.set.room_info[room];
    let within = |c: (i32, i32, i32)| (0..3).all(|i| [c.0, c.1, c.2][i] >= r.min[i] && [c.0, c.1, c.2][i] <= r.max[i]);
    let open = afloat(g, g.player.pos);
    let mid = |v: i32| v.div_euclid(256) * 256 + 128;
    let me = g.player.pos;
    let from = (mid(me[0]), mid(me[1]), mid(me[2]));
    let mut came = std::collections::HashMap::from([(from, from)]);
    let mut todo = std::collections::VecDeque::from([from]);
    let mut end = None;
    while let Some(c) = todo.pop_front() {
        if within(c) {
            end = Some(c);
            break;
        }
        for d in [(256, 0, 0), (-256, 0, 0), (0, 256, 0), (0, -256, 0), (0, 0, 256), (0, 0, -256)] {
            let n = (c.0 + d.0, c.1 + d.1, c.2 + d.2);
            if open.contains(&n) && !came.contains_key(&n) {
                came.insert(n, c);
                todo.push_back(n);
            }
        }
    }
    let mut c = end?;
    let mut way = vec![c];
    while c != from {
        c = came[&c];
        way.push(c);
    }
    way.reverse();
    let mut cine = None;
    for &(x, y, z) in &way[1..] {
        for _ in 0..60 {
            let me = g.player.pos;
            let (dx, dy, dz) = (x - me[0], y - me[1], z - me[2]);
            if dy.abs() <= 40 && dx.abs().max(dz.abs()) <= 64 {
                break;
            }
            let mut keys = Vec::new();
            if dy < -40 {
                keys.push(input::CROSS);
            } else if dy > 40 {
                keys.push(input::CIRCLE);
            }
            if dx.abs().max(dz.abs()) > 64 {
                let h = ((dx as f32).atan2(dz as f32) * 4096.0 / std::f32::consts::TAU) as i32;
                keys.extend_from_slice(toward(g, h.rem_euclid(4096)));
            }
            run(g, &[(&keys, 1)], |g, _| cine = cine.or(g.cine.take()));
            if cine.is_some() {
                return Some(cine);
            }
        }
    }
    run(g, &[(&[], 10)], |g, _| cine = cine.or(g.cine.take()));
    Some(cine)
}

/// Where the player can float from `from` (state 21: no gravity, Cross
/// and Circle rise and sink, walls stop the body), by 256 cubes, their
/// middles.
fn afloat(g: &Game, from: [i32; 3]) -> std::collections::HashSet<(i32, i32, i32)> {
    use rugrats::world::collide;
    const FOOT: i32 = 256;
    let feet = g.player.feet();
    let cell = |v: i32| v.div_euclid(FOOT);
    let open = |c: (i32, i32, i32)| {
        let at = [c.0 * FOOT + FOOT / 2, c.1 * FOOT + FOOT / 2, c.2 * FOOT + FOOT / 2];
        let q = collide::query(&g.col, [at[0], at[1] - feet, at[2]]);
        q.kind == 0 && q.tile.is_some()
    };
    let mut seen = std::collections::HashSet::new();
    let mut todo = vec![(cell(from[0]), cell(from[1]), cell(from[2]))];
    while let Some(c) = todo.pop() {
        if !seen.insert(c) {
            continue;
        }
        for d in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            let n = (c.0 + d.0, c.1 + d.1, c.2 + d.2);
            if !seen.contains(&n) && open(n) {
                todo.push(n);
            }
        }
    }
    seen.into_iter().map(|c| (c.0 * FOOT + FOOT / 2, c.1 * FOOT + FOOT / 2, c.2 * FOOT + FOOT / 2)).collect()
}

/// Every place the level tests put the player can be walked to from
/// where the player is (COL and the objects' boxes; climbs and jumps).
#[test]
fn the_tests_places_are_reachable_on_foot() {
    let Some(disc) = disc() else { return };
    // Four levels at a time, each thread its own check.
    let levels = [2u8, 3, 4, 5, 6, 9, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21];
    let missed: Vec<String> = std::thread::scope(|s| {
        let disc = &disc;
        let runs: Vec<_> = levels
            .chunks(4)
            .map(|some| {
                s.spawn(move || {
                    ON_FOOT.with(|c| c.replace(Some(Vec::new())));
                    for &level in some {
                        win_level(disc, Level(level), &Progress::default());
                    }
                    ON_FOOT.with(|c| c.replace(None)).unwrap()
                })
            })
            .collect();
        runs.into_iter().flat_map(|r| r.join().unwrap()).collect()
    });
    assert!(missed.is_empty(), "not reached on foot ({}):\n{}", missed.len(), missed.join("\n"));
}

/// Runs with the stick through `points` (x, z) in turn, each reached
/// within 100: each leg straight, as a player lines one up along a ledge:
/// turned to it (`turn_to`), the camera put behind (R1), Up held (a climb
/// on the way is had by running into it).
fn go(g: &mut Game, points: &[[i32; 2]]) {
    for &[x, z] in points {
        let mut left = 600;
        let mut aimed = false;
        loop {
            let me = g.player.pos;
            let (dx, dz) = (x - me[0], z - me[2]);
            if dx * dx + dz * dz < 100 * 100 {
                break;
            }
            assert!(left > 0, "to ({x}, {z}): at {me:?}");
            left -= 1;
            let h = (((dx as f32).atan2(dz as f32) * 4096.0 / std::f32::consts::TAU) as i32).rem_euclid(4096);
            // Lined up again when off by more than a little (a climb or a
            // push turns the body).
            let off = ((h - g.player.heading + 2048).rem_euclid(4096) - 2048).abs();
            if !aimed || (off > 128 && dx * dx + dz * dz > 400 * 400) {
                turn_to(g, h);
                if g.fixed.is_none() {
                    run(g, &[(&[input::R1], 1), (&[], 2)], |_, _| {});
                }
                aimed = true;
            }
            let keys = toward(g, g.player.heading);
            run(g, &[(keys, 1)], |_, _| {});
        }
    }
    run(g, &[(&[], 10)], |_, _| {});
}

/// The levels in `LV` (comma separated) won walking (`walk_to`) where the
/// tests would put the player; the places not walked to are listed. Slow,
/// and level 10's races are not always won: run by hand.
#[test]
#[ignore]
fn levels_walked() {
    let Some(disc) = disc() else { return };
    let levels: Vec<u8> = std::env::var("LV").unwrap_or("2,3,4,5,6,9,10,11,12,13,15,16,17,18,20,21".into()).split(',').map(|x| x.parse().unwrap()).collect();
    WALKING.with(|c| c.replace(Some(Vec::new())));
    for level in levels {
        let t = std::time::Instant::now();
        win_level(&disc, Level(level), &Progress::default());
        eprintln!("level {level} won in {:?}", t.elapsed());
    }
    let missed = WALKING.with(|c| c.replace(None)).unwrap();
    eprintln!("not walked ({}):\n{}", missed.len(), missed.join("\n"));
}

/// Running at 45 degrees into a wall (the hub, x about 12300): the body
/// stops short of it, its side's reach at the wall, and slides along it at
/// full speed. The oracle (hub, Tommy put at (11800, 779, 19403) heading
/// 512, R1, Up held): x 11924 from the first step on, z 170 a frame.
#[test]
fn running_into_a_wall_slides_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 1);
    run(&mut g, &[(&[], 5)], |_, _| {});
    g.player.pos = [11800, 779, 19403];
    g.player.heading = 512;
    g.player.target = 512;
    run(&mut g, &[(&[], 5), (&[input::R1], 1), (&[], 4)], |_, _| {});
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::UP], 9)], |g, _| seen.push([g.player.pos[0], g.player.pos[2]]));
    assert_eq!(
        seen,
        [[11924, 19518], [11924, 19688], [11924, 19858], [11924, 20028], [11924, 20198], [11924, 20368], [11924, 20538], [11924, 20708], [11924, 20878]]
    );
    // Straight at it from x 11000 (heading 1024): the oracle's x by frame,
    // stopped at 12020 (the reach, 250, past the step short of the wall).
    g.player.pos = [11000, 779, 19403];
    g.player.heading = 1024;
    g.player.target = 1024;
    run(&mut g, &[(&[], 5), (&[input::R1], 1), (&[], 4)], |_, _| {});
    let mut xs = Vec::new();
    run(&mut g, &[(&[input::UP], 8)], |g, _| xs.push(g.player.pos[0]));
    assert_eq!(xs, [11170, 11340, 11510, 11680, 11850, 12020, 12020, 12020]);
    // With the wall on his left (heading 1536): the same, along -z, and on
    // past the wall's end (z about 18400). The oracle: x held at 11924
    // from the first step, z less 170 a frame to 18437, then diagonal
    // again. (The run's own heading there comes from the camera behind him
    // by the wall, a few degrees apart in the port: the steps across are
    // compared, not the places.)
    // (Head-on into it, the run bumps, 0x8005e1f4: played out first.)
    run(&mut g, &[(&[], 20)], |_, _| {});
    g.player.pos = [11800, 779, 19403];
    g.player.heading = 1536;
    g.player.target = 1536;
    run(&mut g, &[(&[], 5), (&[input::R1], 1), (&[], 4)], |_, _| {});
    let mut seen = Vec::new();
    run(&mut g, &[(&[input::UP], 11)], |g, _| seen.push([g.player.pos[0], g.player.pos[2]]));
    let held = seen[0][0];
    assert!(seen[..6].iter().all(|p| p[0] == held), "{seen:?}");
    assert!(seen[..6].windows(2).all(|w| w[1][1] - w[0][1] == -170), "{seen:?}");
    assert!(seen[6..].windows(2).all(|w| w[1][0] > w[0][0]), "{seen:?}");
}

/// Spike (level 11, his x half width 315) running at the wall past the
/// first hurdle: the oracle (put at (25252, 720, 4500) heading 0, R1, Up):
/// z 4700, 4900, 5100, 5300, 5500, 5700, then held at 5700.
#[test]
fn spike_stops_short_of_a_wall_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 11);
    run(&mut g, &[(&[], 5)], |_, _| {});
    g.player.pos = [25252, 720, 4500];
    g.player.heading = 0;
    g.player.target = 0;
    run(&mut g, &[(&[], 5), (&[input::R1], 1), (&[], 4)], |_, _| {});
    let mut zs = Vec::new();
    run(&mut g, &[(&[input::UP], 9)], |g, _| zs.push(g.player.pos[2]));
    assert_eq!(zs, [4700, 4900, 5100, 5300, 5500, 5700, 5700, 5700, 5700]);
}

/// A level's start cinematic leaves the player where its last frame put
/// him (its actors move the level's objects, 0x80073c34): the oracle, with
/// the start cinematics played, after each.
#[test]
fn the_start_cinematics_leave_the_player_as_the_original() {
    let Some(disc) = disc() else { return };
    for (level, at, heading) in [
        (2u8, [-5334, -245, 10864], 2881),
        (13, [22739, -5365, -5925], 711),
        (15, [44863, 8587, 12281], 2899),
        (18, [2333, 779, -926], 1647),
        (21, [4225, 779, 4176], 734),
    ] {
        let g = open(&disc, level);
        let off = (g.player.heading - heading + 2048).rem_euclid(4096) - 2048;
        assert_eq!(g.player.pos, at, "level {level}");
        assert!(off.abs() <= 16, "level {level} heading {} not {heading}", g.player.heading);
    }
}

/// So does one in play: level 2's gap (cine 3) leaves Tommy on Spike where
/// its last frame poses him, as the oracle shows after it (player object
/// at (45196, -688, -19249), heading 1331).
#[test]
fn a_cinematic_in_play_leaves_the_player_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 2);
    assert_eq!(enter_room(&mut g, 69), Some(3));
    g.played(3);
    let off = (g.player.heading - 1331 + 2048).rem_euclid(4096) - 2048;
    assert_eq!(g.player.pos, [45196, -688, -19249]);
    assert!(off.abs() <= 16, "heading {}", g.player.heading);
}

/// A thing tipped on its side (level 3's templates 526 and 650, turned
/// about x or z) stops the body only over its own outline: Tommy runs west
/// across room 36's floor, where those two had stood as endless strips.
#[test]
fn a_tipped_object_blocks_only_where_it_lies() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    run(&mut g, &[(&[], 5)], |_, _| {});
    put(&mut g, [116857, 779, 3723], 3072);
    let from = g.player.pos;
    for _ in 0..10 {
        let keys = toward(&g, 3072).to_vec();
        run(&mut g, &[(&keys, 1)], |_, _| {});
    }
    assert!(from[0] - g.player.pos[0] > 1000, "ran from {from:?} to {:?}", g.player.pos);
    let tipped: Vec<_> = g.solid.iter().filter(|b| (b.rot.y_axis.y - 1.0).abs() > 1e-3).collect();
    assert!(!tipped.is_empty(), "level 3 has tipped things");
    for b in tipped {
        let at = [b.pos.x as i32, 0, b.pos.z as i32];
        assert!(b.covers(at, 0), "{} covers its own place", b.template);
        assert!(!b.covers([at[0] + 3000, 0, at[2] + 3000], 0) && !b.covers([at[0], 0, at[2] - 20000], 0), "{} only near it", b.template);
    }
}

/// Level 5's elevator, landed on, takes Angelica down and stays at the
/// bottom while she is on it: the oracle (a jump onto it at (65540, 8708))
/// has it from y -4578 down to 7722 by frame 4100 and there still at 5600,
/// her at 6739 on it.
#[test]
fn level_5s_elevator_stays_down_while_ridden_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 5);
    run(&mut g, &[(&[], 2)], |_, _| {});
    g.player.pos = [65540, -6000, 8708];
    run(&mut g, &[(&[], 500)], |_, _| {});
    let y = |g: &Game| g.movers.iter().find(|m| m.template == 855).expect("the elevator").pos[1];
    assert_eq!(y(&g), 7722, "down, Angelica at {:?}", g.player.pos);
    run(&mut g, &[(&[], 600)], |_, _| {});
    assert_eq!(y(&g), 7722, "still down, Angelica at {:?}", g.player.pos);
    assert_eq!(g.player.pos[1], 6739);
}

/// A box put down comes down in front of the player, on the floor, from
/// the put-down's hands (event 2048 of frame 755, carried there by the
/// animation's own arms, 0x80056708): the oracle, level 6, Tommy at
/// (42741, 12263) facing +z with box LP 197, box LP 198 linked in at z
/// 13500 (room to put it down, 0x80063888): 197 rests at (42731, -1280,
/// 12959), short of 198, having slid 67 after it came down. No stacking: a box held into another is not let
/// go at all (`a_box_held_into_another_is_not_put_down_as_the_original`).
#[test]
fn a_box_put_down_comes_down_in_front_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    g.played(46);
    let at = |g: &Game, lp: usize| g.things.iter().position(|t| t.placement == lp).expect("the box");
    let (first, second) = (at(&g, 197), at(&g, 198));
    g.things[second].pos = rrt::glam::Vec3::new(42741.0, -1280.0, 13500.0);
    run(&mut g, &[(&[], 2)], |_, _| {});
    g.player.pos = [42741, -1269, 12263];
    g.player.heading = 0;
    g.player.target = 0;
    run(&mut g, &[(&[], 2), (&[input::CIRCLE], 1), (&[], 60)], |_, _| {});
    assert_eq!(g.carried(), Some(first), "picked up");
    run(&mut g, &[(&[input::CIRCLE], 1), (&[], 60)], |_, _| {});
    assert_eq!(g.carried(), None, "put down");
    let p = g.things[first].pos;
    assert_eq!(p.y as i32, -1280, "on the floor: {p:?}");
    // It slides on along the floor after it lands (the port's 12960; its
    // x 42772, let go a frame before the oracle's hand reached 42731).
    assert!((42681..42801).contains(&(p.x as i32)) && (12940..12980).contains(&(p.z as i32)), "in front: {p:?}");
}

/// GRAB takes a box in front of the player up to his cylinder and its
/// (0x800631d8 over the nearby list): the oracle, level 6's box LP 197 at
/// (42741, -1280, 12863), Tommy on its line facing it: taken from 1324
/// short, not from 1330 (3 x 250 + its corner radius 577), from 1000 with
/// the box 768 off his heading, not 1024 off, nor from 400 behind.
#[test]
fn grab_reaches_as_the_original() {
    let Some(disc) = disc() else { return };
    for (short, heading, taken) in [(1324, 0, true), (1330, 0, false), (1000, 768, true), (1000, 1024, false), (400, 2048, false), (600, 0, true)] {
        let mut g = open(&disc, 6);
        g.played(46);
        let k = g.things.iter().position(|t| t.placement == 197).expect("the box");
        run(&mut g, &[(&[], 2)], |_, _| {});
        g.player.pos = [42741, -1269, 12863 - short];
        g.player.heading = heading;
        g.player.target = heading;
        run(&mut g, &[(&[], 2), (&[input::CIRCLE], 1), (&[], 50)], |_, _| {});
        assert_eq!(g.carried() == Some(k), taken, "{short} short, heading {heading}");
    }
}

/// The step's probes meet an object at its own middle height (0x80063d3c):
/// level 3's tipped toy (template 526) at (70451, 701, 4290), its box's
/// middle y 504 above Tommy's head band's reach of it, is run past. The
/// oracle: run east along z 4290 from x 68500 at four phases of its turn,
/// x 170 a frame on through 73600, his y bobbing about 779 (never on it).
#[test]
fn tommy_runs_past_level_3s_tipped_toy_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    run(&mut g, &[(&[], 5)], |_, _| {});
    for k in 0..4 {
        run(&mut g, &[(&[], 37 * k)], |_, _| {});
        g.player.pos = [68500, 779, 4290];
        g.player.heading = 1024;
        g.player.target = 1024;
        run(&mut g, &[(&[], 5), (&[input::R1], 1), (&[], 4)], |_, _| {});
        let mut seen = Vec::new();
        run(&mut g, &[(&[input::UP], 30)], |g, _| seen.push(g.player.pos));
        let xs: Vec<i32> = seen.iter().map(|p| p[0]).collect();
        assert!(xs.windows(2).all(|w| w[1] - w[0] == 170), "phase {k}: {xs:?}");
        assert!(seen.iter().all(|p| p[1] > 700), "phase {k}: not up on it: {seen:?}");
    }
}

/// A box held into another is not put down (0x80063888): the oracle, level
/// 6, Tommy at (42741, 12263) facing +z holding box LP 197, box LP 198
/// moved to z 13100 or 13400 (and linked into his room): Circle does
/// nothing; at 13500 he puts it down (198's half-depth 256 and the held
/// box's half-width 450 from the held box's middle, z 12744).
#[test]
fn a_box_held_into_another_is_not_put_down_as_the_original() {
    let Some(disc) = disc() else { return };
    for (z, refused) in [(13100, true), (13400, true), (13500, false)] {
        let mut g = open(&disc, 6);
        g.played(46);
        let at = |g: &Game, lp: usize| g.things.iter().position(|t| t.placement == lp).expect("the box");
        let (first, second) = (at(&g, 197), at(&g, 198));
        g.things[second].pos = rrt::glam::Vec3::new(42741.0, -1280.0, z as f32);
        run(&mut g, &[(&[], 2)], |_, _| {});
        g.player.pos = [42741, -1269, 12263];
        g.player.heading = 0;
        g.player.target = 0;
        run(&mut g, &[(&[], 2), (&[input::CIRCLE], 1), (&[], 60)], |_, _| {});
        assert_eq!(g.carried(), Some(first), "picked up");
        run(&mut g, &[(&[input::CIRCLE], 1), (&[], 60)], |_, _| {});
        assert_eq!(g.carried().is_some(), refused, "198 at {z}");
    }
}

/// A run at level 18's wall at a slant goes on along it, and into the
/// corner past it, at another wall and another; head-on into the last it
/// bumps once (state 9, 13 frames), then stands with Up still held, as the
/// oracle shows (from (28664, 12748) facing 1433, R1, Up held: along x
/// 30866 from z 11166, x 31335 from z 6039, the bump at (31335, 5529),
/// then state 0).
#[test]
fn a_run_slides_along_walls_and_bumps_once_as_the_original() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 18);
    g.player.pos = [28664, 779, 12748];
    g.player.heading = 1433;
    g.player.target = 1433;
    run(&mut g, &[(&[], 3), (&[input::R1], 1), (&[], 2)], |_, _| {});
    let (mut path, mut bumps, mut was) = (Vec::new(), 0, None);
    run(&mut g, &[(&[input::UP], 120)], |g, _| {
        path.push(g.player.pos);
        if g.player.action == Some(rugrats::actors::player::anim::BUMP) && was != Some(rugrats::actors::player::anim::BUMP) {
            bumps += 1;
        }
        was = g.player.action;
    });
    let near = |p: [i32; 3], x: i32, z: i32| (p[0] - x).abs() <= 16 && (p[2] - z).abs() <= 16;
    assert!(path.iter().any(|&p| p[0] - 30866 <= 16 && (30866 - p[0]) <= 16 && p[2] < 10000), "along the first wall: {path:?}");
    assert!(path.iter().any(|&p| (p[0] - 31335).abs() <= 16 && p[2] < 5900), "along the second: {path:?}");
    let end = g.player.pos;
    assert!(near(end, 31335, 5529), "held at {end:?}");
    assert_eq!(bumps, 1, "one bump");
    assert_eq!(g.player.state, State::Stand);
}

/// The stick held on in the air pushes the flight along (0x80061330):
/// level 6's shelf (Tommy put at (40334, -4341, -34944) facing 1024, R1,
/// Up held, Cross): the oracle's steps across, a frame apart, 111, 122,
/// 132, 142, 152, 160, 168, then 186, 190, 195, 200, 204, 207, 210. And
/// Cross at the shelf's edge still jumps from it (the oracle: up from the
/// shelf's height, not down to the floor 3072 under it).
#[test]
fn the_stick_pushes_a_jump_on_as_the_original() {
    let Some(disc) = disc() else { return };
    let start = |g: &mut Game, pre: usize| {
        g.player.pos = [40334, -4341, -34944];
        g.player.heading = 1024;
        g.player.target = 1024;
        run(g, &[(&[], 3), (&[input::R1], 1), (&[], 2), (&[input::UP], pre)], |_, _| {});
    };
    let mut g = open(&disc, 6);
    start(&mut g, 1);
    let mut xs = Vec::new();
    run(&mut g, &[(&[input::UP, input::CROSS], 15), (&[input::UP], 3)], |g, _| xs.push(g.player.pos[0]));
    let steps: Vec<i32> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    let oracle = [111, 122, 132, 142, 152, 160, 168, 0, 0, 186, 190, 195, 200, 204, 207, 210];
    for (k, (&s, &o)) in steps.iter().zip(&oracle).enumerate() {
        assert!(o == 0 || (s - o).abs() <= 4, "step {k}: {s} against {o} ({steps:?})");
    }
    let mut g = open(&disc, 6);
    start(&mut g, 3);
    let mut ys = Vec::new();
    run(&mut g, &[(&[input::UP, input::CROSS], 12)], |g, _| ys.push(g.player.pos[1]));
    assert!(ys.iter().all(|&y| y < -4000) && ys.iter().any(|&y| y < -5000), "{ys:?}");
}
