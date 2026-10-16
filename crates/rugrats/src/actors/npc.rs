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
use rugrats_data::model::Models;
use rugrats_data::placement::Placement;
use rugrats_render::{Vtx, models};

use crate::actors::animator::{Animator, RootCurve};
use crate::actors::player::anim::{IDLE, RUN};
use crate::actors::player::{Box3, DT, Pose};
use crate::gte::Trig;
use crate::rand::Rand;
use crate::world::collide;

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
    /// Left in a state that does not walk (Mr. Friend's 19 after his
    /// return in level 9's phase 2): following, it stays put until the
    /// target is within a bite.
    pub rooted: bool,
    /// Its last step was taken back off a wall or an object (the bump,
    /// state 9).
    pub bumped: bool,
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
    /// An animation a level holds it in (Spike asleep).
    pub hold: Option<i16>,
    /// Its height by its animations on the ground.
    pub root: RootCurve,
}

/// What a step of the NPCs asks of the level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Damage(player, target, amount) from a bite (mode 5).
    Bite {
        by: usize,
        target: Target,
        amount: i32,
    },
    /// A post route ended (flag 1).
    RouteEnd(usize),
    /// A goal post was reached (flag 32, GoalPost 0x80077ff0).
    Goal(usize),
    /// Mode 3's swat (0x80084084): the player thrown along `heading`.
    Swat {
        by: usize,
        heading: i32,
    },
    /// Mode 7 entered at a post (0x80083e2c), and left because the player
    /// came near (0x80083efc).
    Waits(usize),
    Resumes(usize),
}

fn wrap(a: i32) -> i32 {
    (a + 2048).rem_euclid(4096) - 2048
}

impl Npc {
    /// Loses `amount` health, to 0 at least.
    pub fn hurt(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
    }

    pub fn feet(&self) -> i32 {
        self.hierarchy.parts.first().map_or(0, |r| r.offset[1])
    }

    pub fn rotation(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        Mat3::from_cols_array(
            &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
    }

    pub fn place(&mut self, at: [i32; 3], col: &Col) {
        self.pos = at;
        self.snap(col);
    }

    fn snap(&mut self, col: &Col) {
        let q = collide::query(col, [self.pos[0], self.pos[1] - self.feet() - 1, self.pos[2]]);
        if q.tile.is_some() && q.kind == 0 {
            self.pos[1] = q.floor + self.feet();
        }
    }

    /// Steers toward `goal` (0x800820e0) and moves at `speed`: a quarter
    /// of it while the turn left is over 1024.
    fn go(&mut self, goal: [i32; 3], speed: i32, w: &World) {
        let (col, trig) = (w.col, w.trig);
        let (dx, dz) = ((goal[0] - self.pos[0]) as f64, (goal[2] - self.pos[2]) as f64);
        let want = ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32;
        let d = wrap(want - self.heading);
        self.heading = (self.heading + d.clamp(-TURN, TURN)).rem_euclid(4096);
        let speed = if wrap(want - self.heading).abs() > 1024 { speed / 4 } else { speed };
        self.speed = speed;
        self.bumped = false;
        let start = self.pos;
        self.pos[0] += (trig.sin(self.heading) * speed * DT) >> 12;
        self.pos[2] += (trig.cos(self.heading) * speed * DT) >> 12;
        // Asked from a step above the feet, as the player's floor test.
        let q = collide::query(col, [self.pos[0], self.pos[1] - self.feet() - 384, self.pos[2]]);
        if q.kind == 2 || q.tile.is_none() {
            self.pos = start;
            self.speed = 0;
            self.bumped = true;
        } else if !matches!(self.mode, Mode::Posts { .. } | Mode::Wait { .. }) && self.walled(start, w.walls) {
            // An object in the way (a shut door, a crate): back across.
            // Route walkers keep to their posts (guess: +0x688 bit 0x800,
            // set for the toss and race runners).
            self.pos = start;
            self.speed = 0;
            self.bumped = true;
        } else if q.floor - (self.pos[1] - self.feet()) > -1200 {
            self.pos[1] = q.floor + self.feet();
        } else {
            // A wall: back across.
            self.pos = start;
            self.speed = 0;
            self.bumped = true;
        }
    }

    /// Whether the body walked deeper into an object's box than it was at
    /// `start` (a box whose top is within a step of the feet is floor).
    fn walled(&self, start: [i32; 3], walls: &[Box3]) -> bool {
        let feet = self.pos[1] - self.feet();
        walls.iter().any(|b| {
            !b.character
                && b.min_y < feet - 384
                && b.max_y > feet - 600
                && b.covers(self.pos, 200)
                && b.depth(self.pos, 200) > b.depth(start, 200)
        })
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
    /// The objects' boxes this frame (doors, crates; not bodies).
    pub walls: &'a [Box3],
}

/// A prowler's bite to its next (the oracle's lobster, level 16).
const BITE_EVERY: i32 = 180;

/// One frame of every NPC (the scheduler running each think, 0x80072bf0).
pub fn step(npcs: &mut [Npc], w: &World, rand: &mut Rand) -> Vec<Event> {
    let mut events = Vec::new();
    let places: Vec<Pose> = npcs.iter().map(|n| Pose::new(n.pos, n.heading)).collect();
    let at = |t: Target| match t {
        Target::Player => Pose::new(w.player, w.player_heading),
        Target::Npc(k) => places.get(k).copied().unwrap_or(Pose::new(w.player, 0)),
        Target::Point(p) => Pose::new(p, 0),
    };
    for (k, n) in npcs.iter_mut().enumerate() {
        n.cooldown = (n.cooldown - DT).max(0);
        let base = n.run_speed.unwrap_or(SPEEDS[n.speed_index.min(5)]);
        let mode = n.mode.clone();
        match mode {
            Mode::Idle => n.speed = 0,
            Mode::Ride { on, up, turn } => {
                let Pose { pos: p, heading: h } = at(on);
                n.pos = [p[0], p[1] - up, p[2]];
                n.heading = (h + turn).rem_euclid(4096);
                n.speed = 0;
            }
            Mode::Follow(t) => {
                let p = at(t).pos;
                let d = n.dist(p);
                if n.rooted && d >= n.range / 2 {
                    // Still in its hurt state (19), which does not walk:
                    // it waits for the target to come within a bite.
                    n.speed = 0;
                } else if d > 2 * n.range {
                    n.go(p, 2 * base, w);
                } else if d >= n.range / 2 {
                    n.go(p, base, w);
                } else {
                    n.speed = 0;
                    // Only with the way to the target clear (0x80084084:
                    // the segment test 0x8002cd7c gets through).
                    let mut seg = collide::Segment::default();
                    let seen = collide::segment(w.col, n.pos, p, &mut seg) == 0;
                    if seen && n.attacks && n.cooldown == 0 && (p[1] - n.pos[1]).abs() < 1200 {
                        events.push(Event::Bite { by: k, target: t, amount: n.damage });
                        n.cooldown = n.bite_wait.min(600);
                        // The bite's state ends in state 0 (mode 1's end).
                        n.rooted = false;
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
                    n.go(post.pos, base, w);
                }
            }
            Mode::Wait { prev, cur, left } => {
                // Resume (0x80083efc) when (0x800843f0) the time is up, the
                // player comes near, or he lies within 768 of the way to
                // the next post: ahead of her (0x800760f4), the one that
                // says a line.
                n.speed = 0;
                let near = n.dist(w.player) < n.range;
                let bearing = |to: [i32; 3]| {
                    let (dx, dz) = ((to[0] - n.pos[0]) as f64, (to[2] - n.pos[2]) as f64);
                    ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32
                };
                let ahead = w
                    .posts
                    .get((cur as usize).wrapping_sub(1))
                    .is_some_and(|post| wrap(bearing(w.player) - bearing(post.pos)).abs() < 768);
                if left <= 0 || near || ahead {
                    if ahead {
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
                        n.go(p, base, w);
                    }
                } else if n.dist(n.home) > 600 {
                    let home = n.home;
                    n.go(home, base, w);
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
                    // Mode 5: the bite, then mode 1 when its animation
                    // ends: every 36 VSyncs (180 dt) in the oracle while
                    // the lobster stays on Tommy (level-16.md).
                    n.speed = 0;
                    events.push(Event::Bite { by: k, target: Target::Player, amount: n.damage });
                    n.mode = Mode::Hit { left: BITE_EVERY, then: Box::new(Mode::Prowl { prev, cur }) };
                } else if n.dist(post.pos) <= 2 * n.range && n.dist(p) <= n.range {
                    n.go(p, base, w);
                } else if n.dist(post.pos) <= n.post_reach {
                    let next = n.next_post(&post, prev, rand);
                    n.mode = Mode::Prowl { prev: cur, cur: next };
                } else {
                    n.go(post.pos, base, w);
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
                        n.go(goal, base, w);
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
        let a = n.hold.unwrap_or(if n.speed > 0 { RUN } else { IDLE });
        n.animator.set(a, w.anims);
        n.animator.step(w.anims);
        let scale = n.feet() as f32 / ROOT_SCALE;
        n.root.step(&n.animator, w.anims, scale);
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
    fn next_post(&mut self, p: &Post, prev: u8, rand: &mut Rand) -> u8 {
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
fn next_post(p: &Post, prev: u8, rand: &mut Rand) -> u8 {
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
/// The root motion's reference height (0x800c41dc, peeked in levels 1, 4
/// and 6): a body other than the player's moves by its root offset over it.
const ROOT_SCALE: f32 = -561.0;

pub fn triangles(n: &Npc, models: &Models, anims: &Anims, trig: &Trig, out: &mut Vec<Vtx>) {
    if n.hidden {
        return;
    }
    let pos = Vec3::new(n.pos[0] as f32, (n.pos[1] + n.root.y) as f32, n.pos[2] as f32);
    models::posed_triangles(&n.hierarchy, models, &n.animator.locals(&n.hierarchy, anims), n.rotation(trig), pos, out);
}
