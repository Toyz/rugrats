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
    // Each climb plays its height's animation (0x80056e44), about a
    // second for two boxes.
    run(&mut g, &[(&[], 3), (&[input::UP], 120)], |_, _| {});
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
    // Won, not timed out: the clock goes off with Angelica stopped
    // (mode 0) and her health back to full.
    let mut ended = None;
    run(&mut g, &steps, |g, f| {
        if ended.is_none() && f > 20 && g.timer.is_none() {
            let a = g
                .npcs
                .iter()
                .find(|n| n.id == 600)
                .map(|n| (matches!(n.mode, rugrats::actors::npc::Mode::Idle), n.health == n.max_health));
            ended = Some((f, a));
        }
    });
    assert!(matches!(ended, Some((_, Some((true, true))))), "toss ended {ended:?}");
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
    run(&mut g, &[(&[], 3), (&[input::UP], 120)], |_, _| {});
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

/// Puts the player 600 short of `at`, facing it, and presses Triangle;
/// the cinematic that follows, if any.
fn use_at(g: &mut Game, at: [i32; 3]) -> Option<usize> {
    put(g, [at[0], at[1], at[2] - 600], 0);
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
    let fridge = g.set.placements.iter().find(|p| p.kind == 1 && p.id == 602).expect("the refrigerator").pos;
    assert_eq!(use_at(&mut g, fridge), Some(67));
    g.cine_finished(67);
    let spike = g.npcs.iter().find(|n| n.id == 14 && !n.hidden).expect("Spike out").pos;
    assert_eq!(use_at(&mut g, spike), Some(68));
}

/// Level 3: the cart ride (room 19, cine 9) to the bakery; the pie toss,
/// throwing at whichever of Larry and Steve is nearer, until four hits
/// (cine 13); the aisles, then the boss's room (36, cine 86); Triangle at
/// the switch: cine 14, won.
#[test]
fn level_3_aisle_seven_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 3);
    assert_eq!(enter_room(&mut g, 19), Some(9));
    g.cine_finished(9);
    let cine = play(&mut g, 30 * 120, |g, f| {
        let me = g.player.pos;
        let near = g.npcs.iter().filter(|n| matches!(n.id, 75 | 76) && !n.hidden).min_by_key(|n| {
            let (dx, dz) = ((n.pos[0] - me[0]) as i64, (n.pos[2] - me[2]) as i64);
            dx * dx + dz * dz
        });
        if let Some(n) = near {
            let at = n.pos;
            face(g, at);
        }
        if f % 15 == 0 { vec![input::TRIANGLE] } else { vec![] }
    });
    assert_eq!(cine, Some(13), "the toss");
    g.cine_finished(13);
    assert_eq!(enter_room(&mut g, 36), Some(86));
    g.cine_finished(86);
    // The switch, LP 278 (+0x44 16, which 0x8004691c makes cine 14).
    let switch = g.set.placements[278].pos;
    assert_eq!(use_at(&mut g, switch), Some(14));
}

/// Level 7: Reptar walks the city to City Hall (room 6): cine 85, the end
/// of the game.
#[test]
fn level_7_reptar_2010_reaches_city_hall() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 7);
    assert_eq!(enter_room(&mut g, 6), Some(85));
    g.cine_finished(85);
    assert!(g.ending);
}

/// Level 10: Chuckie tags each hiding baby (Triangle beside it), then waits
/// in the playpen, which catches it as it runs home (cines 75-77); the
/// third catch plays cine 78: won.
#[test]
fn level_10_chuckies_glasses_is_won() {
    let Some(disc) = disc() else { return };
    let mut g = open(&disc, 10);
    for id in [608u16, 609, 1492] {
        let baby = g.npcs.iter().find(|n| n.id == id).expect("a baby").pos;
        put(&mut g, [baby[0], baby[1], baby[2] - 500], 0);
        face(&mut g, baby);
        run(&mut g, &[(&[input::TRIANGLE], 1), (&[], 2)], |_, _| {});
        let y = g.player.pos[1];
        put(&mut g, [5926, y, 21212], 0);
        let mut cine = None;
        run(&mut g, &[(&[], 30)], |g, _| cine = cine.or(g.cine.take()));
        assert!(matches!(cine, Some(75..=77)), "baby {id}: {cine:?}");
        g.cine_finished(cine.unwrap());
    }
    assert_eq!(g.cine.take(), Some(78));
}

/// Finishes cinematic `n` and any the level chains after it (an intro
/// that follows a win), as the app plays them in turn.
fn finish(g: &mut Game, mut n: usize) {
    loop {
        g.cine_finished(n);
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
    // Ring 1: the bone moves to the next stand each time.
    // Ring 1: the bone moves to the next stand each time; Spike eats it
    // from the stand's top.
    let mut cine = None;
    for _ in 0..12 {
        let b = g.goodies.iter().find(|b| b.kind == 14 && !b.taken).expect("a bone").pos;
        put(&mut g, [b.x as i32, b.y as i32 - 50, b.z as i32 - 100], 0);
        run(&mut g, &[(&[input::UP], 3), (&[], 3)], |g, _| cine = cine.or(g.cine.take()));
    }
    assert_eq!(cine, Some(54), "ring 1");
    finish(&mut g, 54);
    // Ring 2: along +x into the ramp's zone (x 15000..22700), Cross 6700
    // in.
    let y = g.player.pos[1];
    put(&mut g, [13500, y, 34300], 1024);
    let mut from = None;
    let cine = play(&mut g, 30 * 20, |g, _| {
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
    finish(&mut g, 57);
    // Ring 3: the meter rises 6 a frame from 6; stop it at 174 (28
    // frames), then the second from 46 at 172 (21 frames).
    let presses = [0usize, 28, 49];
    let cine = play(&mut g, 120, |_, f| if presses.contains(&f) { vec![input::TRIANGLE] } else { vec![] });
    assert_eq!(cine, Some(39), "ring 3's leap");
    g.cine_finished(39);
    let cine = play(&mut g, 30 * 10, |_, _| vec![]);
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
    face(g, aim);
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
    let course: [(u8, usize, &[(i32, i32)], usize); 9] = [
        (25, 4, &[(384, 217)], 1028),
        (28, 5, &[(0, 217), (0, 64)], 1024),
        (24, 4, &[(-256, 217), (0, 81), (-1920, 115)], 1029),
        (29, 5, &[(256, 217), (0, 115)], 1023),
        (23, 5, &[(256, 166), (128, 217), (-640, 217)], 1030),
        (30, 5, &[(-384, 217), (0, 98)], 1026),
        (26, 4, &[(384, 217), (768, 217), (-1280, 200)], 1031),
        (31, 5, &[(-384, 217), (-768, 217), (-1280, 217), (-1664, 217)], 1027),
        (27, 3, &[(-512, 217)], 35),
    ];
    for (level, par, shots, next) in course {
        let shots: Vec<Shot> = shots.iter().map(|&(offset, power)| Shot { offset, power }).collect();
        assert!(shots.len() <= par);
        let (r, g) = replay(&disc, level, &shots);
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
    g.cine_finished(13);
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
