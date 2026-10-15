//! Mini golf, levels 23-31 (docs/levels/golf.md): walk up to the ball,
//! aim with Left and Right, Triangle starts the power meter and Triangle
//! again swings. The ball rolls on the floor (0x80087c5c with flags
//! 0x19030000) until it stops or drops in the cup. At or under par the
//! next hole follows; over par, a cry and the hole again. The ninth hole
//! plays cine 35 and records level 8.

use rrt::glam::{Mat3, Vec3};

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::player::DT;

/// Hole index (play order) to level (0x80057dcc), par (0x800b7ffc) and the
/// baby who plays it (0x800b7ff0).
pub const LEVELS: [usize; 9] = [25, 28, 24, 29, 23, 30, 26, 31, 27];
/// Level 23's ball-reset object (0x800c4168).
const RESET_ZONE: u32 = 1264;
pub const PAR: [u32; 9] = [4, 5, 4, 5, 5, 5, 4, 5, 3];
const BABY: [usize; 9] = [1, 2, 3, 5, 4, 1, 2, 3, 1];
/// Where the player starts (0x8007a584).
const STARTS: [[i32; 3]; 9] = [
    [35382, 11, 12372],
    [38387, -124, 8978],
    [35197, 215, 2194],
    [53580, -142, -9917],
    [43692, -138, -15253],
    [37468, 11, -10897],
    [35502, -98, 12393],
    [36284, -215, -9095],
    [43693, -2037, 11334],
];
/// The other babies placed in the holes, hidden (cines 31 and 33).
const OTHERS: [u16; 5] = [15, 600, 608, 609, 1776];
/// Templates: the ball and the cup (0x800494d8).
const BALL: u16 = 724;
const CUP: u16 = 789;
/// Animations: walk-in (29), the stance (30), the stroke (31).
const WALK_IN: i16 = 29;
const STANCE: i16 = 30;
const STROKE: i16 = 31;
/// The walk-up reach (0x80058538: d^2 <= 0xfffff).
const REACH2: i64 = 0xfffff;
/// The meter: 1 idle, 4..220, 3 a frame.
const POWER_MAX: i32 = 220;
/// Aim steps a frame (0x8005be24): 25, 8 with Square.
const AIM: i32 = 25;
const AIM_FINE: i32 = 8;
/// The roll: radius 104, friction (98 dt) >> 6, bounce 3000 / 4096, the
/// cup's 384 and the speed it still takes (132 as a power).
const RADIUS: i32 = 104;
const FRICTION: i32 = (98 * DT) >> 6;
const BOUNCE: i32 = 3000;
const CUP_REACH2: i64 = 0x24000;
const CUP_SPEED: i32 = (132 << 16) / 6000;
/// The longest speed (power 220).
const TOP_SPEED: i32 = (220 << 16) / 6000;
/// The hit comes this many frames into the stroke (seen: 13), the
/// player is free again 7 later.
const HIT_AT: i32 = 13;
const STROKE_END: i32 = 20;
/// The walk-in's length when its animation does not say (seen: 56).
const WALK_IN_TIME: i32 = 56;
/// Strings: "Par %d" (195), "Swing " (196), "You were over par." (214),
/// "Try Again" (232), "PRESS X TO CONTINUE" (67).
const PAR_TEXT: u32 = 195;
const SWING_TEXT: u32 = 196;
const OVER_PAR: u32 = 214;
const TRY_AGAIN: u32 = 232;
const PRESS: u32 = 67;
/// The HUD's yellow (0x13cdec).
const YELLOW: [u8; 3] = [0xec, 0xcd, 0x13];
/// The meter's sprites: the frame (293, half 110 x 12) and the bar (294,
/// half height 10), depth 512, at (0, 76).
const METER_FRAME: u32 = 293;
const METER_BAR: u32 = 294;
/// The golf camera (mode 12): eye (440, -768 - 1600, -4096) and aim (440,
/// -768, 0) in the player's frame.
const EYE: Vec3 = Vec3::new(440.0, -2368.0, -4096.0);
const LOOK: Vec3 = Vec3::new(440.0, -768.0, 0.0);
/// The end: cine 35 "Golf Ending", recorded as level 8.
const CINE_END: usize = 35;
const RECORD: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    /// Walking about (state 0).
    Free,
    /// State 13: into the stance, frames so far.
    WalkIn(i32),
    /// State 14: power (1 idle) and whether it falls.
    Aim { power: i32, down: bool },
    /// State 15: frames into the stroke, the power.
    Stroke(i32, i32),
    /// Holed: the over-par box up (waiting for Cross), or done.
    Over,
    Done,
}

pub struct Golf {
    hole: usize,
    strokes: u32,
    stage: Stage,
    ball: [i32; 3],
    /// The roll's velocity, and whether it lipped out this roll.
    roll: Option<[i32; 3]>,
    lipped: bool,
    cup: [i32; 3],
    ball_thing: Option<usize>,
    strings: [String; 5],
    /// The holing-out line tables (0x8007a9d8), by address.
    lines: Vec<(u32, Vec<u16>)>,
    /// Activity play: the bodies a turn may take (characters 1-5 and
    /// Susie, 37), the ball's tee, and "Player %d" / "Hole %d" (203, 204)
    /// with the turn's player and colour.
    bodies: Vec<(usize, rugrats_data::hierarchy::Hierarchy, crate::animator::Animator)>,
    tee: [i32; 3],
    turn_strings: [String; 2],
    turn: Option<(usize, [u8; 3])>,
}

/// Activity play's characters, by face slot (0x800bb4ac).
pub const FACES: [usize; 6] = [1, 2, 3, 5, 4, 37];
/// The players' colours (0x800bb494): red, green, blue, yellow.
pub const PLAYER_COLOURS: [[u8; 3]; 4] = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]];

/// The holing-out tables: over par by character, the last try, the
/// last hole, under par by character.
const LINE_TABLES: [(u32, usize); 11] = [
    (0x800b_9250, 8),
    (0x800b_9260, 7),
    (0x800b_9270, 10),
    (0x800b_9298, 7),
    (0x800b_9284, 9),
    (0x800b_92d8, 2),
    (0x800b_92d0, 3),
    (0x800b_92a8, 5),
    (0x800b_92b4, 4),
    (0x800b_92bc, 3),
    (0x800b_92c4, 5),
];

/// trunc(a / b) toward zero.
fn tdiv(a: i64, b: i64) -> i32 {
    (a / b) as i32
}

fn isqrt(v: i64) -> i32 {
    (v.max(0) as f64).sqrt() as i32
}

impl Golf {
    /// EpisodeStart (0x8007a584) and the player init (0x80057e50).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<Golf, String> {
        let hole = LEVELS.iter().position(|&l| l == g.level).unwrap_or(0);
        g.record = g.level;
        // The hole's baby (0x8005928c).
        let baby = BABY[hole];
        if baby != 1 {
            if let Some(Some(h)) = g.set.hierarchies.get(baby).cloned() {
                let a = crate::animator::Animator::load(disc, baby as u16)?;
                g.player.set_body(h, a, crate::player::TURN, &g.anims);
            }
        }
        // In activity play only holes 0 and 8 hide the others (0x80096a30).
        if !g.activity || matches!(hole, 0 | 8) {
            for id in OTHERS {
                g.hide(id, true);
            }
        }
        g.player.pos = STARTS[hole];
        g.player.no_jump = true;
        let find = |t: u16| g.set.placements.iter().find(|p| p.kind == 1 && p.template as u16 == t).map(|p| p.pos);
        let ball = find(BALL).unwrap_or(STARTS[hole]);
        let cup = find(CUP).unwrap_or(ball);
        let ball_thing = g.things.iter().position(|t| t.template == BALL);
        if let Some(k) = ball_thing {
            g.things[k].hidden = true;
        }
        let string = |n: u32| crate::front::string(disc, n);
        let strings = [string(PAR_TEXT), string(SWING_TEXT), string(OVER_PAR), string(TRY_AGAIN), string(PRESS)];
        let lines = LINE_TABLES
            .iter()
            .map(|&(at, n)| {
                let b = disc.exe_bytes(at, 2 * n).unwrap_or(&[]);
                (at, b.chunks_exact(2).map(|w| u16::from_le_bytes([w[0], w[1]])).collect())
            })
            .collect();
        let mut bodies = Vec::new();
        if g.activity {
            for c in FACES {
                if let Some(Some(h)) = g.set.hierarchies.get(c).cloned() {
                    if let Ok(a) = crate::animator::Animator::load(disc, c as u16) {
                        bodies.push((c, h, a));
                    }
                }
            }
        }
        let turn_strings = [string(203), string(204)];
        let mut golf = Golf {
            hole,
            strokes: 0,
            stage: Stage::Free,
            ball,
            roll: None,
            lipped: false,
            cup,
            ball_thing,
            strings,
            lines,
            bodies,
            tee: ball,
            turn_strings,
            turn: None,
        };
        golf.settle(g);
        Ok(golf)
    }

    /// The ball on the floor: y = floor - radius.
    fn settle(&mut self, g: &Game) {
        let q = crate::collide::query(&g.col, [self.ball[0], self.ball[1] - 1, self.ball[2]]);
        if q.tile.is_some() && q.kind == 0 {
            self.ball[1] = q.floor - RADIUS;
        }
    }

    /// The stance (0x8005bac4): `r` behind the ball along the heading
    /// turned by `off`, and 128 to the side.
    fn stance(&self, g: &mut Game, r: i32, off: i32) {
        let a = (g.player.heading + off).rem_euclid(4096);
        let (s, c) = (g.trig.sin(a) as i64, g.trig.cos(a) as i64);
        let (s2, c2) = (g.trig.sin((a - 1024).rem_euclid(4096)) as i64, g.trig.cos((a - 1024).rem_euclid(4096)) as i64);
        g.player.pos[0] = self.ball[0] + tdiv(r as i64 * -s, 4096) + tdiv(-s2, 32);
        g.player.pos[2] = self.ball[2] + tdiv(r as i64 * -c, 4096) + tdiv(-c2, 32);
    }

    /// The golf camera: behind and above the player's frame.
    fn camera(&self, g: &mut Game) {
        let rot = g.player.rotation(&g.trig);
        let p = Vec3::new(g.player.pos[0] as f32, g.player.pos[1] as f32, g.player.pos[2] as f32);
        let at = |o: Vec3| p + rot * o;
        g.fixed = Some((at(EYE), at(LOOK)));
    }

    /// The hit (0x80058278): along the heading at k = (power << 16) /
    /// 6000.
    fn hit(&mut self, g: &mut Game, power: i32) {
        let k = (power << 16) / 6000;
        let h = g.player.heading;
        let (s, c) = (g.trig.sin(h), g.trig.cos(h));
        self.roll = Some([s * k >> 12, 0, c * k >> 12]);
        self.lipped = false;
        self.strokes += 1;
        g.effects.push((118, crate::sound::EFFECTS_VOLUME));
    }

    /// One frame of the roll (0x80087c5c, 0x80086ef0, 0x80086cd0) and the
    /// ball's think (0x800580a8). Whether it still rolls.
    fn roll(&mut self, g: &mut Game) -> bool {
        let Some(mut v) = self.roll else { return false };
        let start = self.ball;
        // The move, an axis at a time so a wall turns the ball back off
        // it (reflected, times 3000 / 4096).
        for k in [0, 2] {
            let mut p = self.ball;
            p[k] += v[k] * DT >> 6;
            let q = crate::collide::query(&g.col, [p[0], p[1], p[2]]);
            // Objects stop it too (0x80086340): their boxes.
            let object = g.boxes.iter().any(|b| b.covers(p, RADIUS) && b.min_y < p[1] + RADIUS && b.max_y > p[1] - RADIUS);
            let wall = object || q.kind != 0 || q.tile.is_none() || q.floor < start[1] + RADIUS - 384;
            if wall {
                v[k] = -(v[k] * BOUNCE >> 12);
                g.effects.push((137, crate::sound::EFFECTS_VOLUME));
            } else {
                self.ball = p;
            }
        }
        // Rolling (0x80086cd0): on the floor, friction 15 a frame, the
        // slope's push downhill.
        let q = crate::collide::query(&g.col, [self.ball[0], self.ball[1], self.ball[2]]);
        let Some((cx, k)) = q.tile.filter(|_| q.kind == 0) else {
            self.roll = None;
            return self.stopped(g);
        };
        self.ball[1] = q.floor - RADIUS;
        let n = g
            .col
            .columns
            .get(cx)
            .and_then(|c| c.tiles.get(k))
            .and_then(|t| g.col.floor_normals.get(t[1] as usize).filter(|_| t[1] != 0))
            .map_or([0, -4096, 0], |n| n.map(|v| v as i32));
        v[1] = 0;
        let s = isqrt(v[0] as i64 * v[0] as i64 + v[2] as i64 * v[2] as i64);
        let push = slope_push(n[1]);
        if push == 0 && s <= FRICTION {
            self.roll = None;
            return self.stopped(g);
        }
        if s > 0 {
            v[0] = tdiv(v[0] as i64 * (s - FRICTION) as i64, s as i64);
            v[2] = tdiv(v[2] as i64 * (s - FRICTION) as i64, s as i64);
        }
        if push != 0 {
            let h = isqrt(n[0] as i64 * n[0] as i64 + n[2] as i64 * n[2] as i64).max(1);
            v[0] += tdiv(n[0] as i64 * push as i64, h as i64);
            v[2] += tdiv(n[2] as i64 * push as i64, h as i64);
            let s = isqrt(v[0] as i64 * v[0] as i64 + v[2] as i64 * v[2] as i64);
            if s <= FRICTION {
                self.roll = None;
                return self.stopped(g);
            }
        }
        // The cup (0x800580a8): within 384 (2D), slow enough drops; faster
        // lips out at half speed.
        let (dx, dz) = ((self.cup[0] - self.ball[0]) as i64, (self.cup[2] - self.ball[2]) as i64);
        if !self.lipped && dx * dx + dz * dz <= CUP_REACH2 {
            let s = isqrt(v[0] as i64 * v[0] as i64 + v[2] as i64 * v[2] as i64);
            if s > CUP_SPEED {
                g.effects.push((119, crate::sound::EFFECTS_VOLUME));
                self.lipped = true;
                v = lip_out(v, [dx as i32, dz as i32], s);
            } else {
                g.effects.push((120, crate::sound::EFFECTS_VOLUME));
                self.roll = None;
                self.ball = [self.cup[0], self.cup[1], self.cup[2]];
                self.holed(g);
                return false;
            }
        }
        self.roll = Some(v);
        true
    }

    /// The roll ended: back to the follow camera; the next swing.
    fn stopped(&mut self, g: &mut Game) -> bool {
        // A ball that stops within the cup's reach drops.
        let (dx, dz) = ((self.cup[0] - self.ball[0]) as i64, (self.cup[2] - self.ball[2]) as i64);
        if dx * dx + dz * dz <= CUP_REACH2 {
            g.effects.push((120, crate::sound::EFFECTS_VOLUME));
            self.holed(g);
            return false;
        }
        self.lipped = false;
        g.fixed = None;
        g.player.frozen = false;
        false
    }

    fn line(&self, at: u32, k: u32) -> u16 {
        self.lines.iter().find(|t| t.0 == at).and_then(|t| t.1.get(k as usize)).copied().unwrap_or(0)
    }

    /// The baby's lines on holing out (0x8007a9d8): over par, under par,
    /// the last hole. Only the first free one is heard.
    fn say(&self, g: &mut Game) {
        let (s, par) = (self.strokes, PAR[self.hole]);
        let mut r = || g.rand.next() & 255;
        let mut lines = Vec::new();
        if s > par {
            match g.character {
                1 => {
                    if s == par + 2 {
                        lines.push(72);
                    }
                    lines.push(self.line(0x800b_9250, r() & 7));
                }
                2 => {
                    lines.extend((s == par + 1).then_some(336).or((s == par + 2).then_some(337)));
                    lines.push(self.line(0x800b_9260, r() % 7));
                }
                3 => {
                    lines.extend((s == par + 1).then_some(587).or((s == par + 2).then_some(588)));
                    lines.push(self.line(0x800b_9270, r() % 10));
                }
                4 => {
                    if s == par + 2 {
                        lines.push(938);
                    }
                    lines.push(self.line(0x800b_9298, r() % 7));
                }
                5 => {
                    lines.extend((s == par + 1).then_some(809).or((s == par + 2).then_some(810)));
                    lines.push(self.line(0x800b_9284, r() % 9));
                }
                _ => {}
            }
            if g.tries == 1 {
                lines.push(self.line(0x800b_92d8, r() & 1));
            }
        } else if self.hole == 8 {
            lines.push(self.line(0x800b_92d0, r() % 3));
        } else {
            let line = match g.character {
                1 => (s == 1).then_some(70),
                2 => Some(if s + 1 == par { 338 } else { self.line(0x800b_92a8, r() % 5) }),
                3 => Some(match s {
                    1 => 586,
                    _ if s + 1 == par => 589,
                    _ => self.line(0x800b_92b4, r() & 3),
                }),
                5 => Some(match s {
                    1 => 812,
                    _ if s + 1 == par => 811,
                    _ => self.line(0x800b_92bc, r() % 3),
                }),
                4 => Some(if s + 1 == par { 939 } else { self.line(0x800b_92c4, r() % 5) }),
                _ => None,
            };
            lines.extend(line);
        }
        g.speech.extend(lines);
    }

    /// Holing out (0x8007a9d8).
    fn holed(&mut self, g: &mut Game) {
        g.fixed = None;
        // Activity play (0x8007a9d8 step 1): the turn ends (SwitchEpisode
        // to this level), nothing judged or recorded; the round driver
        // takes the next player or hole.
        if g.activity {
            self.stage = Stage::Done;
            g.strokes = Some(self.strokes);
            g.exit = Some((g.level, None));
            return;
        }
        self.say(g);
        if self.strokes > PAR[self.hole] {
            // Over par: the box (0x80043968), then a cry and the hole
            // again.
            self.stage = Stage::Over;
            let line = |k: usize, c| (self.strings[k].clone(), c);
            g.pages.push(crate::screens::Page::Message {
                lines: vec![line(2, crate::screens::PINK), line(3, crate::screens::PINK)],
                bars: None,
                blink: false,
            });
            return;
        }
        self.stage = Stage::Done;
        if self.strokes == 1 {
            // A hole in one: a bar at the end of the level (+0x30 bit
            // 1024, 0x80094b10: bit 31).
            g.found.push(31);
        }
        if self.hole == 8 {
            g.record = RECORD;
            g.cine = Some(CINE_END);
        } else {
            g.exit = Some((LEVELS[self.hole + 1], None));
        }
    }

    fn walk_up(&mut self, g: &mut Game) {
        let p = g.player.pos;
        let (dx, dz) = ((self.ball[0] - p[0]) as i64, (self.ball[2] - p[2]) as i64);
        if dx * dx + dz * dz > REACH2 {
            return;
        }
        // 0x8005ba44: face the ball, then 1024 behind it.
        let want = (((dx as f64).atan2(dz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
        g.player.heading = want.rem_euclid(4096);
        g.player.target = g.player.heading;
        g.player.frozen = true;
        g.player.speed = 0;
        g.player.hold = Some(WALK_IN);
        self.stance(g, 1024, 0);
        self.stage = Stage::WalkIn(0);
    }
}

/// The slope's push (0x80086a20) from the floor normal's y: 0 flat, 1190
/// at 768 or steeper, linear between.
fn slope_push(ny: i32) -> i32 {
    let t = ((-(ny as f64) / 4096.0).clamp(-1.0, 1.0).asin() / std::f64::consts::TAU * 4096.0).round() as i32;
    let slope = 1024 - t;
    if slope >= 768 {
        1190
    } else if slope <= 0 {
        0
    } else {
        ((((99 * slope) >> 6) + 2) * DT) >> 6
    }
}

/// A lip-out (0x80057f78): straight back when it came at the cup head on,
/// else turned off it; half speed.
fn lip_out(v: [i32; 3], to_cup: [i32; 2], s: i32) -> [i32; 3] {
    let yaw = |x: f64, z: f64| (x.atan2(z) / std::f64::consts::TAU * 4096.0).round() as i32;
    let cup = yaw(to_cup[0] as f64, to_cup[1] as f64);
    let moving = yaw(v[0] as f64, v[2] as f64);
    let d = (cup - moving + 2048).rem_euclid(4096) - 2048;
    let out = if d.abs() < 256 {
        cup + 2048
    } else {
        let turn = if s >= TOP_SPEED { 0 } else { (((-d << 6) / TOP_SPEED) * s >> 6) + d };
        moving + 2 * turn
    };
    let a = out as f64 / 4096.0 * std::f64::consts::TAU;
    [(a.sin() * s as f64 / 2.0) as i32, v[1], (a.cos() * s as f64 / 2.0) as i32]
}

impl crate::game::Script for Golf {
    /// State 15, the stroke.
    fn cue(&self) -> i32 {
        if matches!(self.stage, Stage::Stroke(..)) { 15 } else { 0 }
    }

    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        // Level 23's callback (0x80081758): a ball at rest within x +-2000,
        // z +-1000 of template 1264 goes to (x, 280, 2753).
        if g.level == 23 && self.roll.is_none() {
            if let Some(o) = g.set.placements.iter().find(|p| p.kind == 1 && p.template == RESET_ZONE).map(|p| p.pos) {
                if (self.ball[0] - o[0]).abs() < 2000 && (self.ball[2] - o[2]).abs() < 1000 {
                    self.ball = [self.ball[0], 280, 2753];
                    self.settle(g);
                }
            }
        }
        match self.stage {
            Stage::Free => {
                if self.roll.is_some() {
                    // The stick gives no speed while the ball rolls.
                    g.player.frozen = true;
                    if !self.roll(g) && self.stage == Stage::Free {
                        g.player.frozen = false;
                        g.fixed = None;
                    } else if let Some((eye, _)) = g.fixed {
                        let b = Vec3::new(self.ball[0] as f32, self.ball[1] as f32, self.ball[2] as f32);
                        g.fixed = Some((eye, b));
                    }
                } else {
                    self.walk_up(g);
                }
            }
            Stage::WalkIn(n) => {
                self.camera(g);
                if n + 1 >= WALK_IN_TIME {
                    self.stage = Stage::Aim { power: 1, down: false };
                    g.player.hold = Some(STANCE);
                } else {
                    self.stage = Stage::WalkIn(n + 1);
                }
            }
            Stage::Aim { mut power, mut down } => {
                // The meter (0x8005fb38): 3 a frame up to 220 and back to 1.
                if power != 1 {
                    power += if down { -3 } else { 3 };
                    if power >= POWER_MAX {
                        down = true;
                    }
                    if power <= 1 {
                        power = 1;
                        down = false;
                    }
                }
                if power == 1 {
                    // Aiming (0x8005be24): 25 a frame, 8 with Square.
                    let step = if pads.held(0, input::SQUARE) { AIM_FINE } else { AIM };
                    let turn = if pads.held(0, input::LEFT) {
                        -step
                    } else if pads.held(0, input::RIGHT) {
                        step
                    } else {
                        0
                    };
                    g.player.heading = (g.player.heading + turn).rem_euclid(4096);
                    g.player.target = g.player.heading;
                }
                if pads.pressed(0, input::TRIANGLE) {
                    if power == 1 {
                        power = 4;
                        down = false;
                    } else {
                        g.player.hold = Some(STROKE);
                        self.stage = Stage::Stroke(0, power);
                        return;
                    }
                }
                self.stance(g, 425, 1024);
                self.camera(g);
                self.stage = Stage::Aim { power, down };
            }
            Stage::Stroke(n, power) => {
                if n == HIT_AT {
                    self.hit(g, power);
                }
                if n >= STROKE_END {
                    g.player.hold = None;
                    self.stage = Stage::Free;
                } else {
                    self.stage = Stage::Stroke(n + 1, power);
                }
                if n > HIT_AT {
                    self.roll(g);
                }
            }
            Stage::Over => {
                // Back from the box.
                self.stage = Stage::Done;
                g.cry();
            }
            Stage::Done => {}
        }
    }

    fn describe(&self) -> String {
        format!("hole {} strokes {} {:?} ball {:?} roll {:?}", self.hole, self.strokes, self.stage, self.ball, self.roll)
    }

    /// The golf case of ForceCry: the hole again (0x8007aff0).
    fn cry(&mut self, g: &mut Game) -> bool {
        g.exit = Some((g.level, None));
        true
    }

    /// A turn of activity play (0x80096a30 step 6): player `player`'s
    /// pick plays the hole from the tee with an untouched ball.
    fn activity_turn(&mut self, g: &mut Game, player: usize, character: usize) {
        if let Some((_, h, a)) = self.bodies.iter().find(|b| b.0 == character).cloned() {
            g.player.set_body(h, a, crate::player::TURN, &g.anims);
        }
        g.character = character;
        g.player.pos = STARTS[self.hole];
        g.player.v = [0; 3];
        g.player.speed = 0;
        g.player.frozen = false;
        g.player.hold = None;
        g.fixed = None;
        self.strokes = 0;
        self.stage = Stage::Free;
        self.ball = self.tee;
        self.roll = None;
        self.lipped = false;
        self.settle(g);
        let h = g.player.heading;
        g.set_camera_yaw(h as f32);
        self.turn = Some((player, PLAYER_COLOURS[player % 4]));
    }

    fn draw(&self, g: &Game, moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        // "Player %d" (-230, 60) and "Hole %d" (-230, 80), font 0, the
        // player's colour, all turn long (0x80090004).
        if let Some((p, rgb)) = self.turn {
            for (k, (form, n)) in [(&self.turn_strings[0], p + 1), (&self.turn_strings[1], self.hole + 1)].into_iter().enumerate() {
                let t = crate::text::Text { font: 0, spacing: 1, rgb, x: -230, y: 60 + 20 * k as i16, string: form.replace("%d", &n.to_string()), ..Default::default() };
                let mut shadow = t.clone();
                shadow.rgb = [0, 1, 0];
                shadow.x += 2;
                shadow.y += 2;
                shadow.triangles(&g.fonts, overlay);
                t.triangles(&g.fonts, overlay);
            }
        }
        if let Some(t) = self.ball_thing.map(|k| &g.things[k]) {
            if let Some(m) = g.set.models.get(t.shape) {
                let pos = Vec3::new(self.ball[0] as f32, self.ball[1] as f32, self.ball[2] as f32);
                rugrats_render::models::model_triangles(m, Mat3::IDENTITY, pos, moving);
            }
        }
        // "Par %d", "Swing ", "%d" at y -100 (0x8007a584), the walk-up
        // shows the stroke about to be played.
        let shown = if matches!(self.stage, Stage::WalkIn(_) | Stage::Aim { .. }) { self.strokes + 1 } else { self.strokes };
        let text = |x: i16, y: i16, s: String, out: &mut Vec<rugrats_render::Vtx>| {
            let t = crate::text::Text { font: 1, spacing: 1, rgb: YELLOW, x, y, string: s, ..Default::default() };
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(&g.fonts, out);
            t.triangles(&g.fonts, out);
        };
        text(-192, -100, self.strings[0].replace("%d", &PAR[self.hole].to_string()), overlay);
        text(50, -100, self.strings[1].clone(), overlay);
        text(150, -100, shown.to_string(), overlay);
        // The power meter (0x8008cef0, 0x8008cd00): the bar grows from
        // the frame's right end to the left.
        if let (Stage::Aim { power, .. }, Some(s)) = (self.stage, &g.sprites) {
            s.draw(METER_FRAME, (0, 76), 110.0, 12.0, 512.0, 0.0, g.focal(), overlay);
            if power >= 2 {
                let half = 105.0 * power as f32 / POWER_MAX as f32;
                let x = ((105.0 - half) * g.focal() / 512.0 * (512.0 / 320.0)) as i16;
                s.draw(METER_BAR, (x, 76), half, 10.0, 512.0, 0.0, g.focal(), overlay);
            }
        }
    }
}
