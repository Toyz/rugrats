//! Characters the game moves (SLUS_006.50:0x80084084 and friends): their
//! think picks a goal by mode and steers toward it (0x80082264), at a
//! speed from the table at 0x800b9698; the posts they walk come from LP
//! kind 2 (the 28-byte table at 0x800c4b9c).
//!
//! The original runs NPCs through the player's own state machine and
//! physics; the port moves them along the floor with the same speed
//! units (speed * dt along the facing) and plays their run or idle.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::col::Col;
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::placement::Placement;

use crate::animator::Animator;
use crate::gte::Trig;
use crate::player::DT;

/// The speed table (0x800b9698), row 0.
pub const SPEEDS: [i32; 6] = [0, 4, 8, 12, 17, 25];
/// The most an NPC turns a step (0x800c4444).
const TURN: i32 = 512;
/// How near a post counts as reached (+0x690, 1024 at init).
pub const REACH: i32 = 1024;

/// A post (LP kind 2): position, neighbours (1-based, 0 none), flags.
#[derive(Clone, Copy, Debug)]
pub struct Post {
    pub pos: [i32; 3],
    pub next: [u8; 4],
    pub flags: u32,
}

pub mod post {
    pub const END: u32 = 1;
    pub const WAIT: u32 = 2;
    pub const RANDOM: u32 = 4;
    /// A loop post (0x80082ee0): every reload-th time, the second
    /// neighbour.
    pub const LOOP: u32 = 16;
    /// Reaching it posts GoalPost (0x80077ff0).
    pub const GOAL: u32 = 32;
    pub const JUMP: u32 = 64;
}

/// The level's posts, in LP order (post n is entry n - 1): the record's
/// +0x14 neighbours and +0x18 flags.
pub fn posts(placements: &[Placement]) -> Vec<Post> {
    placements
        .iter()
        .filter(|p| p.kind == 2)
        .map(|p| Post { pos: p.pos, next: p.flags.to_le_bytes(), flags: p.rotation[0] as u32 })
        .collect()
}

/// What an NPC goes after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Player,
    Npc(usize),
    /// A place (a thing lying there, a home).
    Point([i32; 3]),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    /// Standing (mode 8, or not scheduled).
    Idle,
    /// Mode 4: after the target, at speed when beyond range / 2, twice it
    /// beyond twice the range; close up, an attack when allowed.
    Follow(Target),
    /// Mode 6: post to post (previous, current).
    Posts { prev: u8, cur: u8 },
    /// Mode 7: waiting at a post for the player, frames left.
    Wait { prev: u8, cur: u8, left: i32 },
    /// Mode 11: guarding home, chasing the player within range.
    Guard,
    /// Mode 1: hit, recovering for `left` dt.
    Hit { left: i32, then: Box<Mode> },
    /// Riding another (0x80083634): its position less `up`, its heading
    /// plus `turn`.
    Ride { on: Target, up: i32, turn: i32 },
    /// Mode 9 (0x80084084): its posts, the player chased while near them
    /// and in range, bitten within twice its half size.
    Prowl { prev: u8, cur: u8 },
    /// Mode 3: keeps itself between the player and what it guards,
    /// facing `heading`; the player in range is swatted back.
    Block { heading: i32 },
}

pub struct Npc {
    /// The registry id (LP +0x2a) and the character (the hierarchy).
    pub id: u16,
    pub character: usize,
    pub hierarchy: Hierarchy,
    pub animator: Animator,
    pub pos: [i32; 3],
    pub heading: i32,
    pub speed: i32,
    pub speed_index: usize,
    /// Template +0x1a, +0x1c (VSync ticks), +0x32.
    pub range: i32,
    pub wait: i32,
    pub damage: i32,
    pub health: i32,
    pub max_health: i32,
    pub mode: Mode,
    pub home: [i32; 3],
    /// Not drawn (0x80078bc4 hides by id).
    pub hidden: bool,
    /// May attack (+0x688 bit 8), and the time until the next bite.
    pub attacks: bool,
    pub cooldown: i32,
    /// LP +0x40.
    pub first_post: u8,
    /// Loop posts (flag 16): the count left and its reload (LP +0x42).
    pub loops: (i32, i32),
    /// Half its size (the hierarchy's, which a level may grow).
    pub size: i32,
    /// How near a guard bites (+0x690; 600 by default, level 5's robots
    /// 1536).
    pub reach: i32,
    /// The time between a guard's bites (dt; the geese 900, level 5's
    /// robots 230 as seen).
    pub bite_wait: i32,
    /// A run speed other than the table's row 0 (levels 9 and 10 use row
    /// 1: index 4 is 11).
    pub run_speed: Option<i32>,
    /// How near a post counts as reached (+0x690: 1024; level 13's
    /// Angelica 1536).
    pub post_reach: i32,
}

/// What a step of the NPCs asks of the level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Damage(player, target, amount) from a bite (mode 5).
    Bite { by: usize, target: Target, amount: i32 },
    /// A post route ended (flag 1).
    RouteEnd(usize),
    /// A goal post was reached (flag 32, GoalPost 0x80077ff0).
    Goal(usize),
    /// Mode 3's swat (0x80084084): the player thrown along `heading`.
    Swat { by: usize, heading: i32 },
    /// Mode 7 entered at a post (0x80083e2c), and left because the player
    /// came near (0x80083efc).
    Waits(usize),
    Resumes(usize),
}

fn wrap(a: i32) -> i32 {
    (a + 2048).rem_euclid(4096) - 2048
}

impl Npc {
    pub fn feet(&self) -> i32 {
        self.hierarchy.parts.first().map_or(0, |r| r.offset[1])
    }

    pub fn rotation(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        Mat3::from_cols_array(&e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap())
    }

    pub fn place(&mut self, at: [i32; 3], col: &Col) {
        self.pos = at;
        self.snap(col);
    }

    fn snap(&mut self, col: &Col) {
        let q = crate::collide::query(col, [self.pos[0], self.pos[1] - self.feet() - 1, self.pos[2]]);
        if q.tile.is_some() && q.kind == 0 {
            self.pos[1] = q.floor + self.feet();
        }
    }

    /// Steers toward `goal` (0x800820e0) and moves at `speed`: a quarter
    /// of it while the turn left is over 1024.
    fn go(&mut self, goal: [i32; 3], speed: i32, col: &Col, trig: &Trig) {
        let (dx, dz) = ((goal[0] - self.pos[0]) as f64, (goal[2] - self.pos[2]) as f64);
        let want = ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32;
        let d = wrap(want - self.heading);
        self.heading = (self.heading + d.clamp(-TURN, TURN)).rem_euclid(4096);
        let speed = if wrap(want - self.heading).abs() > 1024 { speed / 4 } else { speed };
        self.speed = speed;
        let start = self.pos;
        self.pos[0] += trig.sin(self.heading) * speed * DT >> 12;
        self.pos[2] += trig.cos(self.heading) * speed * DT >> 12;
        // Asked from a step above the feet, as the player's floor test.
        let q = crate::collide::query(col, [self.pos[0], self.pos[1] - self.feet() - 384, self.pos[2]]);
        if q.kind == 2 || q.tile.is_none() {
            self.pos = start;
            self.speed = 0;
        } else if q.floor - (self.pos[1] - self.feet()) > -1200 {
            self.pos[1] = q.floor + self.feet();
        } else {
            // A wall: back across.
            self.pos = start;
            self.speed = 0;
        }
    }

    fn dist(&self, p: [i32; 3]) -> i32 {
        let (dx, dz) = ((p[0] - self.pos[0]) as f64, (p[2] - self.pos[2]) as f64);
        (dx * dx + dz * dz).sqrt() as i32
    }
}

/// The positions and headings of the player and every NPC, for targets.
pub struct World<'a> {
    pub player: [i32; 3],
    pub player_heading: i32,
    pub col: &'a Col,
    pub posts: &'a [Post],
    pub trig: &'a Trig,
    pub anims: &'a Anims,
}

/// One frame of every NPC (the scheduler running each think, 0x80072bf0).
pub fn step(npcs: &mut [Npc], w: &World, rand: &mut crate::rand::Rand) -> Vec<Event> {
    let mut events = Vec::new();
    let places: Vec<([i32; 3], i32)> = npcs.iter().map(|n| (n.pos, n.heading)).collect();
    let at = |t: Target| match t {
        Target::Player => (w.player, w.player_heading),
        Target::Npc(k) => places.get(k).copied().unwrap_or((w.player, 0)),
        Target::Point(p) => (p, 0),
    };
    for (k, n) in npcs.iter_mut().enumerate() {
        n.cooldown = (n.cooldown - DT).max(0);
        let base = n.run_speed.unwrap_or(SPEEDS[n.speed_index.min(5)]);
        let mode = n.mode.clone();
        match mode {
            Mode::Idle => n.speed = 0,
            Mode::Ride { on, up, turn } => {
                let (p, h) = at(on);
                n.pos = [p[0], p[1] - up, p[2]];
                n.heading = (h + turn).rem_euclid(4096);
                n.speed = 0;
            }
            Mode::Follow(t) => {
                let (p, _) = at(t);
                let d = n.dist(p);
                if d > 2 * n.range {
                    n.go(p, 2 * base, w.col, w.trig);
                } else if d >= n.range / 2 {
                    n.go(p, base, w.col, w.trig);
                } else {
                    n.speed = 0;
                    if n.attacks && n.cooldown == 0 && (p[1] - n.pos[1]).abs() < 1200 {
                        events.push(Event::Bite { by: k, target: t, amount: n.damage });
                        n.cooldown = n.bite_wait.min(600);
                    }
                }
            }
            Mode::Posts { prev, cur } => {
                let Some(post) = w.posts.get((cur as usize).wrapping_sub(1)).copied() else {
                    n.mode = Mode::Idle;
                    continue;
                };
                if n.dist(post.pos) <= n.post_reach {
                    if post.flags & post::GOAL != 0 {
                        events.push(Event::Goal(k));
                    }
                    if post.flags & post::END != 0 || post.next == [0; 4] {
                        n.mode = Mode::Idle;
                        events.push(Event::RouteEnd(k));
                        continue;
                    }
                    let next = n.next_post(&post, prev, rand);
                    n.mode = if post.flags & post::WAIT != 0 {
                        events.push(Event::Waits(k));
                        Mode::Wait { prev: cur, cur: next, left: n.wait * DT / 2 }
                    } else {
                        Mode::Posts { prev: cur, cur: next }
                    };
                } else {
                    n.go(post.pos, base, w.col, w.trig);
                }
            }
            Mode::Wait { prev, cur, left } => {
                // Resume when the time is up or the player comes near
                // (0x80083efc).
                n.speed = 0;
                let near = n.dist(w.player) < n.range;
                if left <= 0 || near {
                    if near {
                        events.push(Event::Resumes(k));
                    }
                    n.mode = Mode::Posts { prev, cur };
                } else {
                    n.mode = Mode::Wait { prev, cur, left: left - DT };
                    let p = w.player;
                    let (dx, dz) = ((p[0] - n.pos[0]) as f64, (p[2] - n.pos[2]) as f64);
                    let want = ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32;
                    n.heading = (n.heading + wrap(want - n.heading).clamp(-TURN, TURN)).rem_euclid(4096);
                }
            }
            Mode::Guard => {
                let p = w.player;
                let from_home = {
                    let (dx, dz) = ((p[0] - n.home[0]) as f64, (p[2] - n.home[2]) as f64);
                    (dx * dx + dz * dz).sqrt() as i32
                };
                if from_home < n.range && (p[1] - n.pos[1]).abs() < 2800 {
                    if n.dist(p) < n.reach {
                        n.speed = 0;
                        if n.cooldown == 0 && (p[1] - n.pos[1]).abs() < 1200.max(n.reach + 600) {
                            events.push(Event::Bite { by: k, target: Target::Player, amount: n.damage });
                            n.cooldown = n.bite_wait;
                        }
                    } else {
                        n.go(p, base, w.col, w.trig);
                    }
                } else if n.dist(n.home) > 600 {
                    let home = n.home;
                    n.go(home, base, w.col, w.trig);
                } else {
                    n.speed = 0;
                }
            }
            Mode::Prowl { prev, cur } => {
                let Some(post) = w.posts.get((cur as usize).wrapping_sub(1)).copied() else {
                    n.mode = Mode::Idle;
                    continue;
                };
                let p = w.player;
                let reach = 2 * n.half_size().max(1);
                if n.dist(p) <= reach && (p[1] - n.pos[1]).abs() < 1200 {
                    // Mode 5: the bite, then mode 1.
                    n.speed = 0;
                    events.push(Event::Bite { by: k, target: Target::Player, amount: n.damage });
                    n.mode = Mode::Hit { left: 900, then: Box::new(Mode::Prowl { prev, cur }) };
                } else if n.dist(post.pos) <= 2 * n.range && n.dist(p) <= n.range {
                    n.go(p, base, w.col, w.trig);
                } else if n.dist(post.pos) <= n.post_reach {
                    let next = n.next_post(&post, prev, rand);
                    n.mode = Mode::Prowl { prev: cur, cur: next };
                } else {
                    n.go(post.pos, base, w.col, w.trig);
                }
            }
            Mode::Block { heading } => {
                let p = w.player;
                let dist = n.dist(p);
                let higher = p[1] + 600 < n.pos[1] - n.feet();
                if dist <= n.range && !higher {
                    n.speed = 0;
                    events.push(Event::Swat { by: k, heading });
                    n.mode = Mode::Hit { left: 1200, then: Box::new(Mode::Block { heading }) };
                } else {
                    // Sideways along its facing's normal toward the side the
                    // player is on (0x80082230), facing kept.
                    let (s, c) = (w.trig.sin(heading), w.trig.cos(heading));
                    let side = ((p[0] - n.pos[0]) as i64 * c as i64 - (p[2] - n.pos[2]) as i64 * s as i64) >> 12;
                    if side.abs() > 200 {
                        let dir = if side > 0 { 1 } else { -1 };
                        let goal = [n.pos[0] + dir * c * 1024 / 4096, n.pos[1], n.pos[2] - dir * s * 1024 / 4096];
                        n.go(goal, base, w.col, w.trig);
                    } else {
                        n.speed = 0;
                    }
                    n.heading = heading;
                }
            }
            Mode::Hit { left, then } => {
                n.speed = 0;
                n.mode = if left <= 0 { *then } else { Mode::Hit { left: left - DT, then } };
            }
        }
        let a = if n.speed > 0 { crate::player::anim::RUN } else { crate::player::anim::IDLE };
        n.animator.set(a, w.anims);
        n.animator.step(w.anims);
    }
    events
}

impl Npc {
    /// The larger of the hierarchy's half sizes (+0x08, +0x10).
    pub fn half_size(&self) -> i32 {
        self.size
    }

    /// The next post (0x80082c78), with loop posts counted (0x80082ee0):
    /// at one with the count spent, it is reloaded and the second
    /// neighbour taken.
    fn next_post(&mut self, p: &Post, prev: u8, rand: &mut crate::rand::Rand) -> u8 {
        if p.flags & post::LOOP != 0 && self.loops.1 > 0 {
            if self.loops.0 <= 0 {
                self.loops.0 = self.loops.1;
                if p.next[1] != 0 {
                    return p.next[1];
                }
            } else {
                self.loops.0 -= 1;
            }
        }
        next_post(p, prev, rand)
    }
}

/// The next post (0x80082c78): a random neighbour with flag 4, else the
/// first neighbour that is not the one come from, else neighbour 0.
fn next_post(p: &Post, prev: u8, rand: &mut crate::rand::Rand) -> u8 {
    let real: Vec<u8> = p.next.iter().copied().filter(|&n| n != 0).collect();
    if real.is_empty() {
        return prev;
    }
    if p.flags & post::RANDOM != 0 {
        return real[rand.next() as usize % real.len()];
    }
    real.iter().copied().find(|&n| n != prev).unwrap_or(real[0])
}

/// The NPC's triangles.
pub fn triangles(n: &Npc, models: &rugrats_data::model::Models, anims: &Anims, trig: &Trig, out: &mut Vec<rugrats_render::Vtx>) {
    if n.hidden {
        return;
    }
    let pos = Vec3::new(n.pos[0] as f32, n.pos[1] as f32, n.pos[2] as f32);
    rugrats_render::models::posed_triangles(&n.hierarchy, models, &n.animator.locals(&n.hierarchy, anims), n.rotation(trig), pos, out);
}
