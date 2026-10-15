//! Level 11, Circus Angelicus (docs/engine/level-11.md): three rings.
//! Ring 1, Spike eats 12 bones in 110 s (one at a time on six stands).
//! Ring 2, the tricycle takes the ramp; Cross on the boards jumps, and the
//! middle band of press distances lands in the net. Ring 3, Angelica's two
//! meter readings launch Chuckie off the seesaw; both in their zones win
//! (cine 62). A miss is that ring's Lose cine and a try.

use rrt::glam::Vec3;
use rugrats_data::hierarchy::Hierarchy;

use crate::animator::Animator;
use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::npc::Mode;

/// Ids: the tricycle, Angelica, Chuckie, Tommy, the net, the board.
const TRIKE: u16 = 508;
const ANGELICA: u16 = 600;
const CHUCKIE: u16 = 1776;
const TOMMY: u16 = 1492;
const NET: u16 = 7777;
/// Ring 1 (0x8007d6f8): Spike's start, Tommy's, 110 s, the bone's groups.
const SPIKE_AT: [i32; 3] = [25252, 751, 995];
const SPIKE_HEADING: i32 = 1095;
const TOMMY_AT: [i32; 3] = [26113, 833, -4384];
const CLOCK: i32 = 110 * 300;
const BONES: u32 = 12;
const FIRST_GROUP: usize = 2;
const LAST_GROUP: usize = 7;
/// Ring 2: the take-off zone (template 878 at (18427, -733, 34307); the
/// ramp from x 15000 to its top at 22700, as seen), the jump's heading,
/// the net (34198, 1041, 34949).
const ZONE: ([i32; 2], [i32; 2]) = ([15000, 33300], [22700, 35300]);
const TAKE_OFF: i32 = 1050;
const NET_AT: [i32; 3] = [34198, 1041, 34949];
const NET_HALF: i32 = 2500;
/// Ring 3 (0x8007dd3c): Angelica on the tower, Chuckie on the board, the
/// fixed camera.
const TOWER: [i32; 3] = [23400, -2480, 40500];
const BOARD_END: [i32; 3] = [15664, 860, 40434];
const RING3_EYE: Vec3 = Vec3::new(27221.0, -3810.0, 39229.0);
const METER_MAX: i32 = 220;
/// Cines.
const LOSE1: usize = 53;
const WIN1: usize = 54;
const TRIKE_INTRO: usize = 55;
const LOSE2: usize = 56;
const WIN2: usize = 57;
const RING3_INTRO: usize = 58;
const LEAP: usize = 39;
const LOSE3: [usize; 3] = [59, 60, 61];
const WON: usize = 62;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Bones,
    /// Ring 2: driving (9), on the boards from `from` (2), in the air (3).
    Ride,
    Ramp([i32; 3]),
    Flying,
    /// Ring 3: the meter (5) and Chuckie flying (7).
    Meter { first: Option<i32>, first_up: bool, second: Option<i32>, done_first: Option<i32> },
    Launched { good: bool },
    Waiting,
}

pub struct Level11 {
    pub phase: u8,
    step: Step,
    bones: u32,
    group: usize,
    bodies: [(usize, Hierarchy, Animator); 4],
    respawn: ([i32; 3], i32),
    /// Ring 3's launch (heading, forward speed) and whether Chuckie left.
    launch: (i32, i32),
    flown: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level11 {
    /// EpisodeStart (0x8007d8f0).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<Level11, String> {
        for id in [CHUCKIE, TRIKE, NET] {
            g.hide(id, true);
        }
        if let Some(k) = find(g, ANGELICA) {
            g.npcs[k].mode = Mode::Idle;
        }
        let body = |c: usize| -> Result<(usize, Hierarchy, Animator), String> {
            let h = g.set.hierarchies.get(c).cloned().flatten().ok_or(format!("no hierarchy {c}"))?;
            Ok((c, h, Animator::load(disc, c as u16)?))
        };
        let bodies = [body(6)?, body(34)?, body(3)?, body(2)?];
        let mut l = Level11 { phase: 1, step: Step::Bones, bones: 0, group: FIRST_GROUP, bodies, respawn: (SPIKE_AT, SPIKE_HEADING), launch: (3072, 2816), flown: false };
        l.ring1(g);
        Ok(l)
    }

    /// The player takes body `b` (0, Spike .. 3, Chuckie) at `at`
    /// (0x800585b0's swap).
    fn become_body(&self, g: &mut Game, b: usize, at: [i32; 3], heading: i32) {
        let (c, h, a) = (self.bodies[b].0, self.bodies[b].1.clone(), self.bodies[b].2.clone());
        let turn = if matches!(c, 6 | 34) { crate::player::SLOW_TURN } else { crate::player::TURN };
        g.player.set_body(h, a, turn, &g.anims);
        g.character = c;
        g.player.pos = at;
        g.player.heading = heading;
        g.player.target = heading;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.player.frozen = false;
        g.player.moving_jump = if c == 6 { (-1700, 20) } else { (-1000, 40) };
        g.set_camera_yaw(heading as f32);
    }

    /// Ring 1's set-up (0x8007d6f8).
    fn ring1(&mut self, g: &mut Game) {
        self.become_body(g, 0, SPIKE_AT, SPIKE_HEADING);
        if let Some(k) = find(g, TOMMY) {
            let n = &mut g.npcs[k];
            n.hidden = false;
            n.place(TOMMY_AT, &g.col);
            n.speed_index = 2;
            n.mode = Mode::Posts { prev: 0, cur: 5 };
        }
        g.timer = Some(CLOCK);
        self.bones = 0;
        self.group = FIRST_GROUP;
        self.place_bone(g);
        self.step = Step::Bones;
        self.respawn = (SPIKE_AT, SPIKE_HEADING);
        g.checkpoint = Some(self.respawn);
    }

    /// The bone to its group's spot (0x8004fe14(&bone, group, 1)).
    fn place_bone(&mut self, g: &mut Game) {
        let at = g.spots.iter().find(|s| s.group as usize == self.group).map(|s| s.pos);
        if let (Some(at), Some(b)) = (at, g.goodies.iter_mut().find(|x| x.kind == crate::game::goodie::BONE)) {
            b.pos = Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
            b.taken = false;
        }
    }

    /// Ring 3's set-up (0x8007dc68): Angelica on the tower, Chuckie on the
    /// board, the meter.
    fn ring3(&mut self, g: &mut Game) {
        self.become_body(g, 2, TOWER, 3072);
        g.player.frozen = true;
        if let Some(k) = find(g, CHUCKIE) {
            g.hide(CHUCKIE, false);
            g.npcs[k].place(BOARD_END, &g.col);
            g.npcs[k].heading = 3072;
            g.npcs[k].mode = Mode::Idle;
        }
        let look = Vec3::new(BOARD_END[0] as f32, (BOARD_END[1] - 1000) as f32, BOARD_END[2] as f32);
        g.fixed = Some((RING3_EYE, look));
        self.phase = 3;
        self.respawn = (TOWER, 3072);
        g.checkpoint = Some(self.respawn);
        self.step = Step::Meter { first: None, first_up: true, second: None, done_first: None };
    }

    /// A miss (no cry, 0x80080044): a try; the hub when none are left.
    fn lose_try(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some((crate::game::HUB, None));
            return false;
        }
        true
    }

    fn landed(g: &Game) -> bool {
        matches!(g.player.state, crate::player::State::Stand | crate::player::State::Walk | crate::player::State::Run)
    }
}

impl crate::game::Script for Level11 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        g.npc_events.clear();
        match self.step {
            Step::Bones => {
                // 0x800514b0's level 11 path: each bone moves the next to the
                // next stand; the twelfth wins the ring.
                for (kind, _) in std::mem::take(&mut g.collected) {
                    if kind == crate::game::goodie::BONE {
                        self.bones += 1;
                        if self.bones >= BONES {
                            g.timer = None;
                            g.tries = 3;
                            g.cine = Some(WIN1);
                            self.step = Step::Waiting;
                        } else {
                            self.group = if self.group >= LAST_GROUP { FIRST_GROUP } else { self.group + 1 };
                            self.place_bone(g);
                        }
                    }
                }
            }
            Step::Ride => {
                let p = g.player.pos;
                if p[0] >= ZONE.0[0] && p[0] <= ZONE.1[0] && p[2] >= ZONE.0[1] && p[2] <= ZONE.1[1] {
                    self.step = Step::Ramp(p);
                    g.player.no_jump = true;
                }
            }
            Step::Ramp(from) => {
                let p = g.player.pos;
                let inside = p[0] >= ZONE.0[0] && p[0] <= ZONE.1[0] && p[2] >= ZONE.0[1] && p[2] <= ZONE.1[1];
                if pads.pressed(0, input::CROSS) {
                    // 0x800589f4: the press's distance picks the jump.
                    let d = (((p[0] - from[0]) as f64).hypot((p[2] - from[2]) as f64)) as i32;
                    let mult = match d {
                        1..=5999 => 4,
                        6001..=7499 => 10,
                        _ => 16,
                    };
                    let factor = if g.player.speed > 8 { 40 } else { 12 };
                    let along = factor * g.player.speed * mult;
                    g.player.launch(TAKE_OFF, -1000, along, &g.trig);
                    g.player.no_jump = false;
                    self.step = Step::Flying;
                } else if !inside {
                    g.player.no_jump = false;
                    self.step = if Self::landed(g) { Step::Ride } else { Step::Flying };
                }
            }
            Step::Flying => {
                if Self::landed(g) {
                    let p = g.player.pos;
                    let on_net = (p[0] - NET_AT[0]).abs() < NET_HALF && (p[2] - NET_AT[2]).abs() < NET_HALF;
                    g.cine = Some(if on_net { WIN2 } else { LOSE2 });
                    self.step = Step::Waiting;
                }
            }
            Step::Meter { mut first, mut first_up, mut second, done_first } => {
                // The meter (0x80058a78): 6 a frame.
                if done_first.is_none() {
                    if let Some(f) = first.as_mut() {
                        if first_up {
                            *f += 6;
                            if *f >= METER_MAX {
                                first_up = false;
                            }
                        } else {
                            *f -= 6;
                            if *f <= 0 {
                                first = None;
                                first_up = true;
                            }
                        }
                    }
                }
                if let Some(s) = second.as_mut() {
                    *s += 6;
                    if *s >= METER_MAX {
                        second = None;
                        first = None;
                        first_up = true;
                    }
                }
                let mut done_first = done_first;
                if pads.pressed(0, input::TRIANGLE) {
                    match (first, done_first, second) {
                        (None, None, _) => {
                            g.effects.push((124, crate::sound::EFFECTS_VOLUME));
                            first = Some(6);
                            first_up = true;
                        }
                        (Some(f), None, None) => {
                            g.effects.push((125, crate::sound::EFFECTS_VOLUME));
                            done_first = Some(f);
                            second = Some(METER_MAX - f);
                        }
                        (_, Some(f), Some(s)) => {
                            g.effects.push((125, crate::sound::EFFECTS_VOLUME));
                            // 0x80058cb8: the readings pick the launch.
                            let heading = match s {
                                1..=171 => 2944,
                                172..=195 => 3072,
                                _ => 3200,
                            };
                            let forward = match f {
                                1..=170 => 2176,
                                171..=194 => 2816,
                                _ => 3328,
                            };
                            self.step = Step::Launched { good: heading == 3072 && forward == 2816 };
                            self.launch = (heading, forward);
                            g.cine = Some(LEAP);
                            return;
                        }
                        _ => {}
                    }
                }
                self.step = Step::Meter { first, first_up, second, done_first };
            }
            Step::Launched { good } => {
                if Self::landed(g) && g.player.v == [0; 3] && self.flown {
                    self.flown = false;
                    g.fixed = None;
                    g.cine = Some(if good {
                        WON
                    } else {
                        LOSE3[(3 - g.tries.clamp(1, 3)) as usize]
                    });
                    self.step = Step::Waiting;
                }
            }
            Step::Waiting => {}
        }
    }

    fn timer_expired(&mut self, g: &mut Game) {
        // 0x8007dc44: ring 1's Lose cine.
        g.cine = Some(LOSE1);
        self.step = Step::Waiting;
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        match n {
            LOSE1 => {
                if self.lose_try(g) {
                    self.ring1(g);
                }
            }
            WIN1 => {
                // 0x8007da6c, the swap to the tricycle.
                g.hide(NET, false);
                let at = find(g, TRIKE).map_or(g.player.pos, |k| g.npcs[k].pos);
                g.hide(TRIKE, true);
                self.become_body(g, 1, at, 1343);
                g.cine = Some(TRIKE_INTRO);
            }
            TRIKE_INTRO => {
                // 0x8007dadc: Tommy away; ring 2 from here, phase 2.
                g.hide(TOMMY, true);
                self.phase = 2;
                self.respawn = (g.player.pos, g.player.heading);
                g.checkpoint = Some(self.respawn);
                g.song = Some(0);
                self.step = Step::Ride;
            }
            LOSE2 => {
                if self.lose_try(g) {
                    let (at, h) = self.respawn;
                    self.become_body(g, 1, at, h);
                    self.step = Step::Ride;
                }
            }
            WIN2 => {
                let at = find(g, ANGELICA).map_or(TOWER, |k| g.npcs[k].pos);
                g.hide(ANGELICA, true);
                self.become_body(g, 2, at, 3072);
                g.cine = Some(RING3_INTRO);
            }
            RING3_INTRO => self.ring3(g),
            LEAP => {
                // Cine 39's end: Chuckie is the player, launched (state 26,
                // (0, 2 x -1000, forward) turned by the heading).
                g.hide(CHUCKIE, true);
                let (heading, forward) = self.launch;
                self.become_body(g, 3, [BOARD_END[0], BOARD_END[1] - 200, BOARD_END[2]], heading);
                g.player.launch(heading, -2000, forward, &g.trig);
                self.flown = true;
            }
            n if LOSE3.contains(&n) => {
                if self.lose_try(g) {
                    self.ring3(g);
                }
            }
            _ => {}
        }
    }

    /// ForceCry's level 11 case (0x80080044): no cry, a try.
    fn cry(&mut self, g: &mut Game) -> bool {
        if self.lose_try(g) {
            match self.phase {
                1 => self.ring1(g),
                3 => self.ring3(g),
                _ => {
                    let (at, h) = self.respawn;
                    self.become_body(g, 1, at, h);
                    self.step = Step::Ride;
                }
            }
        }
        true
    }

    fn draw(&self, g: &Game, _moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        let Some(s) = &g.sprites else { return };
        let f = g.focal();
        match self.step {
            // The speed bar (0x8008d3b8: 356 over 355 at (170, 76)).
            Step::Ride | Step::Ramp(_) => {
                let top = 2.0 * 32.0 * (1.0 - (g.player.speed as f32 / 20.0).min(1.0));
                s.draw(355, (170, 76), 64.0, 32.0, 1024.0, top, f, overlay);
                s.draw(356, (170, 85), 64.0, 8.0, 1024.0, 0.0, f, overlay);
            }
            // The meter (0x8008cef0: 357, the bar 358).
            Step::Meter { first, second, done_first, .. } => {
                s.draw(357, (0, 76), 110.0, 12.0, 512.0, 0.0, f, overlay);
                let v = match (done_first, second) {
                    (Some(_), Some(s2)) => METER_MAX - s2,
                    _ => first.unwrap_or(0),
                };
                if v >= 2 {
                    let half = 105.0 * v as f32 / METER_MAX as f32;
                    let x = ((105.0 - half) * f / 512.0 * (512.0 / 320.0)) as i16;
                    s.draw(358, (x, 76), half, 8.0, 512.0, 0.0, f, overlay);
                }
            }
            _ => {}
        }
    }

    fn phase(&self) -> u8 {
        self.phase
    }

    fn describe(&self) -> String {
        format!("{} ", self.bones) + &format!("phase {} step {:?} bones {}", self.phase, self.step, self.bones)
    }
}
