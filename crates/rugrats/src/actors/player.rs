//! The player (object 0x800e2b48): the stick, the ground move, the floor
//! under it, and its base animation track.
//!
//! The update (0x8005a068) as far as it is ported: the stick (0x8005be24)
//! sets a wanted heading and a speed, the heading turns toward it
//! (0x8005bc80), the ground move adds speed * dt along the facing
//! (0x8005caac, 0x8005cb48), and the floor query (0x800627a0) puts the feet
//! on the floor or sends the player back to where the frame began.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::col::Col;
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::model::Models;
use rugrats_render::Vtx;
use rugrats_render::models::{self, posed_triangles};

use crate::actors::animator::Animator;
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::input::{self, Pads};
use crate::world::collide;

/// dt a frame at 30 fps (0x800c3f1c: 5 a vertical blank).
pub const DT: i32 = 10;
/// Speed factors (0x800c4200 running, 0x800c4204 walking).
const RUN: i32 = 17;
const WALK: i32 = 8;
/// The stick's dead zone (0x8005be24) and the widest turn taken at once
/// (0x800c42cc), and the speed above which any turn is (0x800c42c8).
const DEAD: i32 = 71;
const SNAP_TURN: i32 = 1024;
const MOVING: i32 = 100;
/// The turn rate (0x800c41fc): 1025 * dt >> 6 a frame.
pub const TURN: i32 = 1025;
/// Spike's (and char 34's) turn rate (0x800c420c).
pub const SLOW_TURN: i32 = 298;
/// The player's character: Tommy (0x800c5680 = 1).
const TOMMY: u16 = 1;
/// Gravity (0x80061330): vy += 14 * dt, less 5 while jump is held.
const GRAVITY: i32 = 14;
const LIFT: i32 = -5;

/// Logical animations (0x800572a8's argument) the port uses; the
/// animator turns them into the character table's states (0x80056fe4).
pub mod anim {
    /// Standing. The stand state asks for +0xe7 (17 from init: "as you
    /// were"); what shows in the oracle once stopped is table state 0,
    /// logical 0.
    pub const IDLE: i16 = 0;
    pub const CARRY_IDLE: i16 = 20;
    pub const PICK_UP: i16 = 13;
    pub const PUT_DOWN: i16 = 14;
    pub const THROW: i16 = 18;
    pub const STAND: i16 = 1;
    pub const RUN: i16 = 2;
    pub const JUMP: i16 = 3;
    pub const LAND: i16 = 4;
    pub const FALL: i16 = 15;
    pub const JUMP_MOVING: i16 = 28;
    /// Climbing (state 6's animation).
    pub const CLIMB: i16 = 5;
    /// The cry (0x8007f920: 0x800572a8(player, 16, 2560)).
    pub const CRY: i16 = 16;
    /// The slip (states 17, 18).
    pub const SLIP: i16 = 33;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stand,
    Walk,
    Run,
    Jump,
    Fall,
    Land,
}

/// Where a body stands and which way it faces: a start, a checkpoint, a
/// respawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pose {
    pub pos: [i32; 3],
    pub heading: i32,
}

impl Pose {
    pub fn new(pos: [i32; 3], heading: i32) -> Pose {
        Pose { pos, heading }
    }
}

pub struct Player {
    pub pos: [i32; 3],
    /// Heading (+0x24) and the one wanted (+0x2c), 4096 a turn.
    pub heading: i32,
    pub target: i32,
    pub speed: i32,
    /// Air velocity (+0x170): x, y, z.
    pub v: [i32; 3],
    pub state: State,
    pub floor: i32,
    /// Feet to origin (hierarchy +0x4c, the root's y offset).
    feet: i32,
    pub hierarchy: Hierarchy,
    /// Health (+0x1a4, 255 at init, 0x80059a88).
    pub health: i32,
    /// Frozen (+0x188 bit 1): turning, no moving.
    pub frozen: bool,
    /// Golf's aim (state 14): 0x8005be24's own case steps the heading, so
    /// the stick turns the body no other way.
    pub script_aims: bool,
    /// The turn rate (0x800c41fc; 298 for chars 6 and 34).
    pub turn_rate: i32,
    pub animator: Animator,
    /// Sound effects asked for this frame (0x8006f644's numbers).
    pub effects: Vec<u8>,
    /// An action's animation playing (pick up 13, put down 14, throw 18):
    /// the player stands until it ends.
    pub action: Option<i16>,
    /// An animation a level holds the player in, looped (golf's stance).
    pub hold: Option<i16>,
    /// Jump ignored (golf, 0x8005d11c).
    pub no_jump: bool,
    /// The highest ledge a climb takes, and climbing refused here
    /// (0x80058da4: room 4 in levels 1 and 12; level 10's type 14).
    pub climb_max: i32,
    pub climb_off: bool,
    /// The moving jump's lift and factor (0x80060f1c case 25: -1000 and
    /// 40; Spike in level 11 -1700 and 20).
    pub moving_jump: (i32, i32),
    /// The last jump was from standing (0x800c4bc4 = 1, 0x80060f1c
    /// case 0); a moving jump clears it.
    pub standing_jump: bool,
    /// The stick's run and walk factors (0x8005be24: 17 and 8; Spike 20
    /// and 10; the tricycle runs at 20, as seen).
    pub gait: (i32, i32),
    /// Floating (state 21, 0x8005db28): the depth anchor (+0x65c) and the
    /// bob's phase (+0x660).
    pub float: Option<(i32, i32)>,
    /// Carrying: the idle is 20 (0x80057244 at the grab).
    pub carrying: bool,
    /// What is carried holds the climbs under 522 (0x80058da4: an object
    /// without bit 0x1000000, class 13's).
    pub carry_climbs: bool,
    /// The jump lock after a throw (0x800c4290: 300, less dt a frame).
    pub jump_lock: i32,
    /// Training messages asked for this frame (0x80097788's n).
    pub hints: Vec<u8>,
    /// Time falling (0x800c428c, summed in 0x80061330).
    air: i32,
    /// Slipping on floor flag 2 (states 3, 17, 18: 0x8005dedc), and the
    /// slip's sound (169 in level 16, else 56).
    slipping: bool,
    pub slip_sound: u8,
    /// A long fall hurts (not in the hub, 0x8005b6e0).
    pub falls_hurt: bool,
    /// The box the feet stand on this frame (player +0xf8), an index into
    /// the boxes `frame` was given.
    pub support: Option<usize>,
    /// The landing began this frame (state 10's entry, 0x8005b624).
    pub landed: bool,
    /// A climb under way: how far it rises. The port puts the body at the
    /// top at once; the original's climb moves it by the animation's root
    /// keys (0x80055904 into +0xd8..+0xdc, moved by 0x80066e60), so the
    /// body is drawn where that motion has it.
    climb: Option<Climb>,
    /// A standing jump's crouch: its launch, kept until the jump's
    /// airborne state comes (table state 6, whose entry sets +0xe0 bit 31,
    /// 0x800562bc, which 0x8005ec9c takes as leaving the ground).
    pending_jump: Option<[i32; 3]>,
    /// The root's y the animations have moved the body by on the ground
    /// (into +0xda each step, 0x80056834): drawn under its position.
    /// Standing, walking or running, the floor takes back 23 a frame of
    /// it below the floor (the oracle: 868, 845, 822, 799, 779 after a
    /// landing).
    root_y: i32,
    /// The base track's root y so far, which animation it was of, and the
    /// frame it was at (a loop starts again under it).
    root_seen: RootSeen,
    /// Where the body was over the floor as the base track's animation
    /// began: its first frame blends it away.
    root_base: i32,
}

impl Player {
    /// Loses `amount` health, to 0 at least.
    pub fn hurt(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
    }

    /// Gains `amount` health, to 255 at most.
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(255);
    }

    /// Where it stands and faces now.
    pub fn pose(&self) -> Pose {
        Pose::new(self.pos, self.heading)
    }

    /// Puts the player at `start` with heading `heading` on the floor
    /// (0x800596e8), with `hierarchy`.
    pub fn new(
        disc: &GameDisc,
        hierarchy: Hierarchy,
        start: [i32; 3],
        heading: i32,
        col: &Col,
        anims: &Anims,
    ) -> Result<Player, String> {
        let animator = Animator::load(disc, TOMMY)?;
        let feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        let mut p = Player {
            pos: start,
            heading,
            target: heading,
            speed: 0,
            v: [0; 3],
            state: State::Stand,
            floor: start[1],
            feet,
            hierarchy,
            health: 255,
            frozen: false,
            script_aims: false,
            turn_rate: TURN,
            animator,
            effects: Vec::new(),
            action: None,
            hold: None,
            no_jump: false,
            climb_max: CLIMB,
            climb_off: false,
            moving_jump: (-1000, 40),
            standing_jump: false,
            gait: (RUN, WALK),
            float: None,
            carrying: false,
            carry_climbs: false,
            jump_lock: 0,
            hints: Vec::new(),
            air: 0,
            slipping: false,
            slip_sound: 56,
            falls_hurt: true,
            support: None,
            landed: false,
            climb: None,
            pending_jump: None,
            root_y: 0,
            root_seen: RootSeen::default(),
            root_base: 0,
        };
        let q = collide::query(col, start);
        if q.tile.is_some() {
            p.floor = q.floor;
            p.pos[1] = q.floor + feet;
        }
        p.animator.set(anim::IDLE, anims);
        Ok(p)
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left), for tools.
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        self.animator.track()
    }

    pub fn anm(&self, a: i16) -> Option<usize> {
        self.animator.anm(a)
    }

    /// One frame of play. `camera_heading` is the camera's (4096 a turn).
    pub fn frame(&mut self, pads: &Pads, camera_heading: i32, col: &Col, boxes: &[Box3], anims: &Anims, trig: &Trig) {
        if self.float.is_some() {
            self.float_frame(pads, camera_heading, col, anims, trig);
            return;
        }
        self.effects.clear();
        self.hints.clear();
        let held = |b: usize| pads.held(0, b);
        let pressed = |b: usize| pads.pressed(0, b);
        let start = self.pos;
        // The stick: digital directions are full (4096) on their axis;
        // up is +z.
        let sx = if held(input::LEFT) {
            -4096
        } else if held(input::RIGHT) {
            4096
        } else {
            0
        };
        let sz = if held(input::UP) {
            4096
        } else if held(input::DOWN) {
            -4096
        } else {
            0
        };
        let mut magnitude = (((sx * sx + sz * sz) as f64).sqrt() as i32).min(4096);
        let on_ground = matches!(self.state, State::Stand | State::Walk | State::Run | State::Land);
        self.jump_lock = (self.jump_lock - DT).max(0);
        // An action holds the player until its animation ends.
        if let Some(a) = self.action {
            if self.animator.playing(a) {
                magnitude = 0;
            } else {
                self.action = None;
                self.finish_climb(col, boxes, false);
                // The slip's fall ends (state 18): Damage 5, message 30.
                if self.slipping {
                    self.slipping = false;
                    self.hurt(SLIP_DAMAGE);
                    self.hints.push(30);
                }
            }
        }
        let acting = self.action.is_some();
        if magnitude >= DEAD {
            let dir = (((sx as f64).atan2(sz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
            let want = (dir + camera_heading).rem_euclid(4096);
            let factor = if held(input::SQUARE) { self.gait.1 } else { self.gait.0 };
            let turn = wrap(want - self.heading);
            if self.frozen {
                // Held in place (+0x188 bit 1): only the turn runs, so the
                // stick aims.
                if !self.script_aims {
                    self.target = want;
                }
                self.speed = 0;
            } else if turn.abs() <= SNAP_TURN || self.speed >= MOVING || !on_ground {
                self.heading = want;
                self.target = want;
                self.speed = (magnitude * factor) >> 12;
                // Running at full speed (0x800590d4): message 2.
                if self.speed > (self.gait.0 * 3933) >> 12 {
                    self.hints.push(2);
                }
            } else {
                self.target = want;
                self.speed = 0;
            }
        } else {
            self.speed = 0;
            if self.frozen {
                // The aim stops where the stick is let go (guess).
                self.target = self.heading;
            }
        }
        // Turn toward the wanted heading (0x8005bbf4).
        let step = (self.turn_rate * DT) >> 6;
        let d = wrap(self.target - self.heading);
        self.heading = (self.heading + d.clamp(-step, step)).rem_euclid(4096);
        let (s, c) = (trig.sin(self.heading), trig.cos(self.heading));
        if on_ground {
            // The ground move along the facing (0x8005caac); none while
            // landing.
            if !self.frozen && self.state != State::Land {
                let (dx, dz) = self.ground_step(col, (s * self.speed * DT) >> 12, (c * self.speed * DT) >> 12);
                self.pos[0] += dx;
                self.pos[2] += dz;
            }
            if pressed(input::CROSS) && !acting && self.jump_lock == 0 && !self.no_jump && !self.frozen {
                // The launch (0x80060f1c): up 1000, and along the facing
                // 40 a unit of speed when running, else 12.
                self.standing_jump = self.speed <= WALK;
                let (up, factor) = if self.standing_jump { (-1000, 12) } else { self.moving_jump };
                let along = factor * self.speed;
                self.pending_jump = Some([(s * along) >> 12, up, (c * along) >> 12]);
                self.v = [0; 3];
                self.state = State::Jump;
            }
        } else if self.pending_jump.is_some() {
            // Crouching for the jump: still on the floor.
        } else {
            // The air (0x80061330): gravity 14, less 5 while jump is held
            // on the way up; drag 32/4096 of the speed.
            let lift = if held(input::CROSS) && self.v[1] < 0 { LIFT } else { 0 };
            let a = GRAVITY + lift;
            for k in [0, 2] {
                self.v[k] -= self.v[k] * 32 / 4096;
                self.pos[k] += (self.v[k] * DT) >> 6;
            }
            self.pos[1] += (self.v[1] * DT + a * DT * DT / 2) >> 6;
            self.v[1] = (self.v[1] + a * DT).min(4096);
        }
        // The floor (0x800627a0): the feet on it, or back where the frame
        // began when the point is not in the open.
        // Asked from a step's height above the feet, so a slope or a
        // step up within STEP_UP is floor, not wall.
        let lift = if on_ground { STEP_UP } else { 1 };
        let mut probe = [self.pos[0], self.pos[1] - self.feet - lift, self.pos[2]];
        let mut q = collide::query(col, probe);
        // In the air the move goes across, then down (0x8005f1b4 through
        // 0x8005ee78): across into a place with no floor record stops the
        // across part only, and the fall goes on.
        if !on_ground && (q.kind != 0 && q.tile.is_none() || q.kind == 2) {
            self.pos[0] = start[0];
            self.pos[2] = start[2];
            probe = [self.pos[0], self.pos[1] - self.feet - lift, self.pos[2]];
            q = collide::query(col, probe);
        }
        // Climbing (state 6, 0x8005f310): walking into a ledge or a
        // climbable object up to 1033 above the feet (521 carrying,
        // 0x80056e44, 0x80058da4) brings the player onto it.
        let walking = matches!(self.state, State::Walk | State::Run) && self.speed > 0 && !acting && !self.climb_off;
        let reach = if self.carrying && self.carry_climbs { CLIMB_CARRYING.min(self.climb_max) } else { self.climb_max };
        let mut climb_to: Option<i32> = None;
        // Falling through the floor this step (0x8005f1b4 lands when y +
        // vy reaches it): onto the floor under where the step began.
        let landing = (!on_ground && self.v[1] > 0 && q.kind != 0)
            .then(|| collide::query(col, [self.pos[0], start[1] - self.feet - 1, self.pos[2]]))
            .filter(|f| f.kind == 0 && f.tile.is_some() && self.pos[1] - self.feet >= f.floor);
        if let Some(f) = landing {
            self.pos[1] = f.floor + self.feet;
            self.floor = f.floor;
        } else if q.kind != 0 && q.tile.is_none() || q.kind == 2 {
            self.pos = start;
        } else if q.kind != 0 && walking && {
            let feet_y = start[1] - self.feet;
            let up = collide::query(col, [self.pos[0], feet_y - reach, self.pos[2]]);
            let ok = up.kind == 0 && up.floor < feet_y - STEP_UP && up.floor >= feet_y - reach;
            if ok {
                climb_to = Some(up.floor);
            } else {
                // Too tall (0x80058da4): a step of 1025..3071, message 16.
                let tall = collide::query(col, [self.pos[0], feet_y - TOO_TALL, self.pos[2]]);
                if tall.kind == 0 && tall.floor < feet_y - CLIMB && tall.floor >= feet_y - TOO_TALL {
                    self.hints.push(16);
                }
            }
            ok
        } {
            // Onto the ledge.
        } else if q.kind != 0 {
            // Into a wall or a step too high: back across, keep the height.
            let back = collide::query(col, [start[0], probe[1], start[2]]);
            self.pos = [start[0], self.pos[1], start[2]];
            if back.kind == 0 {
                self.floor = back.floor;
            }
        } else {
            self.floor = q.floor;
        }
        // Objects' boxes: one whose top is within a step of the feet and
        // not above the body's origin (or below them, falling) is floor
        // (the floor test runs down from the origin, 0x80065578); one the
        // step's probes meet (0x80063d3c: from the body's middle, ahead
        // and to each side, at the box's own middle height kept within
        // the body's, 0x8006a714) sends it back across. A character's body
        // is met where it stands.
        let start_feet = start[1] - self.feet;
        let feet_y = self.pos[1] - self.feet;
        let (dx, dz) = ((self.pos[0] - start[0]) as f32, (self.pos[2] - start[2]) as f32);
        let len = dx.hypot(dz);
        // The probes are the ground move's (0x80064a34); in the air the body
        // is met where it is.
        let ground = on_ground && matches!(self.state, State::Stand | State::Walk | State::Run);
        let probes = if len < 1.0 || !ground { Vec::new() } else { self.probes([dx / len, dz / len], len) };
        let mut blocked = false;
        self.support = None;
        for (k, b) in boxes.iter().enumerate() {
            let near = b.covers(self.pos, RADIUS);
            if !near && probes.is_empty() {
                continue;
            }
            let top = b.min_y;
            let head = feet_y - HEIGHT;
            let floor_like = top >= start_feet.min(feet_y) - STEP_UP && (b.character || !ground || top >= start[1].min(self.pos[1]));
            let met = !floor_like
                && b.max_y > head
                && if b.character || !ground {
                    near && (b.depth(self.pos, RADIUS) > b.depth(start, RADIUS) || b.crosses(start, self.pos))
                } else {
                    let h = self.probe_height(b, start);
                    probes.iter().any(|&(dx, dz)| b.segment_hit([start[0], h, start[2]], [start[0] + dx, h, start[2] + dz]).is_some())
                };
            if near && floor_like {
                // Standing room on it: a floor if higher than the one found.
                if top < self.floor {
                    self.floor = top;
                    self.support = Some(k);
                }
            } else if met {
                if b.climb && walking && top >= start_feet - reach {
                    climb_to = Some(climb_to.map_or(top, |c: i32| c.min(top)));
                } else {
                    // Not climbable (29) or too tall (16).
                    if walking && !b.character {
                        if top >= start_feet - reach {
                            self.hints.push(29);
                        } else if b.climb && top < start_feet - CLIMB && top >= start_feet - TOO_TALL {
                            self.hints.push(16);
                        }
                    }
                    blocked = true;
                }
            }
        }
        if blocked {
            self.pos[0] = start[0];
            self.pos[2] = start[2];
        }
        if let Some(top) = climb_to {
            // The rise picks the climb (+0x144, 0x80056e44).
            self.animator.climb = self.floor - top;
            self.climb = Some(Climb { top: top + self.feet, whole: [0; 3], from: start, moved: [0; 3] });
            self.pos[1] = top + self.feet;
            self.floor = top;
            self.state = State::Stand;
            self.act(anim::CLIMB, anims);
            let rot = self.rotation(trig);
            if let (Some((_, total)), Some(c)) = (self.animator.root_progress(anims), &mut self.climb) {
                let w = rot * Vec3::new(total[0] as f32, total[1] as f32, total[2] as f32);
                c.whole = [w.x as i32, w.y as i32, w.z as i32];
            }
            self.hints.push(1);
        }
        let feet_y = self.pos[1] - self.feet;
        if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            if self.floor - feet_y > STEP_DOWN {
                self.state = State::Fall;
                self.v = [(s * self.speed) >> 6, 0, (c * self.speed) >> 6];
            } else {
                self.pos[1] = self.floor + self.feet;
            }
        } else if self.pending_jump.is_some() {
            self.pos[1] = self.floor + self.feet;
        } else if feet_y >= self.floor && self.v[1] >= 0 {
            self.pos[1] = self.floor + self.feet;
            self.v = [0; 3];
            self.state = State::Land;
            self.landed = true;
            // A long fall (state 10, 0x8005b2d8): Damage 5, message 31.
            if self.air >= LONG_FALL && self.falls_hurt {
                self.hurt(FALL_DAMAGE);
                self.hints.push(31);
            }
        } else if self.v[1] > 0 {
            self.state = State::Fall;
        }
        // Counted on the way down only (guess: a held jump's whole time
        // in the air would pass 193).
        self.air = if matches!(self.state, State::Jump | State::Fall) && self.v[1] > 0 { self.air + DT } else { 0 };
        // Running onto a slippery floor (flag 2, 0x8005c96c): message 32,
        // the slip (anim 33), then its fall.
        let on_floor = matches!(self.state, State::Walk | State::Run);
        if on_floor && self.speed > WALK && !self.slipping && !acting && self.anm(anim::SLIP).is_some() {
            let here = collide::query(col, [self.pos[0], self.pos[1] - self.feet - 1, self.pos[2]]);
            if collide::floor_flags(col, &here) & SLIPPERY != 0 {
                self.hints.push(32);
                self.slipping = true;
                self.speed = 0;
                self.state = State::Stand;
                self.effects.push(self.slip_sound);
                self.act(anim::SLIP, anims);
            }
        }
        if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            self.state = match self.speed {
                // The landing (state 10, animation 4) plays out first, the
                // stick or not: its handler (0x8005cba8) does not move the
                // body (the oracle: x and z held through state 8).
                _ if self.state == State::Land
                    && (self.animator.asked() != anim::LAND || self.animator.playing(anim::LAND)) =>
                {
                    State::Land
                }
                0 => State::Stand,
                s if s <= WALK => State::Walk,
                _ => State::Run,
            };
        }
        let a = match self.state {
            _ if self.hold.is_some() => self.hold.unwrap(),
            _ if self.action.is_some() && on_ground => self.action.unwrap(),
            State::Stand if self.carrying => anim::CARRY_IDLE,
            State::Stand => anim::IDLE,
            State::Walk => anim::STAND,
            State::Run => anim::RUN,
            State::Jump => {
                if self.speed > 0 {
                    anim::JUMP_MOVING
                } else {
                    anim::JUMP
                }
            }
            // A jump's way down is still the jump (state 5) in the
            // original: its rise runs on into its fall (5, 6). Falling
            // off an edge is state 4, animation 15.
            State::Fall if matches!(self.animator.asked(), anim::JUMP | anim::JUMP_MOVING) => self.animator.asked(),
            State::Fall => anim::FALL,
            State::Land => anim::LAND,
        };
        self.animator.set(a, anims);
        // The animation steps before the state's handler (0x8005a068), so
        // a landing's animation starts the frame after it lands.
        if !self.landed {
            self.animator.step(anims);
        }
        self.root_step(anims, col);
        if let Some((_, m)) = self.climb_motion(anims, trig)
            && let Some(c) = &mut self.climb
        {
            c.moved = m;
        }
        // Its animation over, the climb ends this frame (the next state
        // has begun under it).
        if self.climb.is_some() && !self.animator.playing(anim::CLIMB) {
            self.finish_climb(col, boxes, true);
            self.action = None;
        }
    }

    /// Floating (state 21, 0x8005db28): the stick moves at half speed, no
    /// gravity; Cross held rises 40 a frame, Circle sinks 40, else a bob of
    /// 256 about the anchor ((218 dt) >> 6 a frame).
    fn float_frame(&mut self, pads: &Pads, camera_heading: i32, col: &Col, anims: &Anims, trig: &Trig) {
        let Some((mut anchor, mut bob)) = self.float else { return };
        self.effects.clear();
        let start = self.pos;
        let held = |b: usize| pads.held(0, b);
        let sx = if held(input::LEFT) {
            -4096
        } else if held(input::RIGHT) {
            4096
        } else {
            0
        };
        let sz = if held(input::UP) {
            4096
        } else if held(input::DOWN) {
            -4096
        } else {
            0
        };
        if sx != 0 || sz != 0 {
            let dir = (((sx as f64).atan2(sz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
            self.target = (dir + camera_heading).rem_euclid(4096);
            self.speed = RUN / 2;
        } else {
            self.speed = 0;
        }
        let step = (self.turn_rate * DT) >> 6;
        let d = wrap(self.target - self.heading);
        self.heading = (self.heading + d.clamp(-step, step)).rem_euclid(4096);
        let (s, c) = (trig.sin(self.heading), trig.cos(self.heading));
        self.pos[0] += (s * self.speed * DT) >> 12;
        self.pos[2] += (c * self.speed * DT) >> 12;
        bob = (bob + ((218 * DT) >> 6)).rem_euclid(4096);
        if held(input::CROSS) {
            self.pos[1] -= (262 * DT) >> 6;
            anchor = self.pos[1];
        } else if held(input::CIRCLE) {
            self.pos[1] += (131 * DT) >> 5;
            anchor = self.pos[1];
        } else if self.speed == 0 {
            self.pos[1] = anchor + ((trig.sin(bob) * 256) >> 12);
        }
        // Walls and floors still stop the body.
        let q = collide::query(col, [self.pos[0], self.pos[1] - self.feet, self.pos[2]]);
        if q.kind != 0 {
            self.pos = start;
        }
        self.float = Some((anchor, bob));
        self.state = State::Stand;
        let a = if self.speed > 0 { anim::RUN } else { anim::STAND };
        self.animator.set(a, anims);
        self.animator.step(anims);
    }

    /// Runs as another body (0x80059328 swaps +0x154 and +0x15c): its
    /// hierarchy, its animations, its turn rate, the feet kept on the floor.
    /// The body's half width across a move and its reach along one: the
    /// larger x extent of the hierarchy's box (header +4 and +0xc,
    /// 0x80067ba4), both ways. Seen in the oracle: Tommy (250) stops 268
    /// short of a wall at 170 a frame; Spike (x 315, z -220..1000) 479
    /// short at 200, so his reach is not the box's 1000 along z.
    /// The step's probes against objects (0x80064a34 -> 0x80063d3c), as
    /// (x, z) from where it began: its body box's depth plus the step
    /// (speed +0x184 x D_800c3f1c, 10) straight on along `dir`, and the same
    /// at its half-width to either side (the oracle: running at the shut
    /// back door Tommy stops at 23020, walking at 23120, both on their
    /// step's grid, the probe's reach short of the panel).
    pub fn probes(&self, dir: [f32; 2], step: f32) -> Vec<(i32, i32)> {
        let (lo, hi) = self.body_box();
        let side = lo[0].abs().max(hi[0].abs()) as f32;
        let ahead = lo[2].abs().max(hi[2].abs()) as f32 + step;
        let [ux, uz] = dir;
        [0.0, -side, side].iter().map(|&w| ((ux * ahead + uz * w) as i32, (uz * ahead - ux * w) as i32)).collect()
    }

    /// The height the probes meet box `b` at: its own middle (unturned,
    /// 0x80063d3c) kept within the body's from `at`.
    pub fn probe_height(&self, b: &Box3, at: [i32; 3]) -> i32 {
        let (lo, hi) = self.body_box();
        let mid = ((b.lo[1] + b.hi[1]) as f32 / 2.0 + b.pos.y) as i32;
        mid.clamp(at[1] + lo[1], at[1] + hi[1])
    }

    /// The body's box (hierarchy header +4..+0x10: lo x, y, z, hi x, y,
    /// z; 0x80067ba4 without a carried thing).
    pub fn body_box(&self) -> ([i32; 3], [i32; 3]) {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32);
        ([h(0x04), h(0x06), h(0x08)], [h(0x0c), h(0x0e), h(0x10)])
    }

    fn extents(&self) -> (i32, i32) {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32).abs();
        let x = h(0x04).max(h(0x0c));
        (x, x)
    }

    /// The ground step (dx, dz) as the wall test lets it (0x80066014): three
    /// points ahead of the body, its reach past the step straight on and its
    /// half width to each side, must be in the open at a step's height; a
    /// wall there turns the whole step along whichever axis is open (the
    /// slide, 0x80061a64 / 0x80061b28: seen in the oracle, a run at 45
    /// degrees into a wall goes on along it at full speed). A ledge low
    /// enough to climb is left to the climb.
    fn ground_step(&self, col: &Col, dx: i32, dz: i32) -> (i32, i32) {
        if dx == 0 && dz == 0 {
            return (0, 0);
        }
        let feet = self.pos[1] - self.feet;
        let (side, ahead) = self.extents();
        let reach = if self.carrying && self.carry_climbs { CLIMB_CARRYING.min(self.climb_max) } else { self.climb_max };
        let open = |dx: i32, dz: i32| {
            let len = ((dx as f32).hypot(dz as f32)).max(1.0);
            let (ux, uz) = (dx as f32 / len, dz as f32 / len);
            let far = len + ahead as f32;
            [0.0, side as f32, -side as f32].iter().all(|&w| {
                let x = self.pos[0] + (ux * far + uz * w) as i32;
                let z = self.pos[2] + (uz * far - ux * w) as i32;
                let q = collide::query(col, [x, feet - STEP_UP, z]);
                if q.kind == 0 {
                    return true;
                }
                // A ledge the body can climb: the climb takes it.
                let up = collide::query(col, [x, feet - reach, z]);
                up.kind == 0 && up.floor >= feet - reach
            })
        };
        if open(dx, dz) {
            return (dx, dz);
        }
        // The whole speed along the wall (seen: 170 a frame at 17).
        let len = self.speed * DT;
        let along_x = (dx.signum() * len, 0);
        let along_z = (0, dz.signum() * len);
        let ways = if dx.abs() >= dz.abs() { [along_x, along_z] } else { [along_z, along_x] };
        ways.into_iter().find(|&(x, z)| (x != 0 || z != 0) && open(x, z)).unwrap_or((0, 0))
    }

    pub fn set_body(&mut self, hierarchy: Hierarchy, animator: Animator, turn_rate: i32, anims: &Anims) {
        let floor = self.pos[1] - self.feet;
        self.feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        self.pos[1] = floor + self.feet;
        self.hierarchy = hierarchy;
        self.animator = animator;
        self.turn_rate = turn_rate;
        self.animator.set(anim::IDLE, anims);
    }

    /// The feet's height above the root (the hierarchy's root offset).
    pub fn feet(&self) -> i32 {
        self.feet
    }

    /// Thrown through the air (state 16, 0x8005ec9c): along `heading` at
    /// `speed`, up at `up`.
    /// The climb ends where its root motion got to (+0xd8, +0xdc moved by
    /// 0x80066e60), on the ledge it rose to, if there is room there.
    fn finish_climb(&mut self, col: &Col, boxes: &[Box3], whole: bool) {
        let Some(mut c) = self.climb.take() else { return };
        // Run to its end, the last step's motion is had too; the height it
        // ends at against the top is the floor's to take back (the oracle:
        // Chuckie at -193 over a top of -179, then -192).
        if whole {
            c.moved = c.whole;
            self.root_y = c.from[1] + c.whole[1] - c.top;
        }
        let top = c.top - self.feet;
        self.pos[1] = c.top;
        self.floor = top;
        let at = [c.from[0] + c.moved[0], self.pos[1], c.from[2] + c.moved[2]];
        let q = collide::query(col, [at[0], top - 1, at[2]]);
        let on_col = q.kind == 0 && (q.floor - top).abs() <= STEP_UP;
        let on_box = boxes.iter().any(|b| b.covers(at, 0) && (b.min_y - top).abs() <= STEP_UP);
        // Room for the body there: no box over the top within its height.
        let room = !boxes.iter().any(|b| b.covers(at, RADIUS) && b.min_y < top - STEP_UP && b.max_y > top - HEIGHT);
        if (on_col || on_box) && room {
            self.pos[0] = at[0];
            self.pos[2] = at[2];
        }
    }

    /// The base track's root y this step (0x80055904 into +0xda), and the
    /// jump's take-off when its airborne state comes.
    fn root_step(&mut self, anims: &Anims, col: &Col) {
        let (state, first, count, cur, _) = self.animator.track();
        let (done, total) = self.animator.root_progress(anims).map_or((0, 0), |r| (r.0[1], r.1[1]));
        let was = self.root_seen;
        // An animation that ran to its last frame gives the rest of it.
        let rest = if was.cur + 1 == was.first + was.count && !was.air { was.total - was.done } else { 0 };
        let dy = match (was.state, was.first) == (state, first) {
            // Round again: the rest of the loop, then this pass's part.
            true if cur < was.cur => rest + done,
            true => done - was.done,
            false => rest + done,
        };
        // Root y seen in the air or in a climb does not carry on (the climb
        // puts the body on the top itself).
        let air =
            (matches!(self.state, State::Jump | State::Fall) && self.pending_jump.is_none()) || self.climb.is_some();
        self.root_seen = RootSeen { air, state, first, count, cur, done, total };
        if self.climb.is_some() {
            return;
        }
        if let Some(v) = self.pending_jump {
            // The crouch is table state 5: off as it ends, its last part not
            // had (the oracle: 868, 868, then 814). A moving jump's state
            // (41) is off the frame after it comes, with nothing more of
            // its keys (the run's 751, then 731 in state 41, then 585).
            let off = state != 5 && (was.state == 5 || state == was.state);
            if state != was.state {
                self.root_y += done;
            } else if !off {
                self.root_y += dy;
            }
            if off {
                self.pending_jump = None;
                self.pos[1] += self.root_y;
                self.root_y = 0;
                // Off this frame: the air's first step with it (the
                // animation steps before the state's handler, 0x8005a068).
                self.v = v;
                let before = self.pos;
                for k in [0, 2] {
                    self.pos[k] += (self.v[k] * DT) >> 6;
                }
                self.pos[1] += (self.v[1] * DT + GRAVITY * DT * DT / 2) >> 6;
                self.v[1] += GRAVITY * DT;
                // The step across meets walls as any air step does
                // (0x8005ee78): into one, it is not had.
                let q = collide::query(col, [self.pos[0], self.pos[1] - self.feet - 1, self.pos[2]]);
                if q.kind != 0 {
                    self.pos[0] = before[0];
                    self.pos[2] = before[2];
                }
            }
            return;
        }
        if matches!(self.state, State::Jump | State::Fall) {
            return;
        }
        // On the ground the body follows the animation's root curve from
        // the floor; a new animation (or a loop's next round) starts from
        // where the body was, which its first frame blends away (+0xde at
        // the frame's start, 0x80055904). The oracle after a climb: -55
        // over the floor, then -41, -45, -40, -23, -18, -10, +1, the run's
        // own curve from its second frame; after a landing, 89 under it
        // and 23 less a frame through the idle's first frame.
        if (was.state, was.first) != (state, first) || cur < was.cur {
            // From where the last one left the body, its end had.
            self.root_base = self.root_y + rest;
        }
        let (.., left) = self.animator.track();
        let blend = if cur > first {
            1.0
        } else {
            let dur = anims.frames.get(first).map_or(1, |f| f.duration.max(1) as i32);
            ((dur - left) as f32 / dur as f32).clamp(0.0, 1.0)
        };
        self.root_y = done + (self.root_base as f32 * (1.0 - blend)) as i32;
    }

    pub fn launch(&mut self, heading: i32, up: i32, speed: i32, trig: &Trig) {
        self.pending_jump = None;
        self.heading = heading;
        self.target = heading;
        self.speed = 0;
        self.action = None;
        self.v = [(trig.sin(heading) * speed) >> 12, up, (trig.cos(heading) * speed) >> 12];
        self.state = State::Jump;
    }

    /// Starts action animation `a`.
    pub fn act(&mut self, a: i16, anims: &Anims) {
        self.act_at(a, self.animator.written_speed.max(2048), anims);
    }

    /// Starts action animation `a` at animation speed `speed` (0x800572a8's
    /// third argument).
    pub fn act_at(&mut self, a: i16, speed: i32, anims: &Anims) {
        self.action = Some(a);
        self.speed = 0;
        self.animator.start_at(a, speed, anims);
    }

    /// The heading as a rotation (0x80020d10's matrix).
    pub fn rotation(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        Mat3::from_cols_array(
            &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
    }

    /// A climb's root motion so far, turned and scaled into the world
    /// (0x80055904: a body other than 0x800c5680's, 1 Tommy, moves by its
    /// root offset, hierarchy +0x4c, over 0x800c41dc's -561). The y is the
    /// animation's own: the oracle's Tommy rose 904 for a 900 band's
    /// climb, then settled on the top.
    fn climb_motion(&self, anims: &Anims, trig: &Trig) -> Option<(Climb, [i32; 3])> {
        let c = self.climb?;
        (self.action == Some(anim::CLIMB) && self.animator.playing(anim::CLIMB)).then_some(())?;
        let (done, _) = self.animator.root_progress(anims)?;
        // The player's body is 0x800c5680's own character, never scaled
        // (0x80055904 scales only other bodies, by +0x4c over 0x800c41dc).
        let m = self.rotation(trig) * Vec3::new(done[0] as f32, done[1] as f32, done[2] as f32);
        Some((c, [m.x as i32, m.y as i32, m.z as i32]))
    }

    /// Where the body is drawn: during a climb, where its root motion has
    /// carried it from where the climb began; else its position.
    fn drawn_at(&self, anims: &Anims, trig: &Trig) -> Vec3 {
        let pos = Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32);
        match self.climb_motion(anims, trig) {
            Some((c, m)) => Vec3::new((c.from[0] + m[0]) as f32, (c.from[1] + m[1]) as f32, (c.from[2] + m[2]) as f32),
            None => pos + Vec3::new(0.0, self.root_y as f32, 0.0),
        }
    }

    pub fn parts(&self, anims: &Anims, trig: &Trig) -> Vec<(Mat3, Vec3)> {
        let pos = self.drawn_at(anims, trig);
        models::posed_parts(&self.hierarchy, &self.animator.locals(&self.hierarchy, anims), self.rotation(trig), pos)
    }

    /// The part a hand slot names (hierarchy +0x14 + slot).
    pub fn slot_part(&self, slot: usize) -> Option<usize> {
        self.hierarchy.header.get(0x14 + slot).copied().filter(|&p| p != 0xff).map(|p| p as usize)
    }

    /// The player's triangles, turned by its heading (0x80020d10).
    pub fn triangles(&self, models: &Models, anims: &Anims, trig: &Trig, out: &mut Vec<Vtx>) {
        let e = trig.euler([self.heading as i16, 0, 0]);
        let rows = Mat3::from_cols_array(
            &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
        .transpose();
        let pos = self.drawn_at(anims, trig);
        posed_triangles(
            &self.hierarchy,
            models,
            &self.animator.locals(&self.hierarchy, anims),
            rows.transpose(),
            pos,
            out,
        );
    }
}

/// How far below the feet a floor may drop and still be walked onto, and
/// how far above them a box's top may stand and still be stepped onto
/// (0x80066014 tests 384).
const STEP_DOWN: i32 = 384;
const STEP_UP: i32 = 384;
/// The body's radius and height for the object test.
pub const RADIUS: i32 = 160;
pub const HEIGHT: i32 = 600;
/// A climb under way: where it began, and its root motion so far in the
/// world.
#[derive(Clone, Copy, Debug)]
struct Climb {
    /// The body's height on the top (the ledge plus the feet).
    top: i32,
    /// The climb animation's whole root motion, in the world.
    whole: [i32; 3],
    from: [i32; 3],
    moved: [i32; 3],
}

/// The base track as `Player::root_step` last saw it.
#[derive(Clone, Copy, Debug, Default)]
struct RootSeen {
    /// Seen in the air: none of it carries on to the ground.
    air: bool,
    state: i16,
    first: usize,
    count: usize,
    cur: usize,
    done: i32,
    total: i32,
}

/// The highest ledge a climb takes (0x80056e44: under 1034; under 522
/// while carrying).
pub const CLIMB: i32 = 1033;
const CLIMB_CARRYING: i32 = 521;
/// The highest step message 16 calls too tall (0x80058da4).
const TOO_TALL: i32 = 3071;
/// A landing after this long falling hurts (0x8005b2d8).
const LONG_FALL: i32 = 193;
const FALL_DAMAGE: i32 = 5;
/// Floor flag 2 (slippery) and the slip's fall's damage.
const SLIPPERY: u8 = 2;
const SLIP_DAMAGE: i32 = 5;

/// An object's collision box: its model box (`lo`, `hi`) in the object's
/// space, the object's rotation and position, and the world heights it
/// spans (y down: `min_y` its top).
#[derive(Clone, Copy, Debug)]
pub struct Box3 {
    pub lo: [i32; 3],
    pub hi: [i32; 3],
    pub rot: Mat3,
    pub pos: Vec3,
    pub min_y: i32,
    pub max_y: i32,
    /// Its template has flag 64: it can be climbed onto (0x80058da4).
    pub climb: bool,
    /// Another character's body (no climb test).
    pub character: bool,
    /// The placed object's template (0 for none).
    pub template: u16,
    /// Seen from above, when it is turned other than about y.
    pub outline: Outline,
}

/// A box's outline seen from above when it is turned other than about y
/// (a thing tipped on its side, level 3's templates 526 and 650): its
/// corners' convex hull, counter-clockwise in (x, z); two points for one
/// edge on (level 6's wheels, flat in their own plane). Empty (`n` 0) for
/// one turned only about y, its own x and z rectangle.
#[derive(Clone, Copy, Debug, Default)]
pub struct Outline {
    pts: [[f32; 2]; 8],
    n: u8,
}

impl Outline {
    /// The outline of the box `lo`..`hi` turned by `rot` at `pos`.
    pub fn of(lo: [i32; 3], hi: [i32; 3], rot: Mat3, pos: Vec3) -> Outline {
        if (rot.y_axis.y - 1.0).abs() < 1e-3 {
            return Outline::default();
        }
        let mut pts = [[0.0f32; 2]; 8];
        for (k, p) in pts.iter_mut().enumerate() {
            let c = Vec3::new(
                if k & 1 == 0 { lo[0] } else { hi[0] } as f32,
                if k & 2 == 0 { lo[1] } else { hi[1] } as f32,
                if k & 4 == 0 { lo[2] } else { hi[2] } as f32,
            );
            let w = pos + rot * c;
            *p = [w.x, w.z];
        }
        // The convex hull (monotone chain).
        pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
        let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
        let mut hull = [[0.0f32; 2]; 16];
        let mut n = 0;
        for pass in 0..2 {
            let start = n;
            for k in 0..8 {
                let p = pts[if pass == 0 { k } else { 7 - k }];
                while n >= start + 2 && cross(hull[n - 2], hull[n - 1], p) <= 0.0 {
                    n -= 1;
                }
                hull[n] = p;
                n += 1;
            }
            n -= 1;
        }
        let mut out = Outline { pts: [[0.0; 2]; 8], n: n.min(8) as u8 };
        out.pts[..out.n as usize].copy_from_slice(&hull[..out.n as usize]);
        out
    }

    /// Its edges: each start and outward normal (x, z).
    fn edges(&self) -> impl Iterator<Item = ([f32; 2], [f32; 2])> + '_ {
        let n = self.n as usize;
        (0..n).map(move |k| {
            let (e0, e1) = (self.pts[k], self.pts[(k + 1) % n]);
            (e0, [e1[1] - e0[1], e0[0] - e1[0]])
        })
    }
}

impl Box3 {
    /// Whether a circle of radius `r` at `p` (x, z) overlaps the box seen
    /// from above.
    pub fn covers(&self, p: [i32; 3], r: i32) -> bool {
        self.depth(p, r) > 0.0
    }

    /// Whether the step from `a` to `b` (x, z) passes through the box's
    /// own rectangle seen from above (0x80069c40 tests the movement
    /// segment): a thin wall cannot be stepped across.
    pub fn crosses(&self, a: [i32; 3], b: [i32; 3]) -> bool {
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        let mut clip = |o: f32, d: f32, lo: f32, hi: f32| {
            if d.abs() < 1e-6 {
                return lo <= o && o <= hi;
            }
            let (mut u, mut v) = ((lo - o) / d, (hi - o) / d);
            if u > v {
                std::mem::swap(&mut u, &mut v);
            }
            t0 = t0.max(u);
            t1 = t1.min(v);
            t0 <= t1
        };
        match self.outline.n {
            0 => {
                let local = |p: [i32; 3]| self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
                let (p, q) = (local(a), local(b));
                for i in [0, 2] {
                    if !clip(p[i], q[i] - p[i], self.lo[i] as f32, self.hi[i] as f32) {
                        return false;
                    }
                }
            }
            // Edge on: the step crosses its segment.
            1 | 2 => {
                let (e0, e1) = (self.outline.pts[0], self.outline.pts[(self.outline.n - 1) as usize]);
                let (pa, pb) = ([a[0] as f32, a[2] as f32], [b[0] as f32, b[2] as f32]);
                let cross = |o: [f32; 2], p: [f32; 2], q: [f32; 2]| (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0]);
                let (d1, d2) = (cross(e0, e1, pa), cross(e0, e1, pb));
                let (d3, d4) = (cross(pa, pb, e0), cross(pa, pb, e1));
                // Not when the step starts on it.
                return d1 != 0.0 && d1.signum() != d2.signum() && d3.signum() != d4.signum();
            }
            _ => {
                // Each edge's outward normal: the segment's part inside it.
                let (pa, pb) = ([a[0] as f32, a[2] as f32], [b[0] as f32, b[2] as f32]);
                for (e0, n) in self.outline.edges() {
                    let o = n[0] * (pa[0] - e0[0]) + n[1] * (pa[1] - e0[1]);
                    let d = n[0] * (pb[0] - pa[0]) + n[1] * (pb[1] - pa[1]);
                    if !clip(o, d, f32::NEG_INFINITY, 0.0) {
                        return false;
                    }
                }
            }
        }
        // Not when the step only leaves it (it began inside).
        t0 > 0.0
    }

    /// Where the segment from `a` to `b` first enters the box (in 3D, in
    /// the box's own frame): the fraction along it, the point, and the
    /// outward normal (4096 a unit) of the face it enters by. None when it
    /// misses or starts inside.
    pub fn segment_hit(&self, a: [i32; 3], b: [i32; 3]) -> Option<(f32, [i32; 3], [i32; 3])> {
        let local = |p: [i32; 3]| self.rot.transpose() * (Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) - self.pos);
        let (p, q) = (local(a), local(b));
        let d = q - p;
        let (mut t0, mut t1, mut axis) = (0.0f32, 1.0f32, None);
        for k in 0..3 {
            let (lo, hi) = (self.lo[k] as f32, self.hi[k] as f32);
            if d[k].abs() < 1e-6 {
                if p[k] < lo || p[k] > hi {
                    return None;
                }
                continue;
            }
            let (mut u, mut v, mut sign) = ((lo - p[k]) / d[k], (hi - p[k]) / d[k], -1.0);
            if u > v {
                std::mem::swap(&mut u, &mut v);
                sign = 1.0;
            }
            if u > t0 {
                t0 = u;
                axis = Some((k, sign));
            }
            t1 = t1.min(v);
            if t0 > t1 {
                return None;
            }
        }
        let (k, sign) = axis?;
        let mut n = Vec3::ZERO;
        n[k] = sign;
        let n = self.rot * n;
        let at =
            Vec3::new(a[0] as f32, a[1] as f32, a[2] as f32).lerp(Vec3::new(b[0] as f32, b[1] as f32, b[2] as f32), t0);
        Some((
            t0,
            [at.x as i32, at.y as i32, at.z as i32],
            [(n.x * 4096.0) as i32, (n.y * 4096.0) as i32, (n.z * 4096.0) as i32],
        ))
    }

    /// The outward normal (4096 a unit, in the world) of the side of the
    /// box seen from above that `p`, outside it, is beyond the most.
    pub fn side_normal(&self, p: [i32; 3]) -> [i32; 3] {
        let local = self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
        let sides = [
            (self.lo[0] as f32 - local.x, Vec3::NEG_X),
            (local.x - self.hi[0] as f32, Vec3::X),
            (self.lo[2] as f32 - local.z, Vec3::NEG_Z),
            (local.z - self.hi[2] as f32, Vec3::Z),
        ];
        let (_, n) = sides.into_iter().fold((f32::MIN, Vec3::X), |a, s| if s.0 > a.0 { s } else { a });
        let n = self.rot * n;
        [(n.x * 4096.0) as i32, 0, (n.z * 4096.0) as i32]
    }

    /// How deep a circle of radius `r` at `p` is in the box seen from above:
    /// the distance to the nearest side, negative outside.
    pub fn depth(&self, p: [i32; 3], r: i32) -> f32 {
        let r = r as f32;
        match self.outline.n {
            0 => {
                let local = self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
                let x = (local.x - (self.lo[0] as f32 - r)).min(self.hi[0] as f32 + r - local.x);
                let z = (local.z - (self.lo[2] as f32 - r)).min(self.hi[2] as f32 + r - local.z);
                x.min(z)
            }
            // How far inside each edge (its outward normal a unit).
            // Edge on (a wheel standing in its own plane): a segment.
            1 | 2 => {
                let (a, b) = (self.outline.pts[0], self.outline.pts[(self.outline.n - 1) as usize]);
                let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
                let (px, pz) = (p[0] as f32 - a[0], p[2] as f32 - a[1]);
                let t = ((px * dx + pz * dz) / (dx * dx + dz * dz).max(1e-6)).clamp(0.0, 1.0);
                r - (px - t * dx).hypot(pz - t * dz)
            }
            _ => self
                .outline
                .edges()
                .map(|(e0, n)| {
                    let len = n[0].hypot(n[1]).max(1e-6);
                    r - (n[0] * (p[0] as f32 - e0[0]) + n[1] * (p[2] as f32 - e0[1])) / len
                })
                .fold(f32::INFINITY, f32::min),
        }
    }

}

/// An angle difference in -2048..2048.
fn wrap(a: i32) -> i32 {
    (a + 2048).rem_euclid(4096) - 2048
}
