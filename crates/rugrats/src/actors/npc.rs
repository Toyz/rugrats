//! Characters the game moves (SLUS_006.50:0x80084084 and friends): their
//! think picks a goal by mode and steers toward it (0x80082264), at a
//! speed from the table at 0x800b9698; the posts they walk come from LP
//! kind 2 (the 28-byte table at 0x800c4b9c).
//!
//! The original runs NPCs through the player's own state machine and
//! physics; the port moves them along the floor with the same speed
//! units (speed * dt along the facing) and plays their run or idle.

use rrt::glam::Mat3;
use rugrats_data::anim::Anims;
use rugrats_data::col::Col;
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::model::Models;
use rugrats_data::placement::Placement;
use rugrats_render::{Vtx, models};

use crate::actors::animator::{Animator, RootCurve};
use crate::actors::pose::{self, PartMatrix};
use crate::actors::player::anim::{IDLE, RUN, STAND as WALK};
use crate::actors::player::{Box3, DT, Pose};
use crate::gte::Trig;
use crate::rand::Rand;
use crate::world::collide;

/// The speed table (0x800b9698): row 0, and the slow row 1 (+0x670),
/// which 0x80081bac gives every character in levels 9 and 10 and
/// character 3 in levels 20 and 21.
pub const SPEEDS: [[i32; 6]; 2] = [[0, 4, 8, 12, 17, 25], [0, 2, 3, 6, 11, 13]];
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
    /// Mode 0 after a wait (0x80083efc): one frame with no body update,
    /// then the mode before.
    Resume(Box<Mode>),
    /// Mode 11: guarding home, chasing the player within range.
    Guard,
    /// Mode 1: hit, recovering for `left` dt.
    Hit { left: i32, then: Box<Mode> },
    /// Mode 1 as 0x800838b4 sets it (the hiders of level 10): an animation
    /// asked, played out where it stands; at its end mode 0 for a frame,
    /// then the back mode (+0x69b): this one, or mode 1 again.
    Act(Option<Box<Mode>>),
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

/// Where a follower is in its bite (0x80084084's modes 4, 5, 1, 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Bite {
    /// Mode 4: after the target.
    #[default]
    Chase,
    /// Mode 5: the bite to give this frame (no body update).
    Strike,
    /// Mode 1: the bite playing out, the body updated.
    Recover,
    /// Mode 0: back to mode 4 (no body update).
    Resume,
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
    /// The speed table's row (+0x670): the slow one.
    pub slow: bool,
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
    /// May attack (+0x688 bit 8).
    pub attacks: bool,
    /// LP +0x40.
    pub first_post: u8,
    /// Loop posts (flag 16): the count left and its reload (LP +0x42).
    pub loops: (i32, i32),
    /// Half its size (the hierarchy's, which a level may grow).
    pub size: i32,
    /// Its template's flag 0x2000: the collision tests meet its posed
    /// parts (0x8006a318), not its hierarchy's one box (0x8006a008).
    pub posed: bool,
    /// Its template's flag 0x10000000 (+0x688 bit 0x2000, 0x80081d64):
    /// put somewhere, it is not settled onto the floor there (0x800452f8:
    /// level 4's rats, level 15's Spike).
    pub unsettled: bool,
    /// Its draw scale (+0x36, 4096 a whole): each part's model scaled
    /// about its joint (0x8003205c: the part's matrix times the scale's).
    pub scale: i32,
    /// Left in a state that does not walk (Mr. Friend's 19 after his
    /// return in level 9's phase 2): following, it stays put until the
    /// target is within a bite.
    pub rooted: bool,
    /// The bite under way (mode 4's).
    pub bite: Bite,
    /// Where the steer aims (+0x67c), as last set.
    pub goal: [i32; 3],
    /// Its state set anew (mode 1's end): the next ask starts it again.
    pub reenter: bool,
    /// Mode 1's wait (+0x69c, dt), and whether a hit put it there (its
    /// callbacks +0x54/+0x58 off until mode 1 ends).
    pub wait_left: i32,
    pub staggered: bool,
    /// Its think is scheduled (0x80049e18, 0x80072b48): a level script
    /// puts it there, or its trigger (+0x48) when the player comes within
    /// `trigger` (+0xa0: characters 23-26, 33, 35, 39, 0x80081bac). Not
    /// scheduled, nothing of it runs, its animation neither.
    pub scheduled: bool,
    pub trigger: i32,
    /// Its template's trigger radius (+0x34, into +0xa0).
    pub radius: i32,
    /// The placement it was made from (none for one a script makes).
    pub placement: Option<usize>,
    /// The shared body its moves go through (0x8005a068), and whether it
    /// did this frame (it asked for its own animation then).
    pub body: Option<Box<crate::actors::player::Player>>,
    pub driven: bool,
    /// Waiting at a post (mode 7) or just resumed: no ask at the end.
    pub waited: bool,
    /// Its probe held lets it by (+0x688 bit 0x1000): set for route
    /// runners by their scripts and for character 33.
    pub passes: bool,
    /// Its last step was taken back off a wall or an object (the bump,
    /// state 9).
    pub bumped: bool,
    /// How near a guard bites, and how far from home it walks back
    /// (+0x690; 1024 from the init, level 2's geese 2000, level 5's robots
    /// 1536).
    pub reach: i32,
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
    /// Its parts as last drawn (+0x158: the collision tests read the
    /// pose the draw left, 0x8006a318).
    pub drawn: Option<Vec<PartMatrix>>,
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

    /// The hierarchy box's top and bottom (+0x154 + 6, + 0xe).
    pub fn box_y(&self) -> [i32; 2] {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |v| i16::from_le_bytes([v[0], v[1]]) as i32);
        [h(6), h(0xe)]
    }

    /// Its root keys' scale (0x80055904): its feet (hierarchy +0x4c)
    /// << 12 over 0x800c41dc's -561.
    pub fn root_scale(&self) -> i32 {
        (self.feet() << 12) / ROOT_SCALE
    }

    /// Its tracks set up on the idle at once, not asked (0x80082478,
    /// 0x80055f7c), and its root going on from there: an unsettled body
    /// takes the set up's lift (0x80055170) into its place, which a
    /// settled one's settling (0x800452f8) puts back on the floor.
    pub fn set_up_idle(&mut self, s: i16, anims: &Anims) {
        self.animator.set_up(s, anims);
        let lift = self.animator.take_lift(&self.hierarchy);
        if self.unsettled {
            self.pos[1] += lift;
        }
        self.root.set_up(&self.animator, anims, self.root_scale(), 0);
    }

    pub fn feet(&self) -> i32 {
        self.hierarchy.parts.first().map_or(0, |r| r.offset[1])
    }

    /// Each part's matrix as the draw poses it (0x8003205c into
    /// 0x8004a5dc), at its position raised by its root curve.
    pub fn posed(&self, trig: &Trig) -> Vec<PartMatrix> {
        let at = [self.pos[0], self.pos[1] + self.root.y, self.pos[2]];
        pose::posed(trig, &self.hierarchy, &self.animator.turns(&self.hierarchy), [self.heading as i16, 0, 0], at).0
    }

    pub fn rotation(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        Mat3::from_cols_array(
            &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
    }

    pub fn place(&mut self, at: [i32; 3], col: &Col) {
        self.pos = at;
        if !self.unsettled {
            self.snap(col);
        }
    }

    /// Settled where it is put (0x800452f8): the floor under its place
    /// (0x8002b240 at +0x18 itself), and its height that and its feet
    /// (hierarchy +0x4c).
    fn snap(&mut self, col: &Col) {
        let q = collide::query(col, self.pos);
        if q.tile.is_some() && q.kind == 0 {
            self.pos[1] = q.floor + self.feet();
        }
    }

    /// Steers toward `goal` and steps at `speed`, as the NPC's stick
    /// (0x80082264) and the shared body (0x8005a068) have it:
    /// - the heading wanted (+0x2c) is ratan2 of the way to the goal; the
    ///   heading turns toward it by at most 512 (0x800820e0, D_800c4444);
    /// - the way (+0x178) is (0, 0, speed), or a quarter of it while the
    ///   heading is more than 1024 from the one wanted (the two as 0..4095,
    ///   not wrapped), turned by the heading and set to the speed's length
    ///   (0x80076944): small integers, so the step's direction keeps to
    ///   what they can point;
    /// - the step is the way set to speed x dt;
    /// - then the body's turn (0x8005bc80) toward the heading wanted by
    ///   1025 dt >> 6 (Spike 298, 0x8005bbf4).
    fn go(&mut self, me: usize, goal: [i32; 3], speed: i32, w: &World) {
        self.steer(me, goal, speed, w, true);
    }

    /// The stick (0x80082264) and the turn (0x8005bc80); with `body`, the
    /// body's step too (state 19's handler takes none: a bite or stagger
    /// playing out only turns).
    fn steer(&mut self, me: usize, goal: [i32; 3], speed: i32, w: &World, body: bool) {
        let trig = w.trig;
        let want = trig.ratan2(goal[0] - self.pos[0], goal[2] - self.pos[2]) & 4095;
        // (0x800820e0: from 2049 less a turn, under -2048 plus one, so a way
        // straight behind, 2048, is turned to the positive side.)
        let d = match want - self.heading {
            a if a >= 2049 => a - 4096,
            a if a < -2048 => a + 4096,
            a => a,
        };
        let d = if d.abs() < TURN { d } else { TURN * d.signum() };
        self.heading = (self.heading + d).rem_euclid(4096);
        // A way round what blocks the way to the goal (0x800825fc): a
        // quarter turn's half to the side found open, the way half again
        // as long.
        let mut long = speed;
        if speed != 0 {
            let side = match self.detour(me, goal, w) {
                1 | 2 => 512,
                3 | 4 => -512,
                _ => 0,
            };
            if side != 0 {
                self.heading = (self.heading + side).rem_euclid(4096);
                long = speed + ((speed + ((speed as u32) >> 31) as i32) >> 1);
            }
        }
        let near = (self.heading - want).abs() < 1025;
        let v = if near { long } else if long < 0 { (long + 3) >> 2 } else { long >> 2 };
        let way = crate::actors::ground::set_length(trig.turn_by([self.heading as i16, 0, 0], [0, 0, v]), long, trig);
        let rate = if matches!(self.character, 6 | 34 | 45) { 298 } else { 1025 };
        let turn = (rate * DT) >> 6;
        let d = wrap(want - self.heading);
        self.heading = (self.heading + d.clamp(-turn, turn)).rem_euclid(4096);
        self.speed = speed;
        self.bumped = false;
        if body {
            self.drive_body(me, way, want, w);
        }
    }

    /// Its body set to the original's state (+0x18c: 0 stand, 1 walk, 2
    /// run) at its place (for tools starting it where the original is).
    pub fn sync_body(&mut self, state: u8, speed: i32) {
        use crate::actors::player::{Player, State};
        let walk_below = SPEEDS[self.slow as usize][4];
        let body = self.body.get_or_insert_with(|| {
            Box::new(Player::character_body(self.hierarchy.clone(), Animator::default(), self.pos, self.heading, walk_below))
        });
        body.pos = self.pos;
        body.floor = self.pos[1] - body.feet();
        body.reset_air();
        body.state = match state {
            1 => State::Walk,
            2 => State::Run,
            _ => State::Stand,
        };
        body.speed = speed;
        self.speed = speed;
    }

    /// The shared body's step (0x8005a068's handlers, `Player::drive`):
    /// along `way` at the speed, the probes, a fall off an edge, the air
    /// and the landing, the state and its animation. The character's own
    /// solid is not in its way. A body moved by a script starts again
    /// standing where it was put.
    fn drive_body(&mut self, me: usize, way: [i32; 3], want: i32, w: &World) {
        let walk_below = SPEEDS[self.slow as usize][4];
        let body = self.body.get_or_insert_with(|| {
            Box::new(crate::actors::player::Player::character_body(
                self.hierarchy.clone(),
                Animator::default(),
                self.pos,
                self.heading,
                walk_below,
            ))
        });
        if body.pos != self.pos {
            body.pos = self.pos;
            body.floor = self.pos[1] - body.feet();
            body.reset_air();
        }
        body.heading = self.heading;
        body.target = want;
        // (The landing, state 10, takes no step and plays out, the speed
        // kept: as the player's stick has it there, 0 to the body.)
        body.speed = if body.landing() { 0 } else { self.speed };
        body.hold = self.hold;
        body.passes = self.passes;
        // In the air a character is pushed on 12 a dt (0x80061330's
        // object flag 0x4000 branch), the stick's push it has none of.
        body.push = [0, 12];
        std::mem::swap(&mut body.animator, &mut self.animator);
        let solids: Vec<crate::world::boxhit::Exact> =
            w.solids.iter().zip(w.npc_of).filter(|(_, n)| **n != Some(me)).map(|(e, _)| e.clone()).collect();
        let start = body.pos;
        let on_ground = body.on_ground();
        body.drive(start, way, on_ground, false, false, w.col, w.walls, &solids, w.anims, w.trig);
        body.effects.clear();
        body.hints.clear();
        std::mem::swap(&mut body.animator, &mut self.animator);
        self.bumped = body.held_up;
        self.pos = body.pos;
        self.heading = body.heading;
        if !body.landing() {
            self.speed = body.speed;
        }
        self.driven = true;
    }

    /// The look ahead (0x800825fc): three casts from the body to the goal
    /// (the goal's place at the body's height), 384 over the floor (less
    /// the body's height over it): from its left side (its half width,
    /// 500 to 1000, out), its right, and its middle. With the middle clear
    /// and a side open: 1 if the left is blocked, 3 if the right is, else
    /// 0. Otherwise the ways round: the left one (from the left side to
    /// the goal's left, the half width times 8 (2 in levels 10, 12 and 13)
    /// out and 5 times the half depth (5000 to 10000) back) open, 4 if the
    /// right cast was blocked, else 0; else the right one blocked, 5; else
    /// 2.
    fn detour(&self, me: usize, goal: [i32; 3], w: &World) -> i32 {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |v| i16::from_le_bytes([v[0], v[1]]) as i32);
        let wider = |lo: i32, hi: i32| {
            let a = lo.abs();
            let pick = if hi < 0 { a < -hi } else { a < hi };
            (if pick { hi } else { lo }).abs()
        };
        let half = wider(h(4), h(0xc)).clamp(500, 1000);
        let depth = (5 * wider(h(8), h(0x10))).clamp(5000, 10000);
        let out = if matches!(w.level, 10 | 12 | 13) { 2 } else { 8 };
        let floor = self.pos[1] - self.feet();
        let up = 384 - (floor - self.pos[1]);
        let to = [goal[0], self.pos[1], goal[2]];
        let angles = [self.heading as i16, 0, 0];
        let cast = |a: [i32; 3], b: [i32; 3]| {
            let (oa, ob) = (w.trig.turn_by(angles, a), w.trig.turn_by(angles, b));
            let from = [0, 1, 2].map(|k| self.pos[k] + oa[k]);
            let to = [0, 1, 2].map(|k| to[k] + ob[k]);
            self.blocked(me, from, to, w)
        };
        let left = cast([-half, -up, 0], [-half, -up, 0]);
        let right = cast([half, -up, 0], [half, -up, 0]);
        let middle = cast([0, -up, 0], [0, -up, 0]);
        if !(left && right) && !middle {
            return if left {
                1
            } else if right {
                3
            } else {
                0
            };
        }
        if !cast([-half, -up, 0], [-out * half, -up, -depth]) {
            return if right { 4 } else { 0 };
        }
        if cast([half, -up, 0], [out * half, -up, -depth]) { 5 } else { 2 }
    }

    /// Whether a cast from `from` to `to` is blocked (0x80082b60 through
    /// 0x80090554): the nearest object it meets (not itself, nor of
    /// template flag 16), then the world's segment, kept over the object
    /// only when nearer on all three axes. The world blocks; an object of
    /// template flag 72 or 8 does not.
    fn blocked(&self, me: usize, from: [i32; 3], to: [i32; 3], w: &World) -> bool {
        let mut best: Option<(i32, [i32; 3], u32)> = None;
        for o in w.obstacles {
            if o.flags & 16 != 0 || o.npc == Some(me) {
                continue;
            }
            let mut n = [0; 3];
            if let Some((_, hit)) = o.solid.meet(w.trig, from, to, &mut n)
                && hit.t >= 0
                && best.is_none_or(|b| hit.t < b.0)
            {
                best = Some((hit.t, hit.point, o.flags));
            }
        }
        let mut seg = collide::Segment::default();
        if collide::segment(w.col, from, to, &mut seg) != 0 {
            let nearer = |p: [i32; 3]| (0..3).all(|k| (from[k] - seg.point[k]).abs() < (from[k] - p[k]).abs());
            if best.is_none_or(|b| nearer(b.1)) {
                return true;
            }
        }
        best.is_some_and(|b| b.2 & 72 == 0)
    }

    fn dist(&self, p: [i32; 3]) -> i32 {
        approx_dist(self.pos, p)
    }
}

/// The game's distance across (0x8001f57c): |dx| + |dz| less half the
/// smaller.
pub fn approx_dist(a: [i32; 3], b: [i32; 3]) -> i32 {
    let (dx, dz) = ((b[0] - a[0]).abs(), (b[2] - a[2]).abs());
    let m = dx.min(dz);
    dx + dz - ((m + ((m as u32) >> 31) as i32) >> 1)
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
    /// What a character's look ahead meets (0x80090554 over the rooms'
    /// lists): each object's solid, its template's flags and whose it is.
    pub obstacles: &'a [Obstacle],
    /// The level (0x800c5678).
    pub level: u8,
    /// The player's height with its root (+0x1c) and its hierarchy box's
    /// top and bottom (+0x154 + 6, + 0xe).
    pub player_y: i32,
    pub player_box_y: [i32; 2],
    /// This frame's solids, and which character each is.
    pub solids: &'a [crate::world::boxhit::Exact],
    pub npc_of: &'a [Option<usize>],
}

/// An object as the characters' casts see it.
pub struct Obstacle {
    pub solid: crate::world::boxhit::Solid,
    /// Its template's flags (D_800c4300 + 56 x template).
    pub flags: u32,
    /// The NPC it is, if one.
    pub npc: Option<usize>,
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
    let heights: Vec<(i32, [i32; 2])> = npcs.iter().map(|n| (n.pos[1] + n.root.y, n.box_y())).collect();
    for (k, n) in npcs.iter_mut().enumerate() {
        // (A character a level sets going is one its script scheduled.)
        let runs = if n.trigger > 0 { n.scheduled } else { n.scheduled || !matches!(n.mode, Mode::Idle) };
        if !runs {
            continue;
        }
        // The animation steps first and its root with it (0x8005a068),
        // then the mode asks for what plays from the next step. A bite's
        // modes 5 and 0 have no body update (0x80084084 leaves without
        // 0x80082230).
        // Mode 1 at its animation's event 0x2000 (not for 1776): a wait of
        // 300 + (rand & 3) x 300 dt (+0x69c), with no body update.
        if (matches!(n.mode, Mode::Follow(_) | Mode::Guard) && n.bite == Bite::Recover || matches!(n.mode, Mode::Act(_)))
            && n.animator.events & 0x2000 != 0
            && n.id != 1776
        {
            if n.wait_left == 0 {
                n.wait_left = 300 + (rand.next() & 3) as i32 * 300;
            }
            n.wait_left = (n.wait_left - DT).max(0);
            if n.wait_left != 0 {
                continue;
            }
        }
        // (Reaching a post to wait at, mode 6 leaves without the body's
        // update: 0x80083e2c has made it mode 7.)
        let enters_wait = match n.mode {
            Mode::Posts { cur, .. } => w
                .posts
                .get((cur as usize).wrapping_sub(1))
                .is_some_and(|p| p.flags & post::WAIT != 0 && p.flags & post::END == 0 && n.dist(p.pos) <= n.post_reach),
            _ => false,
        };
        let body = !(matches!(n.mode, Mode::Follow(_) | Mode::Guard) && matches!(n.bite, Bite::Strike | Bite::Resume))
            && !matches!(n.mode, Mode::Resume(_))
            && !enters_wait;
        let mut across = [0; 2];
        // (The root's height before this step, for level 3's brothers.)
        let lift_before = n.root.y;
        if body {
            n.animator.step(w.anims, w.trig);
            let lo = n.root_scale();
            // (An unsettled body's keys start at the floor under it.)
            let off = if n.unsettled {
                let q = collide::query(w.col, n.pos);
                if q.tile.is_some() && q.kind == 0 { q.floor + n.feet() - n.pos[1] } else { 0 }
            } else {
                0
            };
            n.root.step(&n.animator, w.anims, lo, off);
            // Across by the root (the stagger's step back), turned by the
            // heading as it was; the body takes it after its stick (the
            // steer reads the place before it, 0x8005a068).
            let d = n.root.across(&n.animator, w.anims, lo);
            if d != [0; 2] {
                let m = w.trig.turn_by([n.heading as i16, 0, 0], [d[0], 0, d[1]]);
                across = [m[0], m[2]];
            }
        }
        let base = n.run_speed.unwrap_or(SPEEDS[n.slow as usize][n.speed_index.min(5)]);
        let mode = n.mode.clone();
        // A bite under way (0x80084084's modes 5, 1 and 0), a follower's
        // on its target, a guard's on the player.
        let biting = match mode {
            Mode::Follow(t) => Some(t),
            Mode::Guard => Some(Target::Player),
            _ => None,
        };
        if let Some(t) = biting {
            match n.bite {
                // Mode 5 (0x800846e4): state 19, the bite (37) asked at
                // 4096, Damage to the target (0x800780ac); then mode 1.
                Bite::Strike => {
                    events.push(Event::Bite { by: k, target: t, amount: n.damage });
                    n.animator.start_at(37, 4096, w.anims);
                    n.bite = Bite::Recover;
                    continue;
                }
                // Mode 1 (0x800841cc): the bite plays out, the body still
                // turning to where it aimed; done (0x8000000), mode 0 and
                // its speed back. (The wait on event 0x2000 does not come
                // in the bites seen.)
                Bite::Recover => {
                    let goal = n.goal;
                    n.steer(k, goal, 0, w, false);
                    n.pos[0] += across[0];
                    n.pos[2] += across[1];
                    if n.animator.events & 0x800_0000 != 0 {
                        n.bite = Bite::Resume;
                        n.staggered = false;
                    }
                    continue;
                }
                // Mode 0: its mode again (+0x69b).
                Bite::Resume => {
                    n.bite = Bite::Chase;
                    n.reenter = true;
                    continue;
                }
                Bite::Chase => {}
            }
        }
        match mode {
            Mode::Idle => n.speed = 0,
            Mode::Ride { on, up, turn } => {
                let Pose { pos: p, heading: h } = at(on);
                n.pos = [p[0], p[1] - up, p[2]];
                n.heading = (h + turn).rem_euclid(4096);
                n.speed = 0;
            }
            Mode::Follow(t) => {
                // Mode 4 (0x80084cd0): with the way to the target clear
                // (0x8002cd7c), within half the range and the two bodies
                // level (0x80083ff0, their boxes' heights overlapping) it
                // stops, and with +0x688 bit 8 bites (mode 5). Its speed:
                // twice the table's past twice the range, the table's past
                // half of it, else as it was.
                let p = at(t).pos;
                n.goal = p;
                let d = n.dist(p);
                let (ty, tb) = match t {
                    Target::Player => (w.player_y, w.player_box_y),
                    Target::Npc(j) => heights.get(j).copied().unwrap_or((p[1], [0, 0])),
                    Target::Point(p) => (p[1], [0, 0]),
                };
                // (Its height before the body's step, as 0x80084cd0 runs
                // before 0x8005a068.)
                let (my, mb) = (n.pos[1] + lift_before, n.box_y());
                let level = !(ty + tb[1] < my + mb[0]) && !(my + mb[1] < ty + tb[0]);
                let mut seg = collide::Segment::default();
                let seen = collide::segment(w.col, n.pos, p, &mut seg) == 0;
                let mut speed = n.speed;
                if seen && d < n.range / 2 && level {
                    speed = 0;
                    if n.attacks {
                        n.bite = Bite::Strike;
                        n.rooted = false;
                    }
                }
                if n.rooted && d >= n.range / 2 {
                    // Still in its hurt state (19), which does not walk:
                    // it waits for the target to come within a bite.
                    speed = 0;
                } else if d > 2 * n.range {
                    speed = 2 * base;
                } else if d >= n.range / 2 {
                    speed = base;
                }
                n.go(k, p, speed, w);
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
                        // 0x80083e2c: the speed put by, the wait (logical
                        // 9) asked at 4096, a line; mode 7 (no body update
                        // this frame).
                        events.push(Event::Waits(k));
                        n.speed = 0;
                        n.animator.start_at(9, 4096, w.anims);
                        n.driven = false;
                        n.waited = true;
                        Mode::Wait { prev: cur, cur: next, left: n.wait * DT / 2 }
                    } else {
                        // The body still steps this frame, aimed at the post
                        // reached (mode 6 takes its place for +0x67c
                        // before 0x80082c78 moves on).
                        n.go(k, post.pos, base, w);
                        Mode::Posts { prev: cur, cur: next }
                    };
                } else {
                    n.go(k, post.pos, base, w);
                }
            }
            Mode::Wait { prev, cur, left } => {
                // Mode 7 (0x800843f0): state 19, the look-at the player; the
                // body updated (its stick turning it to the player, no step:
                // state 19's handler); then the resume (0x80083efc) when the
                // wait (+0x68e, VSync ticks) is up or the player is within
                // range, and again when he lies within 768 of the way to the
                // next post (both as 0..4095, 0x8001f650), the one with a
                // line.
                n.speed = 0;
                // Level 3's Larry and Steve (75, 76) are put at height -384
                // each wait frame (0x80084438), before the body adds the
                // step's lift: drawn at -384 and this step's change of it.
                if w.level == 3 && matches!(n.id, 75 | 76) {
                    n.pos[1] = -384 - lift_before;
                }
                let p = w.player;
                n.steer(k, p, 0, w, false);
                let yaw = |to: [i32; 3]| w.trig.ratan2(to[0] - n.pos[0], to[2] - n.pos[2]) & 4095;
                // (VSync since the entry, 2 a frame, against the wait.)
                let left = left - DT;
                let mut resume = left <= 0 || n.dist(p) < n.range;
                let ahead = w.posts.get((cur as usize).wrapping_sub(1)).is_some_and(|post| (yaw(post.pos) - yaw(p)).abs() < 768);
                if ahead {
                    events.push(Event::Resumes(k));
                    resume = true;
                }
                n.mode = if resume {
                    n.sync_body(0, n.speed);
                    Mode::Resume(Box::new(Mode::Posts { prev, cur }))
                } else {
                    Mode::Wait { prev, cur, left }
                };
                n.waited = true;
            }
            Mode::Resume(then) => {
                n.mode = *then;
                n.waited = true;
            }
            Mode::Guard => {
                let p = w.player;
                // (0x80085220.) Its height is its place and its lift, as
                // the original's +0x1c before the body's update; the
                // distances the game's (0x8001f57c).
                let my_y = n.pos[1] + lift_before;
                let near = approx_dist(p, n.home) < n.range && (p[1] - my_y).abs() < 2800;
                // The goal (+0x67c) and how far (+0x674): the player within
                // the range of its home and height, at their distance; else
                // its home when it is a reach or more away (+0x688 bit 2,
                // set by 0x80083b4c); else the player, at none.
                let (goal, away) = if near {
                    (p, approx_dist(n.pos, p))
                } else {
                    let d = approx_dist(n.home, n.pos);
                    if d < n.reach { (p, 0) } else { (n.home, d) }
                };
                n.goal = goal;
                if away < n.reach {
                    // Stopped (+0x184 into +0x176), it still turns to its
                    // goal: the stick's 512 and the body's 160. Turning so,
                    // standing, 0x800820e0 sets +0x688 bit 0x10000 and the
                    // stand asks the turn in place (logical 53, state 63,
                    // 0x8005d11c).
                    let was = n.heading;
                    // (The turn alone: it stays where it is.)
                    n.steer(k, goal, 0, w, false);
                    if n.heading != was {
                        n.hold = Some(TURN_IN_PLACE);
                        n.animator.set(TURN_IN_PLACE, w.anims);
                    } else if n.hold == Some(TURN_IN_PLACE) {
                        n.hold = None;
                    }
                    n.speed = 0;
                    // The attack (0x80084084 at 0x80085220): near, the way
                    // to the player clear, and the two bodies level
                    // (0x80083ff0: their boxes' heights overlapping).
                    if near {
                        let (ty, tb) = (w.player_y, w.player_box_y);
                        let mb = n.box_y();
                        let level = !(ty + tb[1] < my_y + mb[0]) && !(my_y + mb[1] < ty + tb[0]);
                        let mut seg = collide::Segment::default();
                        let seen = collide::segment(w.col, n.pos, p, &mut seg) == 0;
                        // (+0x688 bit 8: mode 5, the bite. Bit 512's snap
                        // and mode 1 is never set by any code.)
                        if level && seen && n.attacks {
                            n.bite = Bite::Strike;
                        }
                    }
                } else {
                    n.go(k, goal, base, w);
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
                    n.go(k, p, base, w);
                } else if n.dist(post.pos) <= n.post_reach {
                    let next = n.next_post(&post, prev, rand);
                    n.mode = Mode::Prowl { prev: cur, cur: next };
                } else {
                    n.go(k, post.pos, base, w);
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
                        n.go(k, goal, base, w);
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
            // Mode 1 (0x800841cc): state 19, its body updated with no step.
            // (Its stick turns it to +0x67c, which nothing set for the
            // hiders: a heading it has long come to.)
            Mode::Act(then) => {
                n.speed = 0;
                n.pos[0] += across[0];
                n.pos[2] += across[1];
                if n.animator.events & 0x800_0000 != 0 {
                    let back = then.map_or(Mode::Act(None), |m| *m);
                    n.mode = Mode::Resume(Box::new(back));
                }
                continue;
            }
        }
        n.pos[0] += across[0];
        n.pos[2] += across[1];
        // Walking under the table's fifth speed of its row (0x80059008,
        // 0x800590d4 for objects of flag 0x4000), running from it.
        let run_from = SPEEDS[n.slow as usize][4];
        if std::mem::take(&mut n.waited) {
            // (The wait plays its own animation; mode 0 asks nothing.)
            continue;
        }
        if std::mem::take(&mut n.driven) {
            // (Its body asked for what it plays; entered anew after a
            // bite, afresh.)
            if std::mem::take(&mut n.reenter) {
                let a = n.animator.asked();
                n.animator.start(a, w.anims);
            }
            continue;
        }
        let a = n.hold.unwrap_or(match n.speed {
            s if s <= 0 => IDLE,
            s if s < run_from => WALK,
            _ => RUN,
        });
        if std::mem::take(&mut n.reenter) {
            // Back from the bite's state 19 (mode 1 left state 0 for the
            // body's next update): the state's entry asks anew (0x8005b2d8).
            n.animator.start(a, w.anims);
        } else if n.hold.is_none() && n.speed <= 0 {
            n.animator.stand(w.anims);
        } else {
            n.animator.set(a, w.anims);
        }
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

/// The next post (0x80082ee0): with flag 4, a neighbour slot drawn as
/// rand & 3 until one is set (40 draws at most); else the first neighbour
/// that is not the one come from; else the first slot.
fn next_post(p: &Post, prev: u8, rand: &mut Rand) -> u8 {
    if p.flags & post::RANDOM != 0 {
        for _ in 0..40 {
            let n = p.next[(rand.next() & 3) as usize];
            if n != 0 {
                return n;
            }
        }
        return p.next[0];
    }
    p.next.iter().copied().find(|&n| n != 0 && n != prev).unwrap_or(p.next[0])
}

/// The root motion's reference height (0x800c41dc, peeked in levels 1, 4
/// and 6): a body other than the player's moves by its root offset over it.
const ROOT_SCALE: i32 = -561;

/// The turn in place a standing character plays (logical 53, table state
/// 63: 0x8005d11c's ask when 0x80059204 answers below 0).
const TURN_IN_PLACE: i16 = 53;

/// The NPC's triangles.
pub fn triangles(n: &mut Npc, models: &Models, trig: &Trig, out: &mut Vec<Vtx>) {
    if n.hidden {
        return;
    }
    let posed = n.posed(trig);
    for (part, p) in n.hierarchy.parts.iter().zip(&posed) {
        if let Some(m) = models.get(part.model) {
            let (rot, joint) = p.float();
            models::model_triangles(m, rot * (n.scale as f32 / 4096.0), joint, out);
        }
    }
    n.drawn = Some(posed);
}
