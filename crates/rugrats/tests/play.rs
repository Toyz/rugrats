//! Behaviour the port must keep while it changes: levels played headless
//! with scripted pads, as the play_script example runs them. Skipped
//! without a disc in work/disc.

use std::path::Path;

use rugrats::disc::GameDisc;
use rugrats::input::{self, Pads};
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
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

fn open(disc: &GameDisc, level: u8) -> Game {
    Game::open_from(disc, Level(level), None, &Progress::default()).unwrap()
}

fn put(g: &mut Game, at: [i32; 3], heading: i32) {
    g.player.pos = at;
    g.player.heading = heading;
    g.player.target = heading;
    g.set_camera_yaw(heading as f32);
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
    run(&mut g, &[(&[], 2), (&[input::UP], 40)], |_, _| {});
    assert!(g.player.pos[2] < 23500, "the shut door stops him");
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

/// Level 6: four boxes stacked on the -2048 step reach the switch
/// platform (-4736).
#[test]
fn level_6_box_stack_reaches_the_platform() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    let boxes: Vec<usize> = (0..g.things.len()).filter(|&k| g.things[k].template == 1016).take(4).collect();
    for (i, &k) in boxes.iter().enumerate() {
        g.things[k].pos = rrt::glam::Vec3::new(44500.0, -2304.0 - 506.0 * i as f32, 13900.0);
    }
    put(&mut g, [44500, -2293, 12900], 0);
    run(&mut g, &[(&[], 3), (&[input::UP], 60)], |_, _| {});
    assert!(g.player.pos[2] > 14336 && g.player.pos[1] < -4800, "on the platform: {:?}", g.player.pos);
}

/// Level 16's pie toss: five hits on Angelica win it.
#[test]
fn level_16_toss_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 16);
    put(&mut g, [31600, -1750, -8141], 1024);
    let mut steps: Vec<(&[usize], usize)> = vec![(&[input::UP], 15), (&[], 330)];
    for _ in 0..60 {
        steps.push((&[input::UP, input::TRIANGLE], 1));
        steps.push((&[input::UP], 14));
    }
    // Won before the clock (30 s of unfrozen time) could run out: the
    // timer goes off with Angelica's health back to full.
    let mut ended = None;
    run(&mut g, &steps, |g, f| {
        if ended.is_none() && f > 20 && g.timer.is_none() {
            ended = Some(f);
        }
    });
    assert!(ended.is_some_and(|f| f < 1000), "won at {ended:?}");
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

/// Walks onto each pickup `want` names (Tommy put beside it, a step
/// forward), as a player would, until the level plays a cinematic.
fn collect(g: &mut Game, want: impl Fn(&rugrats::world::game::Goodie) -> bool) -> Option<usize> {
    for k in 0..g.goodies.len() {
        if g.goodies[k].taken || !want(&g.goodies[k]) {
            continue;
        }
        let p = g.goodies[k].pos;
        put(g, [p.x as i32, p.y as i32 - 100, p.z as i32 - 300], 0);
        let mut cine = None;
        run(g, &[(&[input::UP], 6), (&[], 2)], |g, _| cine = cine.or(g.cine.take()));
        if cine.is_some() {
            return cine;
        }
        assert!(g.goodies[k].taken, "pickup {k} at {p:?} not taken");
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
    assert_eq!(collect(&mut g, |n| n.template == 446), Some(26));
}

/// Level 18: the 35th egg ends the clock and wins (cine 24).
#[test]
fn level_18_egg_hunt_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 18);
    assert_eq!(collect(&mut g, |e| e.template == 496), Some(24));
}

/// Level 15: Triangle beside each ? box starts its colour, and its
/// balloons taken fill it; the sixth colour wins (cine 72).
#[test]
fn level_15_mirrorland_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 15);
    let mut won = None;
    for id in 207..=212u16 {
        let b = g.set.placements.iter().find(|p| p.kind == 1 && p.id == id).unwrap().pos;
        // Beside the box, in its room.
        let room = g.room_of(b);
        let at = [[600, 0], [-600, 0], [0, 600], [0, -600]]
            .map(|[dx, dz]| [b[0] + dx, b[1], b[2] + dz])
            .into_iter()
            .find(|&p| g.room_of(p) == room)
            .unwrap_or(b);
        put(&mut g, at, 0);
        run(&mut g, &[(&[], 4), (&[input::TRIANGLE], 1), (&[], 2)], |_, _| {});
        assert!(g.is_hidden(id), "box {id} did not open");
        won = collect(&mut g, |b| (6..=11).contains(&b.kind));
    }
    assert_eq!(won, Some(72));
}

/// Level 9: Mr. Friend kicked down four times; the first brings back all
/// three (cine 18, as the app plays it), the fourth wins (cine 19).
#[test]
fn level_9_mr_friend_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 9);
    let friends = [46u16, 91, 800];
    for _ in 0..400 {
        let Some(k) = g.npcs.iter().position(|n| friends.contains(&n.id) && !n.hidden) else { break };
        let p = g.npcs[k].pos;
        put(&mut g, [p[0], p[1], p[2] - 500], 0);
        let mut cine = None;
        run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 40)], |g, _| cine = cine.or(g.cine.take()));
        match cine {
            Some(18) => g.cine_finished(18),
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
    let at = g.spots.iter().find(|s| s.group == 0 && s.used).unwrap().pos;
    put(&mut g, [at[0], at[1], at[2] - 1000], 0);
    let mut cine = None;
    run(&mut g, &[(&[], 10)], |g, _| cine = cine.or(g.cine.take()));
    assert_eq!(cine, Some(48));
}

/// Walks into room `room` (put at its middle, a step), and the cinematic
/// it plays, if any.
fn enter_room(g: &mut Game, room: usize) -> Option<usize> {
    let r = &g.set.room_info[room];
    let mid = [0, 1, 2].map(|i| (r.min[i] + r.max[i]) / 2);
    put(g, mid, g.player.heading);
    let mut cine = None;
    run(g, &[(&[input::UP], 2), (&[], 10)], |g, _| cine = cine.or(g.cine.take()));
    cine
}

/// Level 5: room 30's cine 38 sets Angelica floating through the ducts;
/// the garage, room 38, plays cine 40: won.
#[test]
fn level_5_outer_space_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 5);
    assert_eq!(enter_room(&mut g, 30), Some(38));
    g.cine_finished(38);
    assert!(g.player.float.is_some(), "floating after cine 38");
    assert_eq!(enter_room(&mut g, 38), Some(40));
    g.cine_finished(40);
    assert_eq!(g.won, Some(true));
}

/// Level 6 from the hand-over: room 11's cine 46 gives the level to Tommy,
/// four Reptar boxes stacked by the step take him onto the platform, and
/// Triangle at the switch there plays cine 47: won.
#[test]
fn level_6_toy_palace_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 6);
    assert_eq!(enter_room(&mut g, 11), Some(46));
    g.cine_finished(46);
    let boxes: Vec<usize> = (0..g.things.len()).filter(|&k| g.things[k].template == 1016).take(4).collect();
    for (i, &k) in boxes.iter().enumerate() {
        g.things[k].pos = rrt::glam::Vec3::new(44500.0, -2304.0 - 506.0 * i as f32, 13900.0);
    }
    put(&mut g, [44500, -2293, 12900], 0);
    run(&mut g, &[(&[], 3), (&[input::UP], 60)], |_, _| {});
    assert!(g.player.pos[1] < -4800, "on the platform: {:?}", g.player.pos);
    let switch = g.set.placements.iter().find(|p| p.kind == 1 && p.cine == 47).expect("the switch").pos;
    let me = g.player.pos;
    let heading = ((switch[0] - me[0]) as f32).atan2((switch[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
    put(&mut g, [switch[0], me[1], switch[2] - 600], heading as i32 & 4095);
    let mut cine = None;
    run(&mut g, &[(&[], 3), (&[input::TRIANGLE], 1), (&[], 5)], |g, _| cine = cine.or(g.cine.take()));
    assert_eq!(cine, Some(47), "at {:?}, switch {switch:?}", g.player.pos);
    g.cine_finished(47);
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
    for (level, won, lob) in [(20, 64, 3000i64), (21, 50, 2200)] {
        let mut g = open(&disc, level);
        let mut cine = None;
        for _ in 0..3 {
            cine = play(&mut g, 6000, |g, f| {
                let me = g.player.pos;
                let d2 = |p: [i32; 3], q: [i32; 3]| {
                    let (dx, dz) = ((p[0] - q[0]) as i64, (p[2] - q[2]) as i64);
                    dx * dx + dz * dz
                };
                let Some(angelica) = g.npcs.iter().find(|n| n.id == 600).map(|n| n.pos) else { return vec![] };
                if d2(angelica, me) > lob * lob {
                    return if f % 2 == 0 { vec![input::CIRCLE] } else { vec![] };
                }
                // Turn to the baby farthest from her (the stick's turn,
                // done at once) and lob.
                let babies = [1776u16, 609, 608, 1492];
                let far =
                    g.npcs.iter().filter(|n| babies.contains(&n.id) && !n.hidden).max_by_key(|n| d2(n.pos, angelica));
                if let Some(b) = far {
                    let h =
                        ((b.pos[0] - me[0]) as f32).atan2((b.pos[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
                    let h = (h as i32).rem_euclid(4096);
                    g.player.heading = h;
                    g.player.target = h;
                }
                if f % 2 == 0 { vec![input::TRIANGLE] } else { vec![] }
            });
            if cine == Some(won) {
                break;
            }
            // A lost round: its cinematic, then the next.
            if let Some(n) = cine {
                g.cine_finished(n);
            }
        }
        assert_eq!(cine, Some(won), "level {level}: {}", g.script.as_ref().map_or(String::new(), |s| s.describe()));
    }
}

/// Turns the player to face `to` (the stick's turn, done at once).
fn face(g: &mut Game, to: [i32; 3]) {
    let me = g.player.pos;
    let h = ((to[0] - me[0]) as f32).atan2((to[2] - me[2]) as f32) * 4096.0 / std::f32::consts::TAU;
    let h = (h as i32).rem_euclid(4096);
    g.player.heading = h;
    g.player.target = h;
}

/// Level 13: Tommy into the kitchen (room 20) before Angelica reaches her
/// last post plays cine 21: won.
#[test]
fn level_13_cookie_race_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 13);
    run(&mut g, &[(&[], 30)], |_, _| {});
    assert_eq!(enter_room(&mut g, 20), Some(21));
}

/// Level 2: the gap in the fence (room 69, cine 3), the ride after the
/// goose to the court (room 44, cine 4), then pucks kicked at the goose
/// (Triangle, facing it) until it falls: cine 5.
#[test]
fn level_2_grandpas_teeth_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 2);
    assert_eq!(enter_room(&mut g, 69), Some(3));
    g.cine_finished(3);
    assert_eq!(enter_room(&mut g, 44), Some(4));
    g.cine_finished(4);
    // A clear shot: the goose in range, Chuckie off the line to it.
    let cine = play(&mut g, 30 * 130, |g, f| {
        let pos = |id: u16| g.npcs.iter().find(|n| n.id == id).map(|n| n.pos);
        let (Some(goose), Some(chuckie)) = (pos(11), pos(1776)) else { return vec![] };
        let me = g.player.pos;
        let (gx, gz) = ((goose[0] - me[0]) as f64, (goose[2] - me[2]) as f64);
        let (cx, cz) = ((chuckie[0] - me[0]) as f64, (chuckie[2] - me[2]) as f64);
        let len = gx.hypot(gz).max(1.0);
        let along = (cx * gx + cz * gz) / len;
        let off = (cx * gz - cz * gx).abs() / len;
        let clear = off > 900.0 || along < 0.0 || along > len + 600.0;
        face(g, goose);
        if f % 4 == 0 && len < 14000.0 && clear { vec![input::TRIANGLE] } else { vec![] }
    });
    let goose = g.npcs.iter().find(|n| n.id == 11).map(|n| (n.pos, n.health, n.hidden, format!("{:?}", n.mode)));
    let chuckie = g.npcs.iter().find(|n| n.id == 1776).map(|n| (n.pos, format!("{:?}", n.mode)));
    let court = g.set.room_info.get(44).map(|r| (r.min, r.max));
    assert_eq!(cine, Some(5), "goose {goose:?}, chuckie {chuckie:?}, court {court:?}, Tommy at {:?}", g.player.pos);
}
